#!/usr/bin/env bash
# Deploys out/main.wasm to Stellar testnet (or to the local network) and
# writes a record of the deployment.
#
# Usage: scripts/deploy-testnet.sh [--network testnet|local] [--out FILE]
#   --network  testnet (the default), or local: the network compose.yaml
#              defines, started unless it is already running (it is then
#              used as it is), and left running afterwards. A stopped one is
#              started again as it is, with what it holds
#   --out      the record to write (default: deployments/testnet.json on
#              testnet, and .ctx/deployments/local.json on the local
#              network: under .ctx/, which git ignores, so that a record of
#              a local deployment can never be committed)
#
# It first asks the network for its passphrase, and stops unless it is that
# network's. It then generates a fresh deployer identity, amm-deployer, in
# the compose volume stellar-config and funds it with friendbot; deploys
# out/main.wasm; fetches the contract back and requires the bytes to be
# out/main.wasm; extends the time to live of the contract instance and of its
# Wasm code to the longest the network allows (about 180 days on testnet;
# without it, both are archived after the minimum, about 7 days); invokes
# get_amount_out(1000, 100000, 100000, 30) and requires 987. Only then does it
# write the record, a JSON object with
#   network, contract_id, wasm_sha256, toolchain_version, toolchain_commit,
#   deployed_at (UTC), deployer (the deployer's public key),
#   live_until_ledger (the last ledger both entries live to, unless extended)
# The toolchain fields name the toolchain that built these bytes: infc
# (INFC_PATH, default infc on the PATH) when it rebuilds out/main.wasm from
# src/ byte for byte; otherwise the last row of bench/history.jsonl whose
# wasm_sha256 is out/main.wasm's; otherwise they are null. The contract holds
# no state and has no admin, so the deployer identity has no further use.
# scripts/check-testnet.sh checks the record later.
#
# The contract id is the only line on stdout; progress goes to stderr.
# Needs Docker with Compose v2, and jq. The CLI reads out/main.wasm through a
# bind mount, so the Docker daemon must see this checkout at its own path:
# not a remote daemon, nor docker-outside-of-docker from a container whose
# workspace path differs.
# Bash only: under another shell (dash's sh, zsh) stop with a clear message.
[ -n "${BASH_VERSION:-}" ] || { echo "error: $0 needs bash: run it with bash '$0'" >&2; exit 2; }
set -euo pipefail
unset CDPATH
caller=$PWD
infc=${INFC_PATH:-infc}
if [[ "$infc" == */* && "$infc" != /* ]]; then
  infc=$caller/$infc
fi
cd "$(dirname "$0")/.."
. scripts/lib.sh

usage() { die "usage: scripts/deploy-testnet.sh [--network testnet|local] [--out FILE]" 2; }

network=testnet out=
while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) [[ $# -ge 2 ]] || usage; network=$2; shift 2 ;;
    --out) [[ $# -ge 2 ]] || usage; out=$2; shift 2 ;;
    *) usage ;;
  esac
done
case "$network" in
  testnet) service=cli-net default_out=deployments/testnet.json ;;
  local) service=cli default_out=.ctx/deployments/local.json ;;
  *) usage ;;
esac
if [[ -z "$out" ]]; then
  out=$default_out
elif [[ "$out" != /* ]]; then
  out=$caller/$out
fi
[[ ! -d "$out" ]] || die "--out names a directory, not a file: $out" 2
need_compose
command -v jq >/dev/null 2>&1 || die "this needs jq" 2
[[ -f out/main.wasm ]] || die "no out/main.wasm; build it with infs build" 2

# The CLI container. On the local network, once start_network has it
# healthy, --no-deps keeps each call from converging the stellar service
# again, which could recreate it, and the deployment with it.
no_deps=
cli() { "${DC[@]}" run --rm -T ${no_deps:+--no-deps} "$service" "$@" </dev/null; }
tmp=$(mktemp -d)
remove_tmp() { rm -rf "$tmp"; }
at_exit remove_tmp

wasm_sha256=$(sha256 out/main.wasm)
version='' commit=''
if "$infc" --version >/dev/null 2>&1; then
  if "$infc" src/main.inf --target stellar --out-dir "$tmp/rebuild" > "$tmp/rebuild.log" 2>&1 &&
    cmp -s out/main.wasm "$tmp/rebuild/main.wasm"; then
    version=$("$infc" --version | awk '{print $2}')
    commit=$("$infc" --commit-hash 2>/dev/null || true)
    echo "toolchain: infc $version ($commit) rebuilds out/main.wasm from src/" >&2
  else
    echo "warning: $infc does not rebuild out/main.wasm from src/, so the record will not name it" >&2
  fi
fi
if [[ -z "$version" && -f bench/history.jsonl ]]; then
  row=$(jq -c --arg sha "$wasm_sha256" 'select(.wasm_sha256 == $sha)' bench/history.jsonl) ||
    die "bench/history.jsonl is not JSON lines"
  row=$(tail -n 1 <<< "$row")
  if [[ -n "$row" ]]; then
    version=$(jq -r '.toolchain_version // empty' <<< "$row")
    commit=$(jq -r '.toolchain_commit // empty' <<< "$row")
    echo "toolchain: infc $version ($commit), from the bench/history.jsonl row for out/main.wasm" >&2
  fi
fi
if [[ -z "$version" ]]; then
  echo "warning: no toolchain is known to build out/main.wasm ($wasm_sha256); the record says null" >&2
elif [[ "$version" != "$(ledger_version)" ]]; then
  echo "warning: Inference.toml pins infc $(ledger_version), but out/main.wasm comes from infc $version" >&2
fi

if [[ "$network" == local ]]; then
  # Whatever happens next, say how to remove the network, or what is left of
  # it (a container that was created but never started, say).
  local_hint() {
    local state
    state=$(network_state 2>/dev/null) || return 0
    case "$state" in
      running) echo "the local network is still running; remove it and its deployments with: $(down_command)" >&2 ;;
      stopped) echo "the local network's containers remain; remove them and their deployments with: $(down_command)" >&2 ;;
    esac
  }
  at_exit local_hint
  start_network
  no_deps=1
fi
check_network_identity "$network"

echo "generating and funding the deployer identity amm-deployer on $network" >&2
cli keys generate amm-deployer --overwrite >&2
fund amm-deployer "$network"
deployer=$(cli keys address amm-deployer)
[[ "$deployer" =~ ^G[A-Z2-7]{55}$ ]] || die "keys address printed no public key: '$deployer'"

echo "deploying out/main.wasm ($wasm_sha256)" >&2
id=$(cli contract deploy --wasm out/main.wasm --source amm-deployer --network "$network")
[[ "$id" =~ ^C[A-Z2-7]{55}$ ]] || die "deploy printed no contract id: '$id'"
deployed_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "deployed $id" >&2

cli contract fetch --id "$id" --network "$network" > "$tmp/deployed.wasm" ||
  die "could not fetch $id back from $network; no record written"
cmp -s out/main.wasm "$tmp/deployed.wasm" ||
  die "the network holds $(sha256 "$tmp/deployed.wasm") for $id, not out/main.wasm; no record written"

# The longest extension the network accepts is one ledger short of its
# max_entry_ttl.
cli network settings --network "$network" --output json > "$tmp/settings.json" ||
  die "could not read $network's settings; no record written"
max_ttl=$(jq -r '[.. | objects | select(has("max_entry_ttl")) | .max_entry_ttl] | first // empty' "$tmp/settings.json")
[[ "$max_ttl" =~ ^[0-9]+$ && $max_ttl -gt 1 ]] ||
  die "$network's settings name no max_entry_ttl; no record written"
ledgers=$((max_ttl - 1))
echo "extending the contract instance and its Wasm code by $ledgers ledgers" >&2
instance_until=$(cli contract extend --id "$id" --ledgers-to-extend "$ledgers" --durability persistent \
  --ttl-ledger-only --source amm-deployer --network "$network") ||
  die "could not extend the contract instance of $id; no record written"
code_until=$(cli contract extend --wasm out/main.wasm --ledgers-to-extend "$ledgers" --durability persistent \
  --ttl-ledger-only --source amm-deployer --network "$network") ||
  die "could not extend the Wasm code of $id; no record written"
[[ "$instance_until" =~ ^[0-9]+$ && "$code_until" =~ ^[0-9]+$ ]] ||
  die "contract extend printed no ledger: '$instance_until', '$code_until'; no record written"
live_until=$((instance_until < code_until ? instance_until : code_until))
echo "the contract instance lives until ledger $instance_until, its Wasm code until $code_until" >&2

answer=$(cli contract invoke --id "$id" --source amm-deployer --network "$network" --send=no -- \
  get_amount_out --amount_in 1000 --reserve_in 100000 --reserve_out 100000 --fee_bps 30) ||
  die "get_amount_out(1000, 100000, 100000, 30) on $id failed; no record written"
[[ "$answer" == 987 ]] ||
  die "get_amount_out(1000, 100000, 100000, 30) on $id returned '$answer', not 987; no record written"
echo "get_amount_out(1000, 100000, 100000, 30) = 987" >&2

mkdir -p "$(dirname "$out")"
jq -n \
  --arg network "$network" \
  --arg contract_id "$id" \
  --arg wasm_sha256 "$wasm_sha256" \
  --arg toolchain_version "$version" \
  --arg toolchain_commit "$commit" \
  --arg deployed_at "$deployed_at" \
  --arg deployer "$deployer" \
  --argjson live_until_ledger "$live_until" \
  '{network: $network, contract_id: $contract_id, wasm_sha256: $wasm_sha256,
    toolchain_version: ($toolchain_version | if . == "" then null else . end),
    toolchain_commit: ($toolchain_commit | if . == "" then null else . end),
    deployed_at: $deployed_at, deployer: $deployer,
    live_until_ledger: $live_until_ledger}' > "$tmp/record.json"
mv "$tmp/record.json" "$out"
echo "wrote $out" >&2
if [[ "$network" != local ]]; then
  echo "amm-deployer's key stays in the compose volume stellar-config; $(down_command) removes it" >&2
fi
echo "$id"
finish
