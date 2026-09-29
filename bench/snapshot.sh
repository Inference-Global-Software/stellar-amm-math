#!/usr/bin/env bash
# Toolchain cost ledger snapshot for the stellar-amm-math contract.
#
# Treats the contract as a fixed workload for tracking the Inference compiler.
# With the given toolchain it
#   1. rebuilds the test shell and the proof module in a temporary directory
#      and requires them to be byte-identical to the committed
#      tests/shell/out/main.wasm, proofs/main.v and proofs/main.wasm;
#   2. rebuilds out/main.wasm in place, from scratch: this is the contract
#      the row measures, and it is committed together with the row (on any
#      failure the script puts the previous out/main.wasm back);
#   3. measures the workload of tests/src/workload.rs under the in-process
#      Soroban host with the `cost` binary, twice; the two runs must print the
#      same bytes;
#   4. runs the host tests on the three modules (`cargo test --locked
#      --no-fail-fast -- --include-ignored` in tests/), all but the one that
#      requires this row;
#   5. appends one JSON line to bench/history.jsonl and keeps the contract as
#      bench/modules/main-$MODULE.wasm.
# A row therefore says: these sources and manifests build these modules with
# this toolchain, the host tests pass on them, and the contract costs this
# (unless ALLOW_STALE, below). tests/repo.rs requires a row for the committed
# build, so a change to src/, to a manifest, to a module, to the workload or
# to the host needs a new row.
#
# Step 4 includes tests/src/ledger.rs's the_costs_decompose_as_documented,
# which checks the cost model that file and bench/RESULTS.md describe (for
# example, that mem_bytes moves only by 32 per argument). A toolchain that
# changes that model, say by importing host functions or growing memory,
# fails it: update the model in both files in the same change as the row.
#
# A new toolchain changes the shell and the proof module too, so first bump
# infc_version in Inference.toml and tests/shell/Inference.toml (the script
# refuses a pin that is not the toolchain's version: the row hashes both
# manifests, so a pin bumped after the row leaves the row matching no
# committed build), then regenerate them with the same toolchain:
#   (cd tests/shell && "$INFS" build)
#   INFC_PATH=<infc> proofs/generate.sh
#
# Usage (from anywhere):
#   INFS=<infs> INFC_PATH=<infc> [MODULE=<name>] [NOTES=<text>] [DATE=<yyyy-mm-dd>] \
#     bash bench/snapshot.sh
#
#   INFS / INFC_PATH  the toolchain's binaries (required; a path, or a name on
#                     the PATH)
#   TOOLCHAIN_COMMIT  the toolchain's commit, 7 to 40 hex digits (default:
#                     `infc --commit-hash`). A build made outside a git
#                     checkout prints `unknown`, which the script refuses, and
#                     a build from a checkout prints its HEAD even when the
#                     tree is dirty: set it for either
#   MODULE            the name the build is kept under, of letters, digits,
#                     `.`, `_` and `-` (default: the commit); set it when a
#                     row re-measures a changed source under a toolchain that
#                     already has one
#   NOTES             free text stored in the row
#   DATE              the row's date (default: today, UTC)
#   SNAPSHOT_TARGET_DIR  the cargo target directory of the measurement and
#                     the host tests (default: tests/target of this checkout).
#                     An inherited CARGO_TARGET_DIR is not used: a target
#                     directory shared with another checkout could hand the
#                     library's unit tests that checkout's build. The host
#                     tests' log is kept there as snapshot-test.log
#   ALLOW_STALE=1     record the row even though the committed shell or proof
#                     build is not what this toolchain builds, or a manifest
#                     pins another infc_version. The row records the fresh
#                     builds, so tests/repo.rs does not take it as the
#                     committed build's, but step 4 still runs the host tests
#                     on the committed shell and proof build, not on the
#                     fresh ones the row names
#   FORCE=1           record the row even though the ledger has one that is
#                     the same in every field but date and notes
#
# A row that measured the same module under the same workload, soroban-sdk
# and soroban-env-host as a row of the ledger must record the same cost, code
# size and interface: the host meters deterministically, so a difference
# there means the harness measures differently from what the workload's id
# names. The script refuses such a row, FORCE or not; list the measurement
# as a new workload in tests/src/workload.rs instead.
#
# The row's fields:
#   date, notes
#   toolchain_version, toolchain_commit  `infc --version` and the commit
#   infs_version       `infs --version`
#   module             the name the contract is kept under in bench/modules/
#   source_sha256      sha256 of the listing `sha256sum src/*.inf` prints,
#                      one `<sha256>  src/<file>` line per file in C-locale
#                      order: it names the files as well as their bytes
#   manifest_sha256    sha256 of Inference.toml, whose [build] section picks
#                      the target and the mode
#                      (every text file, main.v included, is hashed with its
#                      \r\n line ends read as \n, as tests/repo.rs reads it,
#                      so that a checkout that converts them hashes the same)
#   shell_source_sha256  the same listing hash over tests/shell/Inference.toml
#                      and tests/shell/src/*.inf
#   wasm_sha256        sha256 of out/main.wasm, the deployable contract
#   wasm_size          its size in bytes
#   code_size          the size of its code section, without the custom
#                      sections (names, interface, metadata)
#   shell_wasm_sha256  sha256 of the test shell's build (tests/shell)
#   proof_v_sha256, proof_wasm_sha256  sha256 of the proof build's main.v
#                      and main.wasm
#   interface_sha256   sha256 of out/main.wasm's contractspecv0 section
#   soroban_sdk, soroban_env_host  the host versions tests/Cargo.lock pins
#   workload, workload_sha256  the workload's id and the sha256 of its
#                      description (the cases and how each is measured); rows
#                      compare only on the same workload, soroban-sdk and
#                      soroban-env-host
#   cost               method -> case -> instructions, mem_bytes, wasm_insns
#                      (Wasm fuel, not an instruction count) and
#                      delta_instructions (instructions less the
#                      minimum_liquidity() baseline); see tests/src/ledger.rs
set -euo pipefail
unset CDPATH

die() {
  echo "bench/snapshot.sh: $*" >&2
  exit 1
}

warn() {
  echo "bench/snapshot.sh: warning: $*" >&2
}

# A binary as an absolute path: a path is resolved against the directory the
# script was started in, a bare name is looked up on the PATH.
absolute() {
  case $1 in
    */*) (cd "$(dirname "$1")" 2> /dev/null && printf '%s/%s\n' "$(pwd)" "$(basename "$1")") ;;
    *) command -v "$1" ;;
  esac
}

: "${INFS:?set INFS to the infs binary}"
: "${INFC_PATH:?set INFC_PATH to the infc binary}"
given=$INFS
INFS=$(absolute "$given") || die "cannot find INFS=$given"
given=$INFC_PATH
INFC_PATH=$(absolute "$given") || die "cannot find INFC_PATH=$given"
[ -x "$INFS" ] || die "INFS=$INFS is not an executable"
[ -x "$INFC_PATH" ] || die "INFC_PATH=$INFC_PATH is not an executable"
for tool in jq cargo cmp awk sed sort perl; do
  command -v "$tool" > /dev/null || die "needs $tool on the PATH"
done
command -v sha256sum > /dev/null || command -v shasum > /dev/null || die "needs sha256sum or shasum"
export INFC_PATH
TARGET_DIR=${SNAPSHOT_TARGET_DIR:-}
case $TARGET_DIR in
  "" | /*) ;;
  *) TARGET_DIR="$PWD/$TARGET_DIR" ;;
esac

cd "$(dirname "$0")/.."
export CARGO_TARGET_DIR="${TARGET_DIR:-$PWD/tests/target}"
TEST_LOG="$CARGO_TARGET_DIR/snapshot-test.log"
LEDGER=bench/history.jsonl
VERSION=$("$INFC_PATH" --version | awk '{print $2}') || die "$INFC_PATH --version failed"
COMMIT=${TOOLCHAIN_COMMIT:-$("$INFC_PATH" --commit-hash)} || die "$INFC_PATH --commit-hash failed"
INFS_VERSION=$("$INFS" --version | awk '{print $2}') || die "$INFS --version failed"
[ -n "$VERSION" ] && [ -n "$COMMIT" ] && [ -n "$INFS_VERSION" ] || die "cannot read the toolchain's versions"
[ "$COMMIT" != unknown ] || die "$INFC_PATH does not know its commit (it prints unknown); set TOOLCHAIN_COMMIT"
[[ "$COMMIT" =~ ^[0-9a-f]{7,40}$ ]] || die "the toolchain's commit $COMMIT is not 7 to 40 hex digits"
MODULE=${MODULE:-$COMMIT}
NOTES=${NOTES:-}
DATE=${DATE:-$(date -u +%F)}
[[ "$MODULE" =~ ^[A-Za-z0-9._-]+$ ]] || die "MODULE=$MODULE: use letters, digits, '.', '_' and '-'"
[[ "$DATE" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || die "DATE=$DATE is not yyyy-mm-dd"
for manifest in Inference.toml tests/shell/Inference.toml; do
  pinned=$(sed -n 's/^infc_version *= *"\([^"]*\)".*$/\1/p' "$manifest")
  if [ "$pinned" != "$VERSION" ]; then
    stale_pin="$manifest pins infc_version ${pinned:-nothing}, but $INFC_PATH is $VERSION"
    if [ "${ALLOW_STALE:-}" = 1 ]; then
      warn "$stale_pin; recording anyway (ALLOW_STALE=1)"
    else
      die "$stale_pin: bump infc_version in $manifest to $VERSION before recording, because the row hashes
the manifest and a pin bumped afterwards leaves it matching no committed build (or set ALLOW_STALE=1)"
    fi
  fi
done

# The ledger, when it has rows, must be JSON lines: one object per line, each
# line ended. Checked first, so that a damaged ledger costs no build.
if [ -s "$LEDGER" ]; then
  jq -e -R -s 'endswith("\n")
    and (rtrimstr("\n") | split("\n") | all(.[]; ((fromjson? | type) // "not JSON") == "object"))' \
    "$LEDGER" > /dev/null || die "$LEDGER is not JSON lines, one object per line; repair it first"
fi

TMP=$(mktemp -d)
DONE=0
# On any failure after out/main.wasm is rebuilt, put the previous one back.
cleanup() {
  if [ "$DONE" != 1 ] && [ -e "$TMP/committed.wasm" ] && ! cmp -s "$TMP/committed.wasm" out/main.wasm; then
    cp "$TMP/committed.wasm" out/main.wasm
    echo "bench/snapshot.sh: restored the previous out/main.wasm" >&2
  fi
  rm -rf "$TMP"
}
trap cleanup EXIT

sha256() {
  if command -v sha256sum > /dev/null; then sha256sum "$@"; else shasum -a 256 "$@"; fi | awk '{print $1}'
}

# The sha256 of a text file with its \r\n line ends read as \n, as the tests
# read it.
text_sha256() {
  [ -r "$1" ] || { echo "bench/snapshot.sh: cannot read $1" >&2; return 1; }
  perl -pe 's/\r\n/\n/' < "$1" | sha256
}

# The sha256 of the listing `sha256sum` prints for the given text files, in
# C-locale order, one `<sha256>  <path>` line each (text_sha256 of each).
# Fails when a file cannot be read, such as a glob that matched nothing.
listing_sha256() {
  local file hash
  printf '%s\n' "$@" | LC_ALL=C sort | while IFS= read -r file; do
    hash=$(text_sha256 "$file") || exit 1
    printf '%s  %s\n' "$hash" "$file"
  done | sha256
}

# The version tests/Cargo.lock pins for a package, which it must list once.
lock_version() {
  local versions count
  versions=$(awk -v name="$1" '
    { sub(/\r$/, "") }
    listed { if (sub(/^version = "/, "") && sub(/"$/, "")) print; listed = 0 }
    $0 == "name = \"" name "\"" { listed = 1 }
  ' tests/Cargo.lock)
  count=$(printf '%s' "$versions" | awk 'END { print NR }')
  [ "$count" = 1 ] || die "tests/Cargo.lock lists $1 $count times, not once"
  echo "$versions"
}
SOROBAN_SDK=$(lock_version soroban-sdk)
SOROBAN_ENV_HOST=$(lock_version soroban-env-host)

# ---- the shell and the proof module, in a temporary directory -----------
mkdir -p "$TMP/shell"
cp -R tests/shell/Inference.toml tests/shell/src "$TMP/shell/"
(cd "$TMP/shell" && "$INFS" build) > "$TMP/shell.log" 2>&1 || { cat "$TMP/shell.log"; exit 1; }
SHELL_SHA=$(sha256 "$TMP/shell/out/main.wasm")

"$INFC_PATH" src/main.inf --target wasm32 -v --out-dir "$TMP/proof" > "$TMP/proof.log" 2>&1 \
  || { cat "$TMP/proof.log"; exit 1; }
PROOF_V_SHA=$(text_sha256 "$TMP/proof/main.v")
PROOF_WASM_SHA=$(sha256 "$TMP/proof/main.wasm")

STALE=0
# Reports whether the committed file $3 is the fresh build $2: the same
# bytes, or for text the same text (text_sha256); $1 is `bytes` or `text`.
check_fresh() {
  local same
  if [ "$1" = text ]; then
    [ "$(text_sha256 "$2")" = "$(text_sha256 "$3")" ] && same=1 || same=0
  else
    cmp -s "$2" "$3" && same=1 || same=0
  fi
  if [ "$same" = 1 ]; then
    printf '%-26s same as the fresh build\n' "$3:"
  else
    printf '%-26s DIFFERS from the fresh build\n' "$3:"
    STALE=1
  fi
}
check_fresh bytes "$TMP/shell/out/main.wasm" tests/shell/out/main.wasm
check_fresh text "$TMP/proof/main.v" proofs/main.v
check_fresh bytes "$TMP/proof/main.wasm" proofs/main.wasm
if [ "$STALE" = 1 ] && [ "${ALLOW_STALE:-}" != 1 ]; then
  die "the committed shell or proof build is not what this toolchain builds; regenerate them first,
  (cd tests/shell && $INFS build)
  INFC_PATH=$INFC_PATH proofs/generate.sh
or set ALLOW_STALE=1 to record the row anyway"
fi

# ---- the contract, rebuilt in place -------------------------------------
# out/main.wasm is removed first, so that everything below can only see this
# build's output.
if [ -e out/main.wasm ]; then cp out/main.wasm "$TMP/committed.wasm"; fi
rm -f out/main.wasm
"$INFS" build > "$TMP/build.log" 2>&1 || { cat "$TMP/build.log"; exit 1; }
WASM_SHA=$(sha256 out/main.wasm)
WASM_SIZE=$(wc -c < out/main.wasm | tr -d ' ')
SRC_SHA=$(listing_sha256 src/*.inf)
SHELL_SRC_SHA=$(listing_sha256 tests/shell/Inference.toml tests/shell/src/*.inf)
MANIFEST_SHA=$(text_sha256 Inference.toml)
if [ ! -e "$TMP/committed.wasm" ]; then
  printf '%-26s built; there was none before\n' "out/main.wasm:"
elif cmp -s out/main.wasm "$TMP/committed.wasm"; then
  printf '%-26s same as before the build\n' "out/main.wasm:"
else
  printf '%-26s changed by the build: commit it with the row\n' "out/main.wasm:"
fi

mkdir -p bench/modules
if [ -e "bench/modules/main-$MODULE.wasm" ] && ! cmp -s out/main.wasm "bench/modules/main-$MODULE.wasm"; then
  die "bench/modules/main-$MODULE.wasm already holds a different module; set MODULE to a new name"
fi

# ---- measure ----------------------------------------------------------
for run in 1 2; do
  (cd tests && cargo run --locked --release --quiet --bin cost) > "$TMP/cost$run.json" 2> "$TMP/cost$run.err" \
    || { cat "$TMP/cost$run.err" >&2; exit 1; }
done
if ! cmp -s "$TMP/cost1.json" "$TMP/cost2.json"; then
  echo "two runs of the cost binary measured different costs:" >&2
  diff "$TMP/cost1.json" "$TMP/cost2.json" >&2 || true
  exit 1
fi
MEASURED_SHA=$(jq -r .wasm_sha256 "$TMP/cost1.json")
[ "$MEASURED_SHA" = "$WASM_SHA" ] || die "the cost binary measured $MEASURED_SHA, not this build ($WASM_SHA)"

# ---- the row ----------------------------------------------------------
jq -c -n \
  --arg date "$DATE" \
  --arg toolchain_version "$VERSION" \
  --arg toolchain_commit "$COMMIT" \
  --arg infs_version "$INFS_VERSION" \
  --arg module "$MODULE" \
  --arg source_sha256 "$SRC_SHA" \
  --arg manifest_sha256 "$MANIFEST_SHA" \
  --arg shell_source_sha256 "$SHELL_SRC_SHA" \
  --arg wasm_sha256 "$WASM_SHA" \
  --argjson wasm_size "$WASM_SIZE" \
  --arg shell_wasm_sha256 "$SHELL_SHA" \
  --arg proof_v_sha256 "$PROOF_V_SHA" \
  --arg proof_wasm_sha256 "$PROOF_WASM_SHA" \
  --arg soroban_sdk "$SOROBAN_SDK" \
  --arg soroban_env_host "$SOROBAN_ENV_HOST" \
  --slurpfile measured "$TMP/cost1.json" \
  --arg notes "$NOTES" \
  '{date: $date, toolchain_version: $toolchain_version, toolchain_commit: $toolchain_commit,
    infs_version: $infs_version, module: $module, source_sha256: $source_sha256,
    manifest_sha256: $manifest_sha256, shell_source_sha256: $shell_source_sha256,
    wasm_sha256: $wasm_sha256, wasm_size: $wasm_size, code_size: $measured[0].code_size,
    shell_wasm_sha256: $shell_wasm_sha256, proof_v_sha256: $proof_v_sha256,
    proof_wasm_sha256: $proof_wasm_sha256, interface_sha256: $measured[0].interface_sha256,
    soroban_sdk: $soroban_sdk, soroban_env_host: $soroban_env_host,
    workload: $measured[0].workload, workload_sha256: $measured[0].workload_sha256,
    cost: $measured[0].cost, notes: $notes}' \
  > "$TMP/row.json"

# What a row's measurement is a function of, and what it measures: a row that
# agrees with an earlier one on the first must agree on the second. These are
# tests/repo.rs's two lists, and it checks these two lines against them.
MEASURED_UNDER='["wasm_sha256", "workload_sha256", "soroban_sdk", "soroban_env_host"]'
MEASURED='["cost", "code_size", "interface_sha256"]'
if [ -s "$LEDGER" ]; then
  contradicts=0
  jq -e -s --slurpfile row "$TMP/row.json" --argjson under "$MEASURED_UNDER" --argjson measured "$MEASURED" \
    '$row[0] as $new
    | any(.[]; . as $old | ($under | all($old[.] == $new[.])) and ($measured | any($old[.] != $new[.])))' \
    "$LEDGER" > /dev/null || contradicts=$?
  case $contradicts in
    0) die "$LEDGER has a row for this module, workload $(jq -r .workload "$TMP/row.json"), soroban-sdk
$SOROBAN_SDK and soroban-env-host $SOROBAN_ENV_HOST with another cost, code size or interface: the harness
measures differently from what the workload's id names; append the measurement to workload::WORKLOADS in
tests/src/workload.rs under a new id" ;;
    1) ;;
    *) die "cannot read $LEDGER (jq exit $contradicts)" ;;
  esac
fi

if [ -s "$LEDGER" ] && [ "${FORCE:-}" != 1 ]; then
  duplicate=0
  jq -e -s --slurpfile row "$TMP/row.json" 'any(.[]; del(.date, .notes) == ($row[0] | del(.date, .notes)))' \
    "$LEDGER" > /dev/null || duplicate=$?
  case $duplicate in
    0) die "$LEDGER already has this row, the same in every field but date and notes; set FORCE=1 to add it again" ;;
    1) ;;
    *) die "cannot read $LEDGER (jq exit $duplicate)" ;;
  esac
fi

# ---- the host tests on this build ---------------------------------------
mkdir -p "$CARGO_TARGET_DIR"
if ! (cd tests && cargo test --locked --no-fail-fast -- --include-ignored \
  --skip the_ledger_records_the_committed_build) > "$TEST_LOG" 2>&1; then
  # Each failing target's `failures:` blocks: the panic messages, then the
  # names of the tests that failed.
  awk '/^failures:$/ { shown = 1 } shown; /^test result:/ { shown = 0 }' "$TEST_LOG" >&2
  tail -n 20 "$TEST_LOG" >&2
  die "the host tests fail on this build; no row recorded (the whole log: $TEST_LOG)"
fi
awk '/^test result: ok\./ { passed += $4 } END { printf "%-26s %d passed\n", "host tests:", passed }' "$TEST_LOG"

# ---- append -----------------------------------------------------------
cp out/main.wasm "bench/modules/main-$MODULE.wasm"
cat "$TMP/row.json" >> "$LEDGER"
DONE=1
echo "appended to $LEDGER:"
jq . "$TMP/row.json"
