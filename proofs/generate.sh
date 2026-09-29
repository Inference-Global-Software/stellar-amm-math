#!/usr/bin/env bash
# Rebuilds proofs/main.v and proofs/main.wasm, the proof artifacts, from src/.
#
# Usage: proofs/generate.sh [--check]     (from anywhere)
#   --check    write nothing: only compare the fresh build with proofs/main.v
#              and proofs/main.wasm, and exit with status 1, naming each file
#              that differs, unless both are byte for byte what src/ builds
#   INFC_PATH  the infc binary to use (default: infc on the PATH); a relative
#              path is taken from the directory this is run from
#
# The stellar target refuses proof mode, so infc builds the proof for the
# wasm32 target. It writes into a fresh temp dir, and both files must be
# there before they replace proofs/main.v and proofs/main.wasm; nothing else
# is touched, out/main.wasm (the contract) included. With the toolchain
# Inference.toml pins, the files come out byte for byte as committed.
# stdout is the files' SHA-256s: the new files', or with --check the fresh
# build's.
# Bash only: under another shell (dash's sh, zsh) stop with a clear message.
[ -n "${BASH_VERSION:-}" ] || { echo "error: $0 needs bash: run it with bash '$0'" >&2; exit 2; }
set -euo pipefail
unset CDPATH
infc=${INFC_PATH:-infc}
if [[ "$infc" == */* && "$infc" != /* ]]; then
  infc=$PWD/$infc
fi
cd "$(dirname "$0")/.."
. scripts/lib.sh

check=0
case "$*" in
  '') ;;
  --check) check=1 ;;
  *) die "usage: proofs/generate.sh [--check]" 2 ;;
esac

if ! command -v "$infc" >/dev/null 2>&1; then
  [[ -z "${INFC_PATH:-}" ]] ||
    die "INFC_PATH=$INFC_PATH is not an executable infc" 2
  die "infc not found on the PATH: install the toolchain (infs install) or set INFC_PATH" 2
fi
version=$("$infc" --version | awk '{print $2}')
pinned=$(ledger_version)
echo "infc $version" >&2
[[ "$version" == "$pinned" ]] ||
  echo "warning: Inference.toml pins infc $pinned; infc $version may emit different files" >&2

tmp=$(mktemp -d)
remove_tmp() { rm -rf "$tmp"; }
at_exit remove_tmp
"$infc" src/main.inf --target wasm32 -v --out-dir "$tmp" >&2
for f in main.v main.wasm; do
  [[ -s "$tmp/$f" ]] || die "infc wrote no $f; proofs/ is unchanged"
done

if [[ $check -eq 1 ]]; then
  stale=0
  for f in main.v main.wasm; do
    echo "$(sha256 "$tmp/$f")  proofs/$f"
    if ! cmp -s "$tmp/$f" "proofs/$f"; then
      echo "error: proofs/$f is not what src/ builds with infc $version; run proofs/generate.sh" >&2
      stale=1
    fi
  done
  [[ $stale -eq 0 ]] || exit 1
  echo "proofs/main.v and proofs/main.wasm are what src/ builds with infc $version" >&2
  finish
  exit 0
fi

cp "$tmp/main.v" "$tmp/main.wasm" proofs/
for f in proofs/main.v proofs/main.wasm; do
  digest=$(sha256 "$f")
  echo "$digest  $f"
done
finish
