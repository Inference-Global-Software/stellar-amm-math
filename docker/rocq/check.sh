#!/usr/bin/env bash
# rocq-check --stdin NAME.v   type-check the file on stdin, as NAME.v, against
#                             this image's library (ROCQ_TIER: stub or real)
# rocq-check --self-test      prove which library this image carries
#
# The file is checked in a private temp dir, since coqc writes .vo and .glob
# beside its input. A pass prints `ok   NAME.v [TIER] sha256=DIGEST` on
# stdout, with the digest of what arrived, so that the caller can tell that
# the whole file was checked; nothing else goes to stdout. coqc's own output
# goes to stderr, except on the real tier, where it can quote the library,
# which is not public: there a pass shows none of it, and a failure only
# coqc's final error, or nothing if coqc stopped in any other way than with
# its error status, 1 (killed, say). The file decides what that error
# quotes, so keep the real tier's output private (see proofs/check.sh).
# Bash only: under another shell (dash's sh, zsh) stop with a clear message.
[ -n "${BASH_VERSION:-}" ] || { echo "error: $0 needs bash: run it with bash '$0'" >&2; exit 2; }
set -euo pipefail

case "${ROCQ_TIER:-}" in
  stub) flags=(-Q "$HOME/rocq-stub/wasm" Wasm -Q "$HOME/rocq-stub/wasm_verifier" WasmVerifier) ;;
  # coq-wasm (logical root Wasm) is an installed opam library, which coqc
  # finds on its own load path; only WasmVerifier is mapped, to the dune build.
  real) flags=(-R "$HOME/wasm-verifier/_build/default/theories" WasmVerifier) ;;
  *) echo "rocq-check: ROCQ_TIER must be stub or real" >&2; exit 2 ;;
esac

usage() { echo "usage: rocq-check --stdin NAME.v | --self-test" >&2; exit 2; }

# coq DIR NAME: type-check DIR/NAME, with DIR as the working directory, and
# show coqc's output on stderr as far as this tier allows (see above).
coq() {
  local log rc=0
  if [[ "$ROCQ_TIER" == stub ]]; then
    (cd "$1" && coqc "${flags[@]}" "$2") >&2
    return
  fi
  log=$(mktemp)
  (cd "$1" && coqc "${flags[@]}" "$2") > "$log" 2>&1 || rc=$?
  if [[ $rc -eq 1 ]]; then
    # From the last line that starts with "Error" (or the "File ..." line
    # just before it) to the end.
    awk '{ line[NR] = $0 } /^Error/ { err = NR }
      END { if (!err) exit; if (err > 1 && line[err - 1] ~ /^File "/) err--
            for (i = err; i <= NR; i++) print line[i] }' "$log" >&2
  elif [[ $rc -ne 0 ]]; then
    echo "rocq-check: coqc stopped on $2 with status $rc; on the real tier, its output is not shown" >&2
  fi
  rm -f "$log"
  return "$rc"
}

case "${1:-}" in
  --self-test)
    [[ $# -eq 1 ]] || usage
    # HA_pto and ktrue are declared by the real WasmVerifier.Assertions and
    # not by the stub. Each is probed in a file of its own, since coqc stops
    # at its first error, so that on the stub both are shown to be missing.
    t=$(mktemp -d)
    seen=
    for name in HA_pto ktrue; do
      printf 'From WasmVerifier Require Import Assertions.\nCheck %s.\n' "$name" > "$t/probe_$name.v"
      if out=$(coq "$t" "probe_$name.v" 2>&1); then
        found=real
        echo "self-test: the library declares $name" >&2
      elif [[ "$(tr -s ' \n' '  ' <<< "$out")" == *"The reference $name was not found"* ]]; then
        found=stub
        echo "self-test: the library does not declare $name" >&2
      else
        echo "$out" >&2
        echo "self-test: coqc neither accepted $name nor reported it missing" >&2
        exit 1
      fi
      [[ -z "$seen" || "$seen" == "$found" ]] ||
        { echo "self-test: the library behaves as neither the stub nor the real one" >&2; exit 1; }
      seen=$found
    done
    rm -rf "$t"
    [[ "$seen" == "$ROCQ_TIER" ]] ||
      { echo "self-test: the image claims to carry the $ROCQ_TIER library, but carries the $seen one" >&2; exit 1; }
    echo "self-test: ok ($ROCQ_TIER)"
    ;;
  --stdin)
    [[ $# -eq 2 && "$2" =~ ^[A-Za-z_][A-Za-z0-9_]*\.v$ ]] || usage
    work=$(mktemp -d)
    cat > "$work/$2"
    [[ -s "$work/$2" ]] || { echo "rocq-check: nothing arrived on stdin for $2" >&2; exit 2; }
    sha=$(sha256sum "$work/$2" | cut -d' ' -f1)
    rc=0
    coq "$work" "$2" || rc=$?
    rm -rf "$work"
    if [[ $rc -ne 0 ]]; then
      echo "FAIL $2 [$ROCQ_TIER]" >&2
      exit 1
    fi
    echo "ok   $2 [$ROCQ_TIER] sha256=$sha"
    ;;
  *) usage ;;
esac
