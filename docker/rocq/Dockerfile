# syntax=docker/dockerfile:1@sha256:ecfaec9ed6d810b56388c508f4121597bfbba70d41a6dfeee4d8cad5f295fc32
#
# Rocq images that type-check the proof-mode .v files Inference emits
# (proofs/main.v), one target per library the .v can be checked against:
#
#   stub  the public signature stub Inferara/inference ships at a release tag
#         (core/wasm-to-v/rocq-stub), from the named build context
#         `rocq-stub` that scripts/fetch-rocq-stub.sh stages in .ctx/rocq-stub
#   real  coq-wasm at a pinned commit and the wasm-verifier library, from the
#         named build context `wasm-verifier` that
#         scripts/stage-wasm-verifier.sh stages in .ctx/wasm-verifier or in a
#         DEST of your own (the library's dune-project and theories/, nothing
#         else)
#
# compose.yaml builds both (services rocq-stub and rocq-real), and
# proofs/check.sh runs them, with main.v on stdin. Both run check.sh,
# installed as rocq-check. It is copied last in each target, so that editing
# it rebuilds no library.
#
# The wasm-verifier library is not public, and the real image and its build
# cache hold its sources: never push that image, `docker save` it, or export
# its layers to a shared build cache. proofs/check.sh says how to remove every
# copy.
#
# coqorg/coq publishes linux/amd64 only: native on GitHub's ubuntu runners,
# emulated on Apple Silicon, where the first build of `real` takes about 22
# minutes (opam dependencies, then wasm-verifier). A Linux arm64 host has no
# amd64 emulation by default; register it once with (pinned: it runs
# privileged)
#   docker run --privileged --rm tonistiigi/binfmt@sha256:400a4873b838d1b89194d982c45e5fb3cda4593fbfd7e08a02e76b03b21166f0 --install amd64
#
# Reproducibility: the Dockerfile frontend and the base image are pinned by
# digest, and no `opam update` runs, so every opam package resolves against
# the repository snapshot inside the base image. The versions below are what
# that snapshot resolves; they are spelled out so that a changed base fails
# loudly instead of drifting.

ARG COQ_IMAGE=coqorg/coq:8.20.1-ocaml-4.13.1-flambda@sha256:18ebf3da56e60e3ddfd7d4e51f4c53d10241a129f34e93dacbc71562dd43c57a

FROM ${COQ_IMAGE} AS base
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
USER coq
WORKDIR /home/coq

FROM base AS stub
COPY --from=rocq-stub --chown=coq:coq . /home/coq/rocq-stub/
# Compiled in _CoqProject order with the load-path flags rocq-check uses for
# this tier.
# The loop stops at the first module that fails: a `for` loop's status is its
# last command's, and SHELL has no -e.
RUN cd /home/coq/rocq-stub \
 && modules=$(grep '\.v$' _CoqProject) \
 && test -n "$modules" \
 && for m in $modules; do \
      opam exec -- coqc -Q wasm Wasm -Q wasm_verifier WasmVerifier "$m" || exit 1; \
    done
ENV ROCQ_TIER=stub
COPY --chmod=0755 check.sh /usr/local/bin/rocq-check
ENTRYPOINT ["opam", "exec", "--", "rocq-check"]

FROM base AS real-deps
ARG COQ_WASM_COMMIT=0fd83fa708922721132b6d6737179568d1f1d553
RUN opam pin add -y -n coq-wasm "https://github.com/WasmCert/WasmCert-Coq.git#${COQ_WASM_COMMIT}" \
 && opam install -y -j "$(nproc)" \
      coq-wasm.2.2.0 coq-compcert.3.17 coq-mathcomp-ssreflect.2.4.0 coq-ext-lib.0.13.1 \
 && opam clean -a -c -s --logs \
 && opam list --installed --columns=name,version > /home/coq/opam-installed.txt

FROM real-deps AS real
COPY --from=wasm-verifier --chown=coq:coq . /home/coq/wasm-verifier/
RUN cd /home/coq/wasm-verifier && opam exec -- dune build -j "$(nproc)"
ENV ROCQ_TIER=real
COPY --chmod=0755 check.sh /usr/local/bin/rocq-check
ENTRYPOINT ["opam", "exec", "--", "rocq-check"]
