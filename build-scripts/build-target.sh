#!/usr/bin/env bash
#
# Build a single broot target (or every target matching a filter) into
# build/<triple>/. Handy to check that a change still compiles for another
# platform, without running the full release build.
#
# Usage:
#   ./build-target.sh <filter>     e.g. ./build-target.sh aarch64-apple-darwin
#                                       ./build-target.sh MUSL
#   ./build-target.sh --list       list the available targets
#
# The filter is matched as a substring against each target's label and triple.
#
# This is NOT for normal installation — see https://dystroy.org/broot/install.

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_targets.sh"

filter=${1:-}
[[ -n $filter ]] || die "usage: $(basename "$0") <target-filter>   (try --list)"

if [[ $filter == --list || $filter == -l ]]; then
    h2 "Available targets (host $(host_os), darwin=$DARWIN_METHOD)"
    while IFS='|' read -r label triple tool _feat; do
        [[ -n $label ]] || continue
        printf '    %-9s %-32s %s\n' "$tool" "$triple" "$label"
    done < <(all_targets)
    exit 0
fi

matched=0
while IFS= read -r row; do
    [[ -n $row ]] || continue
    IFS='|' read -r label triple _tool _feat <<< "$row"
    if [[ $label == *"$filter"* || $triple == *"$filter"* ]]; then
        build_row "$row"
        matched=$((matched + 1))
    fi
done < <(all_targets)

[[ $matched -gt 0 ]] || die "no target matched '$filter'   (try --list)"
h1 "Built $matched target(s) matching '$filter'"
