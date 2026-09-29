#!/usr/bin/env bash
# Type-checks proofs/main.v with coqc, in the Rocq images compose.yaml builds.
#
# Usage: proofs/check.sh stub|real [--self-test]     (from anywhere)
#   stub         against the public signature stub Inferara/inference ships at
#                the toolchain's release tag (v + Inference.toml's
#                infc_version, or ROCQ_STUB_TAG=vX.Y.Z), staged in
#                .ctx/rocq-stub by scripts/fetch-rocq-stub.sh and matching
#                docker/rocq/stub-TAG.sha256 (UNVERIFIED, with a warning, for
#                a tag without that file)
#   real         against coq-wasm and the private wasm-verifier library at the
#                revision docker/rocq/wasm-verifier-TAG.rev records, staged by
#                scripts/stage-wasm-verifier.sh CHECKOUT [DEST] and unchanged
#                since; WASM_VERIFIER_CONTEXT=DEST names a DEST other than
#                .ctx/wasm-verifier (a relative one from the current directory)
#   --self-test  also show that the check means something: the image proves
#                which library it carries (the real one declares HA_pto and
#                ktrue, the stub neither), and a copy of main.v with one
#                reference renamed must fail with coqc's error for it
#
# Trust model. main.v is the translator's output. proofs/generate.sh --check
# establishes that it is the file src/ generates: CI runs it with the
# toolchain Inference.toml pins, and it fails on any difference. This script
# then establishes that the file elaborates against the chosen library and
# states the expected theorems: coqc checks main.v followed by a section that
# uses each of them at its expected type, with fully qualified names
# (main.valid_main : WasmVerifier.Verifier.ValidModule main.main, and
# main.valid_main__X : WasmVerifier.Verifier.ValidSpec main.main
# main.main__X_specs for each spec block X in src/main.inf, main__F_X for one
# in src/F.inf), so a translator change that renames or drops an obligation
# fails here. It does not defend against a hand-edited, adversarial main.v,
# which can pass by defining those names itself. Nor does a pass mean that
# the obligations are proved: their proofs end in Admitted.
#
# The image is rebuilt from docker/rocq/ first: in seconds once Docker has
# cached it; the first build of `real` takes about 22 minutes on Apple Silicon
# and prints nothing meanwhile (linux/amd64 only; see docker/rocq/Dockerfile).
# main.v goes to the container on stdin, so the Rocq services mount nothing,
# and the container's `ok` line must carry the SHA-256 of the whole text sent.
#
# The wasm-verifier library is not public. Run the real tier only if you may
# read the library, only where its output stays private (coqc's error on a
# failure can quote the library), and never with a pull request's files
# checked out (its scripts, compose.yaml or Dockerfile could read the staged
# library). To remove every copy of it (PROJECT: COMPOSE_PROJECT_NAME, default
# stellar-amm-math):
#   docker image rm PROJECT-rocq-real
#   docker builder prune --all --force   ALL the build cache of that builder,
#       other projects' included: a --filter on the library's name, with or
#       without --all, leaves its COPY layer and the layers built on it
#   docker buildx history rm REF...      each build that `docker buildx
#       history ls` names real or docker/rocq (real): its log quotes the
#       library, and no prune removes it
#   rm -rf DEST DEST.sha256 DEST.rev     (DEST: .ctx/wasm-verifier by default)
# Bash only: under another shell (dash's sh, zsh) stop with a clear message.
[ -n "${BASH_VERSION:-}" ] || { echo "error: $0 needs bash: run it with bash '$0'" >&2; exit 2; }
set -euo pipefail
unset CDPATH
caller=$PWD
cd "$(dirname "$0")/.."
. scripts/lib.sh

usage() { die "usage: proofs/check.sh stub|real [--self-test]" 2; }
[[ $# -eq 1 || ($# -eq 2 && "$2" == --self-test) ]] || usage
tier=$1 self_test=0
[[ $# -eq 1 ]] || self_test=1
case "$tier" in
  stub | real) service=rocq-$tier ;;
  *) usage ;;
esac

# expected_lists: the spec lists src/ asks for, one per line: main__X for
# each spec block X in src/main.inf, and main__F_X for one in src/F.inf.
expected_lists() {
  local f prefix
  for f in src/*.inf; do
    prefix=main__$(basename "$f" .inf)_
    [[ "$prefix" != main__main_ ]] || prefix=main__
    sed -n "s/^spec \([A-Za-z_][A-Za-z0-9_]*\) {\$/$prefix\1/p" "$f"
  done
}

# with_uses FILE: FILE, then a section that uses each expected theorem at its
# expected type, every name qualified: the predicates by the library's
# module, the host class by coq-wasm's, FILE's names by main (main.v's).
with_uses() {
  local x
  cat "$1"
  printf '\n(* Appended by proofs/check.sh: each obligation, used at its type. *)\n'
  printf 'Section CheckObligations.\nContext `{ho: Wasm.host.host}.\n'
  printf 'Definition check_valid_main : WasmVerifier.Verifier.ValidModule main.main := main.valid_main.\n'
  for x in $(expected_lists); do
    printf 'Definition check_valid_%s : WasmVerifier.Verifier.ValidSpec main.main main.%s_specs := main.valid_%s.\n' "$x" "$x" "$x"
  done
  printf 'End CheckObligations.\n'
}

# check FILE: FILE with the uses appended, sent to the container as main.v.
# Prints the container's `ok` line, which must carry the text's digest.
check() {
  local sha out
  with_uses "$1" > "$tmp/sent.v"
  sha=$(sha256 "$tmp/sent.v")
  out=$("${DC[@]}" run --rm -T "$service" --stdin main.v < "$tmp/sent.v") || return
  [[ "$out" == "ok   main.v [$tier] sha256=$sha" ]] ||
    { echo "error: the checker passed, but not on the text sent (sha256 $sha): '$out'" >&2; return 3; }
  echo "$out"
}

need_compose
[[ -f proofs/main.v ]] || die "no proofs/main.v; build it with proofs/generate.sh" 2
tmp=$(mktemp -d)
remove_tmp() { rm -rf "$tmp"; }
at_exit remove_tmp

if [[ "$tier" == stub ]]; then
  tag=${ROCQ_STUB_TAG:-v$(ledger_version)}
  [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "ROCQ_STUB_TAG is not a release tag: '$tag'" 2
  if [[ ! -d .ctx/rocq-stub || "$(cat .ctx/rocq-stub.tag 2>/dev/null)" != "$tag" ]]; then
    bash scripts/fetch-rocq-stub.sh "$tag" >&2
  fi
  sums=docker/rocq/stub-$tag.sha256
  library="the $tag stub (matches $sums)"
  if [[ ! -f "$sums" ]]; then
    echo "warning: no $sums, so the $tag stub in .ctx/rocq-stub is UNVERIFIED" >&2
    library="the $tag stub (UNVERIFIED)"
  elif ! stub_digests .ctx/rocq-stub > "$tmp/digests" || ! diff -u "$sums" "$tmp/digests" >&2; then
    die "the stub in .ctx/rocq-stub does not match $sums; run scripts/fetch-rocq-stub.sh $tag"
  fi
  "${COMPOSE[@]}" --profile rocq build rocq-stub >&2
else
  tag=v$(ledger_version)
  revfile=docker/rocq/wasm-verifier-$tag.rev
  [[ -f "$revfile" ]] || die "no $revfile: the wasm-verifier revision of $tag is not recorded" 2
  pin=$(sed '/^#/d' "$revfile" | tr -d '[:space:]')
  [[ "$pin" =~ ^[0-9a-f]{40}$ ]] || die "$revfile holds no revision" 2
  # The staged library as an absolute path, exported so that compose builds
  # from the directory checked here (the environment wins over .env).
  wv=${WASM_VERIFIER_CONTEXT:-$PWD/.ctx/wasm-verifier}
  while [[ "$wv" == */ && "$wv" != / ]]; do wv=${wv%/}; done
  [[ "$wv" == /* ]] || wv=$caller/$wv
  export WASM_VERIFIER_CONTEXT=$wv
  [[ -d "$wv" ]] ||
    die "no wasm-verifier library in $wv; stage revision $pin there with: scripts/stage-wasm-verifier.sh CLONE $(printf %q "$wv")" 2
  [[ "$(cat "$wv.rev" 2>/dev/null)" == "$pin" ]] ||
    die "the library in $wv is not revision $pin, which $tag pins; run scripts/stage-wasm-verifier.sh again" 2
  if ! stub_digests "$wv" > "$tmp/digests" || ! cmp -s "$wv.sha256" "$tmp/digests"; then
    die "the files in $wv are not the ones $wv.sha256 recorded when they were staged; run scripts/stage-wasm-verifier.sh again" 2
  fi
  library="wasm-verifier $pin (staged in $wv)"
  # real_hint: from the build on, whatever happens, say where the copies are.
  real_hint() {
    echo "to remove every copy of the library when done (the prune drops ALL this builder's cache):" >&2
    printf '  docker image rm %q\n  docker builder prune --all --force\n' "$(project_name)-rocq-real" >&2
    echo "  docker buildx history rm REF...   (each build that docker buildx history ls names real or docker/rocq (real))" >&2
    printf '  rm -rf %q %q %q\n' "$wv" "$wv.sha256" "$wv.rev" >&2
  }
  at_exit real_hint
  echo "building the rocq-real image, quietly: its build log quotes the library" >&2
  "${DC[@]}" --profile rocq-real build rocq-real >&2 ||
    die "the rocq-real image did not build; for its log, which quotes the library, run: WASM_VERIFIER_CONTEXT=$(printf %q "$wv") docker compose -f $(printf %q "$PWD/compose.yaml") -p $(printf %q "$(project_name)") --progress plain --profile rocq-real build rocq-real"
fi

if [[ $self_test -eq 1 ]]; then
  "${DC[@]}" run --rm -T "$service" --self-test </dev/null
fi
echo "checking proofs/main.v and its $(($(expected_lists | grep -c . || true) + 1)) obligations against $library" >&2
check proofs/main.v

# The negative control: main.v with its first BI_return renamed must fail
# with coqc's error for it (compared with coqc's wrapped lines joined).
if [[ $self_test -eq 1 ]]; then
  want="The reference BI_return_renamed was not found in the current environment."
  sed '1,/BI_return ::/s/BI_return ::/BI_return_renamed ::/' proofs/main.v > "$tmp/renamed.v"
  ! cmp -s proofs/main.v "$tmp/renamed.v" || die "self-test: found no BI_return in main.v to rename"
  status=0
  out=$(check "$tmp/renamed.v" 2>&1) || status=$?
  if [[ $status -eq 0 || "$(tr -s ' \n' '  ' <<< "$out")" != *"$want"* ]]; then
    echo "$out" >&2
    die "self-test: the copy with BI_return renamed exited with status $status, without \"$want\""
  fi
  echo "self-test: the copy with BI_return renamed fails as expected: $want" >&2
fi
finish
