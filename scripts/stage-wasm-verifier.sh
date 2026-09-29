#!/usr/bin/env bash
# Stages the wasm-verifier library at the revision the toolchain pins, where
# proofs/check.sh real builds it into its image: in .ctx/wasm-verifier, or in
# DEST.
#
# Usage: scripts/stage-wasm-verifier.sh CHECKOUT [DEST]
#   CHECKOUT  a git clone of wasm-verifier (or any directory inside one) that
#             contains the pinned revision
#   DEST      the directory to stage it in (default: .ctx/wasm-verifier in
#             this checkout; a relative path is taken from the directory this
#             is run from). It must not exist yet, or be a library this
#             script staged (with DEST.sha256 beside it); inside a git work
#             tree, git must ignore it. Run the real tier with
#             WASM_VERIFIER_CONTEXT=DEST then.
#
# The revision is the one docker/rocq/wasm-verifier-TAG.rev records for the
# toolchain Inference.toml pins (TAG = v + its infc_version). Only what the
# library's build reads is staged: its dune-project and theories/. `git
# archive` reads the clone's object store only: its working tree, index and
# branches are left as they are, and nothing is fetched. If the revision is
# missing, fetch it into the clone yourself and run this again.
#
# The staged files replace the old ones in one step. Then DEST.sha256 records
# their digests (proofs/check.sh real refuses a staged tree that no longer
# matches them), and DEST.rev names the revision, only once both are
# complete. They are private to you: directories 0700, files 0600.
#
# The library is not public, and the staged copy survives branch switches:
# git ignores .ctx/, so no checkout or `gh pr checkout` removes it, and any
# other branch's code run in this checkout (its scripts, compose.yaml,
# Dockerfile or tests) finds it where this script puts it. Remove it before
# running another branch's code here,
#   rm -rf .ctx/wasm-verifier .ctx/wasm-verifier.sha256 .ctx/wasm-verifier.rev
# or stage it outside any checkout, in a DEST of your own.
# Bash only: under another shell (dash's sh, zsh) stop with a clear message.
[ -n "${BASH_VERSION:-}" ] || { echo "error: $0 needs bash: run it with bash '$0'" >&2; exit 2; }
set -euo pipefail
unset CDPATH
umask 077

[[ $# -eq 1 || $# -eq 2 ]] || { echo "usage: scripts/stage-wasm-verifier.sh CHECKOUT [DEST]" >&2; exit 2; }
[[ -d "$1" ]] || { echo "error: no such directory: $1" >&2; exit 2; }
# The clone's git directory, so that `git archive` below starts from the top
# of the tree even when CHECKOUT is a subdirectory.
gitdir=$(git -C "$1" rev-parse --absolute-git-dir 2>/dev/null) ||
  { echo "error: $1 is not in a git clone" >&2; exit 2; }
dest=${2:-}
if [[ -n "$dest" && "$dest" != /* ]]; then
  dest=$PWD/$dest
fi

cd "$(dirname "$0")/.."
. scripts/lib.sh

dest=${dest:-$PWD/.ctx/wasm-verifier}
while [[ "$dest" == */ && "$dest" != / ]]; do dest=${dest%/}; done
[[ "$dest" != / ]] || die "DEST cannot be /" 2
# DEST is removed and replaced below, so it must not exist, or be a library
# this script staged: a directory with DEST.sha256 beside it, which only
# this script writes, and nothing in it but dune-project and theories/.
if [[ -e "$dest" || -L "$dest" ]]; then
  [[ -d "$dest" && ! -L "$dest" && -f "$dest.sha256" ]] ||
    die "$dest exists, and this script did not stage it; name a new directory" 2
  odd=$(cd "$dest" && find . -mindepth 1 -maxdepth 1 ! -name dune-project ! -name theories)
  [[ -z "$odd" ]] ||
    die "$dest holds more than a staged library (dune-project and theories/); name a new directory" 2
fi
# Inside a git work tree, git must ignore the library and its two records,
# or a commit could publish them. git is asked from DEST's nearest existing
# ancestor, so that nothing is created before it answers.
parent=$(dirname "$dest")
while [[ ! -d "$parent" ]]; do parent=$(dirname "$parent"); done
if top=$(git -C "$parent" rev-parse --show-toplevel 2>/dev/null); then
  for f in "$dest" "$dest.sha256" "$dest.rev"; do
    git -C "$parent" check-ignore -q "$f" ||
      die "$f is inside the git work tree $top, which does not ignore it; stage it where git ignores it, or outside any checkout" 2
  done
fi

tag=v$(ledger_version)
revfile=docker/rocq/wasm-verifier-$tag.rev
[[ -f "$revfile" ]] || die "no $revfile: the wasm-verifier revision of $tag is not recorded" 2
rev=$(sed '/^#/d' "$revfile" | tr -d '[:space:]')
[[ "$rev" =~ ^[0-9a-f]{40}$ ]] || die "$revfile holds no revision" 2

git --git-dir="$gitdir" cat-file -e "$rev^{commit}" 2>/dev/null ||
  die "$1 does not contain revision $rev; fetch it (git -C '$1' fetch origin) and run this again" 2

# Without DEST.rev, a staging that stops halfway is never taken for a
# complete one; DEST.sha256 stays, marking DEST as this script's, until the
# new one replaces it.
rm -f "$dest.rev"
mkdir -p "$(dirname "$dest")"
new=$(mktemp -d "$dest.XXXXXX")
remove_new() { rm -rf "$new" "$new.sha256"; }
at_exit remove_new
git --git-dir="$gitdir" archive "$rev" -- dune-project theories | tar -x -C "$new" ||
  die "could not extract revision $rev from $1"
[[ -f "$new/dune-project" && -f "$new/theories/dune" ]] ||
  die "revision $rev has no dune-project and theories/dune"
stub_digests "$new" > "$new.sha256" || die "revision $rev holds entries that are not plain files"
rm -rf "$dest"
mv "$new" "$dest"
mv "$new.sha256" "$dest.sha256"
echo "$rev" > "$dest.rev"
echo "staged wasm-verifier $rev ($tag) in $dest" >&2
if [[ "$dest" != "$PWD/.ctx/wasm-verifier" ]]; then
  echo "check with it: WASM_VERIFIER_CONTEXT=$(printf %q "$dest") proofs/check.sh real" >&2
fi
if [[ -n "${top:-}" ]]; then
  printf "it survives branch switches in %s: remove it before running another branch's code there: rm -rf %q %q %q\n" \
    "$top" "$dest" "$dest.sha256" "$dest.rev" >&2
fi
finish
