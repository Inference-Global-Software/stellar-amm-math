#!/usr/bin/env bash
# End-to-end test of the contract on a local Stellar network.
#
# Starts the network compose.yaml defines, deploys out/main.wasm with the
# pinned stellar CLI and checks that
#   - the contract's interface is e2e/interface.txt, byte for byte
#   - the bytes the network holds are out/main.wasm
#   - every row of e2e/vectors.tsv holds: a value row prints exactly its
#     expected value and exits 0, a TRAP row exits 1 with the host reporting
#     UnreachableCodeReached
#   - every row of e2e/vectors.tsv was called
# and prints a summary line. Pass or fail, it then stops the network and
# removes its volumes, unless KEEP=1 or the network existed before it (see
# below). If the network does not become healthy within 300 seconds, it
# prints the health checks and the container's last log lines first.
#
# Before it calls anything, it checks the table: every row is well formed,
# every method of the interface has a value row, and every method that takes
# arguments has a TRAP row.
#
# If this project's network exists already (left by
# scripts/deploy-testnet.sh --network local, or by KEEP=1), it says so, runs
# on that network as it is, with whatever it holds, and never removes it: a
# running network is left running, and a stopped one (after `docker compose
# stop`, or a restart of Docker) is started for the run and stopped again.
# Set COMPOSE_PROJECT_NAME to run on a fresh network instead.
#
# Usage: scripts/e2e-local.sh      (from anywhere; no arguments; needs Docker
#                                  with Compose v2)
#   KEEP=1                leave the network running (with the funded `e2e`
#                         identity); remove it later with the command it prints
#   COMPOSE_PROJECT_NAME  compose project name (default: stellar-amm-math)
#   STELLAR_PORT          the network's port on the host's loopback interface
#                         (default: 8000); the CLI does not use it, but it
#                         must be free when the network starts
#
# The CLI reads out/main.wasm through a bind mount, so the Docker daemon must
# see this checkout at its own path: not a remote daemon, nor
# docker-outside-of-docker from a container whose workspace path differs.
#
# Results go to stdout, one `ok` or `FAIL` line per check and the summary;
# progress and the CLI's own messages, a failing call's included, go to
# stderr.
# Bash only: under another shell (dash's sh, zsh) stop with a clear message.
[ -n "${BASH_VERSION:-}" ] || { echo "error: $0 needs bash: run it with bash '$0'" >&2; exit 2; }
set -euo pipefail
unset CDPATH
cd "$(dirname "$0")/.."
. scripts/lib.sh

[[ $# -eq 0 ]] || die "usage: scripts/e2e-local.sh (no arguments; see its header for the variables it reads)" 2
for f in out/main.wasm e2e/interface.txt e2e/vectors.tsv; do
  [[ -f "$f" ]] || die "no $f" 2
done
need_compose

# The rows must be well formed. Every method of the interface needs a value
# row, and a TRAP row unless it takes no arguments (its `fn` line names only
# `env`), because a table of only TRAP rows (or only values) for a method
# would pass on a contract that computes it wrongly (or never refuses).
# Methods and arguments are Soroban symbols: [A-Za-z0-9_].
name_re='^[A-Za-z0-9_]+$'
args_re='^(-|[A-Za-z0-9_]+=[0-9]+( [A-Za-z0-9_]+=[0-9]+)*)$'
expected_re='^([0-9]+|true|false|TRAP)$'
line=0 rows=0
while IFS= read -r row || [[ -n "$row" ]]; do
  line=$((line + 1))
  [[ -z "$row" || "$row" == \#* ]] && continue
  IFS=$'\t' read -r method args expected note <<< "$row"
  [[ "$method" =~ $name_re && "$args" =~ $args_re && "$expected" =~ $expected_re && -n "$note" ]] ||
    die "e2e/vectors.tsv line $line is not method<TAB>args<TAB>expected<TAB>note"
  rows=$((rows + 1))
done < e2e/vectors.tsv
[[ $rows -gt 0 ]] || die "e2e/vectors.tsv has no rows"
methods=$(sed -n 's/^ *fn \([^(]*\)(.*/\1/p' e2e/interface.txt)
[[ -n "$methods" ]] || die "e2e/interface.txt names no method"
# has_row METHOD value|TRAP: whether e2e/vectors.tsv has such a row for METHOD.
has_row() {
  awk -F '\t' -v m="$1" -v kind="$2" '
    $1 == m && (kind == "TRAP") == ($3 == "TRAP") { found = 1 }
    END { exit !found }' e2e/vectors.tsv
}
while IFS= read -r method; do
  [[ "$method" =~ $name_re ]] || die "e2e/interface.txt has a method that is not a Soroban symbol: '$method'"
  has_row "$method" value || die "e2e/vectors.tsv has no value row for $method"
  if ! grep -qE "^ *fn $method\([A-Za-z0-9_]+: [^,]*\)" e2e/interface.txt; then
    has_row "$method" TRAP || die "e2e/vectors.tsv has no TRAP row for $method"
  fi
done <<< "$methods"

# teardown: stop the network and remove its volumes, unless it existed
# before this run (network_reused running or stopped), which leaves it as it
# was, or KEEP=1 keeps it running. Until start_network has looked
# (network_reused empty), this run has started nothing, so there is nothing
# to remove.
tmp=$(mktemp -d)
teardown() {
  rm -rf "$tmp"
  if [[ -z "$network_reused" ]]; then
    return
  elif [[ "${KEEP:-}" == 1 && $network_started -eq 1 ]]; then
    echo "KEEP=1: the network is still running; remove it and its deployments with: $(down_command)" >&2
  elif [[ "$network_reused" == running ]]; then
    echo "the network was running before this run, so it is left running; remove it and its deployments with: $(down_command)" >&2
  elif [[ "$network_reused" == stopped ]]; then
    if "${DC[@]}" stop stellar >/dev/null 2>&1; then
      echo "the network was stopped before this run, so it is stopped again, with what it holds; start it with: $(start_command)" >&2
      echo "or remove it and its deployments with: $(down_command)" >&2
    else
      echo "warning: could not stop the network again; run: docker compose -f $(printf %q "$PWD/compose.yaml") -p $(printf %q "$(project_name)") stop stellar" >&2
    fi
  else
    "${DC[@]}" --profile '*' down -v >/dev/null 2>&1 ||
      echo "warning: could not stop the network; run: $(down_command)" >&2
  fi
}
at_exit teardown

# The CLI container; stdin is closed so that it cannot read the vectors file.
# It runs only once start_network has the network healthy, and --no-deps
# keeps each call from converging the stellar service again, which would
# take time and could recreate it. A stellar container that stops still
# fails every call: the CLI cannot join its network namespace.
cli() { "${DC[@]}" run --rm -T --no-deps cli "$@" </dev/null; }

start_network

echo "funding the e2e identity" >&2
cli keys generate e2e --overwrite >&2
fund e2e local

failed=0
cli contract info interface --wasm out/main.wasm > "$tmp/interface.txt"
if diff -u e2e/interface.txt "$tmp/interface.txt" >&2; then
  echo "ok   the interface is e2e/interface.txt"
else
  echo "FAIL the interface differs from e2e/interface.txt (+ lines are the contract's; the diff is on stderr)"
  failed=$((failed + 1))
fi

# deploy prints the contract id alone on stdout; its progress goes to stderr.
id=$(cli contract deploy --wasm out/main.wasm --source e2e --network local)
[[ "$id" =~ ^C[A-Z2-7]{55}$ ]] || die "deploy printed no contract id: '$id'"
echo "deployed $id" >&2

cli contract fetch --id "$id" --network local > "$tmp/deployed.wasm"
if cmp -s out/main.wasm "$tmp/deployed.wasm"; then
  echo "ok   the network holds out/main.wasm ($(sha256 out/main.wasm))"
else
  echo "FAIL the network holds $(sha256 "$tmp/deployed.wasm"), out/main.wasm is $(sha256 out/main.wasm)"
  failed=$((failed + 1))
fi

# The table is read on its own descriptor, 3, so that no command in the loop
# can consume its rows from stdin; the count below also catches that.
passed=0 values=0 traps=0
while IFS=$'\t' read -r -u 3 method args expected note || [[ -n "$method" ]]; do
  [[ -z "$method" || "$method" == \#* ]] && continue
  call=("$method")
  if [[ "$args" != - ]]; then
    read -r -a pairs <<< "$args"
    for pair in ${pairs[@]+"${pairs[@]}"}; do
      call+=("--${pair%%=*}" "${pair#*=}")
    done
  fi
  status=0
  out=$(cli contract invoke --id "$id" --source e2e --network local --send=no -- "${call[@]}" \
    2> "$tmp/stderr") || status=$?
  if [[ "$expected" == TRAP ]]; then
    traps=$((traps + 1))
    if [[ $status -eq 1 ]] && grep -q UnreachableCodeReached "$tmp/stderr"; then
      passed=$((passed + 1))
      echo "ok   $method $args: TRAP"
      continue
    fi
    echo "FAIL $method $args: expected TRAP ($note), got exit $status, stdout '$out'"
  else
    values=$((values + 1))
    if [[ $status -eq 0 && "$out" == "$expected" ]]; then
      passed=$((passed + 1))
      echo "ok   $method $args: $out"
      continue
    fi
    echo "FAIL $method $args: expected $expected ($note), got exit $status, stdout '$out'"
  fi
  {
    echo "the CLI's messages for: $method $args"
    sed 's/^/     /' "$tmp/stderr"
  } >&2
  failed=$((failed + 1))
done 3< e2e/vectors.tsv

called=$((values + traps))
if [[ $called -ne $rows ]]; then
  echo "FAIL called $called of the $rows rows of e2e/vectors.tsv"
  failed=$((failed + 1))
fi
echo "e2e: $passed of $rows rows passed ($values values, $traps traps called); $failed checks failed"
[[ $failed -eq 0 ]] || exit 1
finish
