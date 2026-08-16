# _common.sh — shared helpers for broot's build/release/deploy scripts.
#
# Source it near the top of a script:
#   source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"
# It enables strict mode and an error trap for the sourcing script.
#
# This file is not meant to be run directly.

set -Eeuo pipefail

# --- error reporting ---------------------------------------------------------
_broot_on_err() {
    local code=$?
    printf '\n%s✗ failed (exit %d)%s at %s:%s\n    %s\n' \
        "${_c_err:-}" "$code" "${_c_reset:-}" \
        "${BASH_SOURCE[1]:-?}" "${BASH_LINENO[0]:-?}" "$BASH_COMMAND" >&2
    exit "$code"
}
trap _broot_on_err ERR

# Resolve all relative paths against the repo root. This file lives in
# build-scripts/, so the root is its parent directory.
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# --- decorations (only when writing to a terminal, and NO_COLOR unset) -------
if [[ -t 1 && -z "${NO_COLOR:-}" ]]; then
    _c_reset=$'\033[0m'; _c_h1=$'\033[1;97;44m'; _c_h2=$'\033[97;44m'
    _c_ok=$'\033[32m'; _c_warn=$'\033[33m'; _c_err=$'\033[31m'
else
    _c_reset=; _c_h1=; _c_h2=; _c_ok=; _c_warn=; _c_err=
fi
h1()   { printf '\n%s %s %s\n' "$_c_h1" "$*" "$_c_reset"; }
h2()   { printf '\n%s %s %s\n' "$_c_h2" "$*" "$_c_reset"; }
info() { printf '    %s\n' "$*"; }
ok()   { printf '    %s✓%s %s\n' "$_c_ok" "$_c_reset" "$*"; }
warn() { printf '%s⚠ %s%s\n' "$_c_warn" "$*" "$_c_reset" >&2; }
die()  { printf '%s✗ %s%s\n' "$_c_err" "$*" "$_c_reset" >&2; exit 1; }

# --- tool detection ----------------------------------------------------------
have() { command -v "$1" >/dev/null 2>&1; }
need() { # need <tool> [install hint]
    have "$1" && return 0
    die "required tool '$1' not found${2:+ — $2}"
}

# --- facts about the project / host ------------------------------------------
# First matching line only, without piping into head (which would close the
# pipe early and, under pipefail, abort the script on SIGPIPE).
broot_version() {
    local v
    v=$(sed -n 's/^version = "\([^"]*\)".*/\1/p' Cargo.toml)
    printf '%s\n' "${v%%$'\n'*}"
}
host_target() {
    local t
    t=$(rustc -vV | sed -n 's/^host: //p')
    printf '%s\n' "${t%%$'\n'*}"
}
host_os() { uname -s; }

# --- portability helpers -----------------------------------------------------
sed_inplace() { # sed_inplace <expr> <file>   (GNU vs BSD/macOS in-place syntax)
    if sed --version >/dev/null 2>&1; then
        sed -i "$1" "$2"
    else
        sed -i '' "$1" "$2"
    fi
}
safe_wipe() { # remove the *contents* of a directory, refusing dangerous targets
    local dir=${1:-}
    [[ -n "$dir" ]] || die "safe_wipe: refusing to wipe an empty path"
    [[ "$dir" != "/" && "$dir" != "$HOME" ]] || die "safe_wipe: refusing to wipe '$dir'"
    [[ -d "$dir" ]] && rm -rf "${dir:?}"/*
    return 0
}

# Ensure a container engine (for cross / dockerized zig) is running and select
# it for cross. cross defaults to docker even when its daemon is down, then
# silently falls back to a host build — this makes that a clear, early error.
ensure_container_engine() {
    if [[ -n ${CROSS_CONTAINER_ENGINE:-} ]]; then
        "$CROSS_CONTAINER_ENGINE" info >/dev/null 2>&1 || \
            die "container engine '$CROSS_CONTAINER_ENGINE' isn't running — start it, then retry"
        return 0
    fi
    local eng
    for eng in docker podman; do
        if have "$eng" && "$eng" info >/dev/null 2>&1; then
            export CROSS_CONTAINER_ENGINE="$eng"
            info "container engine: $eng"
            return 0
        fi
    done
    die "no running container engine — start Docker Desktop / colima, or 'podman machine start' (first time: 'podman machine init')"
}

# --- release staging (optional multi-host builds via a shared server) ---------
# When BROOT_STAGE_HOST is set (see _local.sh), build-all-targets.sh pushes its
# build/ to <host>:<BROOT_STAGE_DIR>/<id>/ and release.sh fetches the union from
# there. <id> ties every artifact to one commit. When unset, builds stay local.
staging_configured() { [[ -n ${BROOT_STAGE_HOST:-} ]]; }

require_clean_tree() {
    git diff --quiet HEAD 2>/dev/null \
        || die "working tree has uncommitted changes — commit before a staged release"
}

release_id() { # <version>-<short commit>, e.g. 1.58.0-0717a94
    printf '%s-%s\n' "$(broot_version)" "$(git rev-parse --short HEAD)"
}

# Date of HEAD (YYYY/MM/DD) — used for the man page so both hosts, and re-runs,
# produce identical metadata for a given commit.
commit_date() { git show -s --format=%cd --date=format:'%Y/%m/%d' HEAD; }

stage_push() { # stage_push <local-dir>  -> pushes its contents into <dir>/<id>/
    local dir=$1 id dest
    id=$(release_id)
    dest="$BROOT_STAGE_DIR/$id"
    ssh "$BROOT_STAGE_HOST" "mkdir -p '$dest'"
    rsync -az "$dir"/ "$BROOT_STAGE_HOST:$dest/"
}

stage_fetch() { # stage_fetch <local-dir>  <- pulls <dir>/<id>/ into it
    local dir=$1 id
    id=$(release_id)
    rsync -az "$BROOT_STAGE_HOST:$BROOT_STAGE_DIR/$id/" "$dir"/
}

# --- machine-local overrides (gitignored): staging + deploy paths -------------
_broot_common_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[[ -f "$_broot_common_dir/_local.sh" ]] && source "$_broot_common_dir/_local.sh"
: "${BROOT_STAGE_HOST:=}"
: "${BROOT_STAGE_DIR:=broot-staging}"
