# shellcheck shell=bash
# Helpers for the scripts in scripts/ and proofs/, which source this file
# after changing to the repository root. Written for bash 3.2 (macOS) and
# later; each script refuses to run under another shell, such as dash or zsh.

# COMPOSE: docker compose on this repository's compose.yaml, named explicitly
# so that a COMPOSE_FILE (in the environment or in .env) or a
# compose.override.yaml cannot swap in another definition, whose containers
# and volumes a teardown would then remove. COMPOSE_PROJECT_NAME and
# STELLAR_PORT still apply. DC: the same with its progress output off.
COMPOSE=(docker compose -f compose.yaml)
# shellcheck disable=SC2034 # used by the scripts that source this file
DC=("${COMPOSE[@]}" --progress quiet)

# die MESSAGE [STATUS]: print MESSAGE to stderr and exit with STATUS (default 1).
die() {
  echo "error: $1" >&2
  exit "${2:-1}"
}

# sha256 FILE: print the SHA-256 of FILE in hex.
sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

# ledger_version: the toolchain version Inference.toml pins (its infc_version).
ledger_version() {
  sed -n 's/^infc_version *= *"\([^"]*\)".*$/\1/p' Inference.toml
}

# need_compose: fail with status 2 unless Docker Compose v2 is available and
# the Docker daemon answers. The scripts were measured with Docker Compose
# 2.38.2 and 5.5.1; no version is checked or refused.
need_compose() {
  docker compose version >/dev/null 2>&1 ||
    die "this needs Docker with the Compose v2 plugin (docker compose)" 2
  docker info >/dev/null 2>&1 ||
    die "the Docker daemon is not reachable (is Docker running? check DOCKER_HOST and the docker context)" 2
}

# project_name: the compose project's name: COMPOSE_PROJECT_NAME, from the
# environment or from .env, or else compose.yaml's own.
project_name() {
  local name
  name=$("${COMPOSE[@]}" config 2>/dev/null | sed -n 's/^name: //p') || true
  echo "${name:-${COMPOSE_PROJECT_NAME:-stellar-amm-math}}"
}

# down_command: the command that stops this project and removes its volumes,
# spelled with the compose file's path and the project's name, so that it
# works from any directory and whatever the shell it is pasted into has set.
down_command() {
  printf "docker compose -f %q -p %q --profile '*' down -v\n" "$PWD/compose.yaml" "$(project_name)"
}

# start_command: the command that starts this project's stopped local
# network again, spelled like down_command's, and with --no-recreate, so
# that compose starts the container that holds the deployments instead of
# replacing it with an empty one when its configuration seems to have
# changed.
start_command() {
  printf "docker compose -f %q -p %q up -d --wait --no-recreate stellar\n" "$PWD/compose.yaml" "$(project_name)"
}

# network_state: running, stopped or none: whether this project's stellar
# container is running, exists without running (after `compose stop`, or a
# restart of Docker; it still holds its deployments, as do its volumes), or
# does not exist. When compose cannot list the project's containers (a
# broken .env, say), it fails after compose's own message instead of
# answering none. Call it as state=$(network_state) || exit 1.
network_state() {
  local running all
  if ! running=$("${DC[@]}" ps --status running -q stellar) ||
    ! all=$("${DC[@]}" ps -aq stellar); then
    echo "error: docker compose could not list the containers of project $(project_name)" >&2
    return 1
  fi
  if [[ -n "$running" ]]; then
    echo running
  elif [[ -n "$all" ]]; then
    echo stopped
  else
    echo none
  fi
}

# start_network: start this project's local network, wait at most 300
# seconds for it to become healthy, and set network_started=1. A network
# that exists already is reused: a running one as it is, and a stopped one
# is started again. Neither is recreated, whatever changed in its
# configuration (STELLAR_PORT, say), so that a deployment on it survives.
# network_reused says which it was, from before anything starts: running,
# stopped, or 0 for a network this call created; it stays empty until
# start_network has looked. If the network does not become healthy, print
# its health checks and last log lines, which a teardown would otherwise
# remove, and fail.
# shellcheck disable=SC2034 # read by the scripts that source this file
network_started=0 network_reused=
# shellcheck disable=SC2034
start_network() {
  local container log hint state up=(up -d --wait --wait-timeout 300)
  state=$(network_state) || exit 1
  case "$state" in
    running)
      network_reused=running
      echo "reusing the running local network of project $(project_name)" >&2
      up+=(--no-recreate)
      ;;
    stopped)
      network_reused=stopped
      echo "starting the stopped local network of project $(project_name) again; it keeps its deployments" >&2
      up+=(--no-recreate)
      ;;
    *)
      network_reused=0
      echo "starting the local network" >&2
      ;;
  esac
  log=$(mktemp)
  # compose's output goes to stderr, and is kept to tell a port clash apart.
  if "${COMPOSE[@]}" "${up[@]}" stellar 2>&1 | tee "$log" >&2; then
    rm -f "$log"
    network_started=1
    return 0
  fi
  if grep -qiE 'port is already allocated|address already in use' "$log"; then
    if [[ "$network_reused" == stopped ]]; then
      hint="the host port the stopped network was created with is taken; free it, or remove the network (and what it holds) with $(down_command) and set STELLAR_PORT to a free port"
    else
      hint="host port ${STELLAR_PORT:-8000} is taken; set STELLAR_PORT to a free port"
    fi
  else
    hint="see compose's messages above"
  fi
  rm -f "$log"
  container=$("${DC[@]}" ps -aq stellar 2>/dev/null || true)
  if [[ -n "$container" ]]; then
    echo "the health checks of the stellar service:" >&2
    docker inspect --format '{{if .State.Health}}{{range .State.Health.Log}}exit {{.ExitCode}}: {{.Output}}{{println}}{{end}}{{else}}none: the container never ran{{end}}' \
      "$container" >&2 || true
    echo "its last 200 log lines:" >&2
    "${DC[@]}" logs --no-color --tail 200 stellar >&2 || true
  fi
  die "the local network did not start and become healthy ($hint)"
}

# fund NAME NETWORK: have friendbot fund the identity NAME on NETWORK, with
# three attempts. It runs the calling script's `cli` function. `keys generate
# --fund` exits with status 0 when the funding fails, so the scripts generate
# a key without --fund and fund it here.
fund() {
  local attempt
  for attempt in 1 2 3; do
    if cli keys fund "$1" --network "$2" >&2; then
      return 0
    fi
    [[ $attempt -lt 3 ]] || break
    echo "funding $1 on $2 failed; trying again in $((attempt * 5)) seconds" >&2
    sleep $((attempt * 5))
  done
  die "friendbot could not fund $1 on $2"
}

# check_network_identity NETWORK: fail unless the network the CLI reaches
# under the name NETWORK reports that network's passphrase. It runs the
# calling script's `cli` function, and needs jq. Without it, a network that
# the CLI's config defines under the same name, or STELLAR_RPC_URL and
# STELLAR_NETWORK_PASSPHRASE in the container's environment, could put any
# network in its place.
check_network_identity() {
  local expected actual
  case "$1" in
    testnet) expected='Test SDF Network ; September 2015' ;;
    local) expected='Standalone Network ; February 2017' ;;
    *) die "no passphrase is known for the network '$1'" 2 ;;
  esac
  actual=$(cli network info --network "$1" --output json | jq -r '.passphrase // empty') ||
    die "could not ask the $1 network for its passphrase"
  [[ "$actual" == "$expected" ]] ||
    die "the network the CLI calls $1 has the passphrase '$actual', not '$expected'; nothing was done on it"
}

# stub_digests DIR: print "SHA256  PATH" for each file under DIR, in C-locale
# order of PATH: the format of docker/rocq/stub-TAG.sha256 (and of
# .ctx/wasm-verifier.sha256). Fails if DIR holds anything but regular files
# and directories, such as a symlink.
stub_digests() {
  local dir=$1 odd files f digest
  odd=$(cd "$dir" && find . ! -type f ! -type d) || return 1
  if [[ -n "$odd" ]]; then
    echo "error: $dir holds entries that are neither files nor directories:" >&2
    echo "$odd" >&2
    return 1
  fi
  files=$(cd "$dir" && find . -type f | sed 's|^\./||' | LC_ALL=C sort) || return 1
  while IFS= read -r f; do
    [[ -n "$f" ]] || continue
    digest=$(sha256 "$dir/$f") || return 1
    printf '%s  %s\n' "$digest" "$f"
  done <<< "$files"
}

# at_exit FUNCTION: call FUNCTION when the script exits, for any reason.
# A script that uses it ends with `finish`, and an exit with status 0 before
# that becomes status 1: bash 3.2 (macOS) exits with status 0 when `set -u`
# stops a script that has an EXIT trap, which would otherwise pass for success.
_at_exit=:
_finished=0
at_exit() {
  _at_exit="$_at_exit; $1"
  trap _on_exit EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
}
_on_exit() {
  local rc=$?
  eval "$_at_exit" || true
  if [[ $rc -eq 0 && $_finished -ne 1 ]]; then
    echo "error: $0 stopped before its end" >&2
    rc=1
  fi
  exit "$rc"
}
finish() { _finished=1; }
