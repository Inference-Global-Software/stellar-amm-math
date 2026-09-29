#!/usr/bin/env bash
# Checks a deployment that scripts/deploy-testnet.sh recorded: the contract is
# still on the network and live, its bytes are the recorded ones and
# out/main.wasm, and it computes the reference swap.
#
# Usage: scripts/check-testnet.sh [--network testnet|local] [--record FILE]
#   --network  testnet (the default) or local; it must match the record. The
#              local network must already be running: this never starts,
#              recreates or stops it. A stopped one still holds its
#              deployments, and the error says how to start it again
#   --record   the record to check (default: deployments/testnet.json on
#              testnet, and .ctx/deployments/local.json on the local
#              network, where scripts/deploy-testnet.sh writes it: under
#              .ctx/, which git ignores, so that it can never be committed)
#
# It asks the network for its passphrase and stops unless it is that
# network's; checks the recorded contract id's checksum; reads the time to
# live of the contract instance and of its Wasm code; fetches the contract
# and compares the bytes' SHA-256 with the record's wasm_sha256 and with
# out/main.wasm; generates and funds a throwaway identity (amm-check); and
# invokes get_amount_out(1000, 100000, 100000, 30), which must return 987.
#
# Exit status:
#   0  all checks pass
#   1  anything unexpected, such as an unreachable network or a failed funding
#   2  usage error, unreadable record, invalid contract id or missing tool
#   3  contract not found: testnet was reset (or the local network restarted)
#      since the deployment, or the local network is not running (stopped,
#      or not there at all); re-run scripts/deploy-testnet.sh, or start a
#      stopped network again as the error says
#   4  deployed bytes differ from the record, or out/main.wasm does
#   5  wrong answer: a value other than 987, or a trap
#   6  archived: the time to live of the contract instance or of its code
#      ran out; restore both (stellar contract restore --id ID, and --wasm
#      out/main.wasm) or re-run scripts/deploy-testnet.sh
# Needs Docker with Compose v2, and jq. The CLI's container bind-mounts out/,
# so the Docker daemon must see this checkout at its own path: not a remote
# daemon, nor docker-outside-of-docker from a container whose workspace path
# differs.
# Bash only: under another shell (dash's sh, zsh) stop with a clear message.
[ -n "${BASH_VERSION:-}" ] || { echo "error: $0 needs bash: run it with bash '$0'" >&2; exit 2; }
set -euo pipefail
unset CDPATH
caller=$PWD
cd "$(dirname "$0")/.."
. scripts/lib.sh

usage() { die "usage: scripts/check-testnet.sh [--network testnet|local] [--record FILE]" 2; }

network=testnet record=
while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) [[ $# -ge 2 ]] || usage; network=$2; shift 2 ;;
    --record) [[ $# -ge 2 ]] || usage; record=$2; shift 2 ;;
    *) usage ;;
  esac
done
no_deps=
case "$network" in
  testnet) service=cli-net default_record=deployments/testnet.json ;;
  local) service=cli default_record=.ctx/deployments/local.json no_deps=1 ;;
  *) usage ;;
esac
if [[ -z "$record" ]]; then
  record=$default_record
elif [[ "$record" != /* ]]; then
  record=$caller/$record
fi
need_compose
command -v jq >/dev/null 2>&1 || die "this needs jq" 2
[[ -f "$record" ]] || die "no record $record; deploy with scripts/deploy-testnet.sh --network $network" 2
jq -e 'type == "object"' "$record" >/dev/null 2>&1 || die "$record is not a JSON object" 2
field() { jq -r --arg k "$1" '.[$k] // empty' "$record"; }
recorded_network=$(field network)
id=$(field contract_id)
recorded_sha=$(field wasm_sha256)
recorded_until=$(field live_until_ledger)
[[ "$recorded_network" == "$network" ]] ||
  die "$record is a deployment on '${recorded_network:-nothing}', not $network" 2
[[ "$id" =~ ^C[A-Z2-7]{55}$ ]] || die "$record has no valid contract_id" 2
[[ "$recorded_sha" =~ ^[0-9a-f]{64}$ ]] || die "$record has no valid wasm_sha256" 2
[[ -z "$recorded_until" || "$recorded_until" =~ ^[0-9]+$ ]] || die "$record has an invalid live_until_ledger" 2
[[ -f out/main.wasm ]] || die "no out/main.wasm; build it with infs build" 2

# This checks the local network the deployment went to, as it is, and never
# starts one. Without --no-deps, each cli call would start a stopped network
# again, and would recreate one, running or stopped, whose configuration
# compose sees as changed (another STELLAR_PORT, or another Compose
# version), with the deployment on it. A network that stops during the
# check fails it.
if [[ "$network" == local ]]; then
  state=$(network_state) || exit 1
  case "$state" in
    stopped) die "the local network is stopped; a stopped network keeps its deployments: start it with $(start_command) and check again" 3 ;;
    none) die "the local network is not running, so it holds no deployment; deploy with scripts/deploy-testnet.sh --network local" 3 ;;
  esac
fi

cli() { "${DC[@]}" run --rm -T ${no_deps:+--no-deps} "$service" "$@" </dev/null; }
tmp=$(mktemp -d)
remove_tmp() { rm -rf "$tmp"; }
at_exit remove_tmp

check_network_identity "$network"

# Offline: an id with a bad checksum is taken for an alias, and "not found"
# without asking the network. cli-net, which needs no local network, runs
# it, so that a local network that stops now fails a later call, and not
# this one, which would blame the record.
if ! "${DC[@]}" run --rm -T cli-net strkey decode "$id" </dev/null > "$tmp/strkey.json" 2> "$tmp/stderr" ||
  ! jq -e '.contract' "$tmp/strkey.json" >/dev/null 2>&1; then
  die "$record has no valid contract_id: $id fails its checksum or is not a contract" 2
fi

# not_found: exit 3, or 6 if the record says the entries' time to live ran
# out before the ledger the network is at (an archived entry may be reported
# as absent).
not_found() {
  if [[ -n "$recorded_until" && $1 -gt $recorded_until ]]; then
    die "contract $id is archived on $network: the network is at ledger $1, and the record says it lived until ledger $recorded_until; restore it (stellar contract restore) or re-run scripts/deploy-testnet.sh" 6
  fi
  if [[ "$network" == testnet ]]; then
    die "contract $id not found on testnet (testnet reset? re-run scripts/deploy-testnet.sh)" 3
  fi
  die "contract $id not found on the local network (restarted? re-run scripts/deploy-testnet.sh --network local)" 3
}

# check_live WHAT JSON: fail unless JSON (from `ledger entry fetch`) holds
# an entry that is live, and say until when.
check_live() {
  local live_to latest
  latest=$(jq -r '.latestLedger' "$2")
  live_to=$(jq -r '.entries[0].liveUntilLedgerSeq // empty' "$2")
  [[ "$latest" =~ ^[0-9]+$ ]] || die "the $network network reported no latest ledger"
  [[ -n "$live_to" ]] || not_found "$latest"
  [[ "$live_to" =~ ^[0-9]+$ ]] || die "the $network network reported no time to live for $1"
  if [[ $live_to -lt $latest ]]; then
    die "contract $id is archived on $network: $1 lived until ledger $live_to, and the network is at $latest; restore it (stellar contract restore) or re-run scripts/deploy-testnet.sh" 6
  fi
  echo "$1 lives until ledger $live_to (the network is at $latest: about $(((live_to - latest) * 5 / 86400)) days at 5 s a ledger)" >&2
}

echo "reading the time to live of $id on $network" >&2
cli ledger entry fetch contract-data --contract "$id" --instance --network "$network" --output json \
  > "$tmp/instance.json" || die "could not read the contract instance of $id on $network"
check_live "the contract instance" "$tmp/instance.json"
code_hash=$(jq -r '.entries[0].val.contract_data.val.contract_instance.executable.wasm // empty' "$tmp/instance.json")
[[ "$code_hash" =~ ^[0-9a-f]{64}$ ]] || die "the contract instance of $id names no Wasm code"
cli ledger entry fetch contract-code --wasm-hash "$code_hash" --network "$network" --output json \
  > "$tmp/code.json" || die "could not read the Wasm code of $id on $network"
check_live "its Wasm code" "$tmp/code.json"

echo "fetching $id from $network" >&2
if ! cli contract fetch --id "$id" --network "$network" > "$tmp/deployed.wasm" 2> "$tmp/stderr"; then
  if grep -qi 'contract not found' "$tmp/stderr"; then
    not_found "$(jq -r '.latestLedger' "$tmp/code.json")"
  fi
  cat "$tmp/stderr" >&2
  die "could not fetch $id from $network"
fi
deployed_sha=$(sha256 "$tmp/deployed.wasm")
local_sha=$(sha256 out/main.wasm)
[[ "$deployed_sha" == "$recorded_sha" ]] ||
  die "deployed bytes differ: $network holds $deployed_sha for $id, the record says $recorded_sha" 4
[[ "$local_sha" == "$recorded_sha" ]] ||
  die "deployed bytes differ: out/main.wasm is $local_sha, the deployment $recorded_sha (rebuild, or redeploy with scripts/deploy-testnet.sh)" 4
echo "the deployed bytes are out/main.wasm ($deployed_sha)" >&2

echo "generating and funding the throwaway identity amm-check on $network" >&2
cli keys generate amm-check --overwrite >&2
fund amm-check "$network"
status=0
answer=$(cli contract invoke --id "$id" --source amm-check --network "$network" --send=no -- \
  get_amount_out --amount_in 1000 --reserve_in 100000 --reserve_out 100000 --fee_bps 30 \
  2> "$tmp/stderr") || status=$?
if [[ $status -ne 0 ]] && ! grep -q HostError "$tmp/stderr"; then
  cat "$tmp/stderr" >&2
  die "could not invoke $id on $network"
fi
if [[ $status -ne 0 || "$answer" != 987 ]]; then
  cat "$tmp/stderr" >&2
  die "wrong answer: get_amount_out(1000, 100000, 100000, 30) on $id returned '$answer' (exit $status), not 987" 5
fi
echo "ok: $id on $network is out/main.wasm and get_amount_out(1000, 100000, 100000, 30) = 987"
if [[ "$network" == local ]]; then
  echo "the local network is still running; remove it and its deployments with: $(down_command)" >&2
else
  echo "amm-check's key stays in the compose volume stellar-config; $(down_command) removes it" >&2
fi
finish
