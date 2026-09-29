#!/usr/bin/env bash
# Stages the public Rocq signature stub that Inferara/inference ships at a
# release tag (core/wasm-to-v/rocq-stub) in .ctx/rocq-stub, where
# proofs/check.sh stub builds it into its image.
#
# Usage: scripts/fetch-rocq-stub.sh [TAG]
#   TAG  a release tag such as v0.0.6 (default: v + Inference.toml's infc_version)
#
# The download needs no credentials, and only HTTPS is followed. When
# docker/rocq/stub-TAG.sha256 exists, the stub must hold exactly the files it
# lists, with those digests, and nothing but files and directories, or
# nothing is staged: a tag that moved fails here. For a tag without such a
# file the stub is staged unverified, with a warning on stderr, and its
# digests go to stdout in that file's format; after reviewing the stub,
#   scripts/fetch-rocq-stub.sh TAG > sums.new &&
#     mv sums.new docker/rocq/stub-TAG.sha256
# records them (not with `> docker/rocq/stub-TAG.sha256` directly: the shell
# creates that file empty first, and the stub is then checked against it).
#
# The staged files replace the old ones in one step, and .ctx/rocq-stub.tag
# names the tag only once they are complete.
# Bash only: under another shell (dash's sh, zsh) stop with a clear message.
[ -n "${BASH_VERSION:-}" ] || { echo "error: $0 needs bash: run it with bash '$0'" >&2; exit 2; }
set -euo pipefail
unset CDPATH
cd "$(dirname "$0")/.."
. scripts/lib.sh

[[ $# -le 1 ]] || die "usage: scripts/fetch-rocq-stub.sh [TAG]" 2
tag=${1:-v$(ledger_version)}
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "not a release tag: '$tag'" 2
sums=docker/rocq/stub-$tag.sha256

tmp=$(mktemp -d)
new=
remove_tmp() { rm -rf "$tmp" ${new:+"$new"}; }
at_exit remove_tmp

url=https://codeload.github.com/Inferara/inference/tar.gz/refs/tags/$tag
echo "fetching $url" >&2
# --retry covers timeouts, 408, 429 and 5xx; a 404 (no such tag) fails at once.
code=$(curl -sSL --fail --proto '=https' --proto-redir '=https' \
  --retry 4 --retry-connrefused --retry-delay 5 --connect-timeout 20 --max-time 300 \
  -w '%{http_code}' -o "$tmp/src.tar.gz" "$url") || {
  [[ "${code:-}" != 404 ]] || die "Inferara/inference has no tag $tag" 2
  die "download failed: $url"
}
# GitHub names the archive's top directory after the tag without its "v".
path=inference-${tag#v}/core/wasm-to-v/rocq-stub
tar -xzf "$tmp/src.tar.gz" -C "$tmp" "$path" || die "$tag has no core/wasm-to-v/rocq-stub"
stub=$tmp/$path

stub_digests "$stub" > "$tmp/digests" || die "the stub at $tag is not plain files; nothing staged"
if [[ -f "$sums" ]]; then
  if ! diff -u "$sums" "$tmp/digests" >&2; then
    die "the stub at $tag does not match $sums (lines with - are expected, + are fetched); nothing staged"
  fi
  echo "verified $(wc -l < "$tmp/digests" | tr -d ' ') files against $sums" >&2
else
  echo "warning: no $sums, so the stub at $tag is UNVERIFIED; its digests are on stdout" >&2
  cat "$tmp/digests"
fi

rm -f .ctx/rocq-stub.tag
mkdir -p .ctx
new=$(mktemp -d .ctx/rocq-stub.XXXXXX)
chmod 755 "$new"
cp -R "$stub/." "$new/"
rm -rf .ctx/rocq-stub
mv "$new" .ctx/rocq-stub
new=
echo "$tag" > .ctx/rocq-stub.tag
echo "staged the $tag stub in .ctx/rocq-stub" >&2
finish
