#!/usr/bin/env bash
#
# Assemble and package a full broot release: build/ plus broot_<version>.zip in
# the repo root (also copied into releases/).
#
# With staging configured (BROOT_STAGE_HOST, see _local.sh) this does NOT build —
# it fetches the artifacts every host pushed for the current commit and checks
# the set is complete (so the macOS binary from the Mac and the armv7-musl binary
# from Linux come together). Without staging it falls back to a local build-all
# (single host). Either way it then verifies every binary and zips.
#
# This isn't used for normal compilation (see https://dystroy.org/broot).

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_targets.sh"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

version=$(broot_version)

if staging_configured; then
    require_clean_tree
    id=$(release_id)
    h1 "Assembling release $version from $BROOT_STAGE_HOST ($id)"
    rm -rf build && mkdir build
    h2 "Fetching staged artifacts"
    stage_fetch build
    ok "fetched $BROOT_STAGE_DIR/$id"
else
    h1 "Building release $version locally (no staging configured)"
    "$here/build-all-targets.sh"
fi

# Completeness: every target of the full (cross-host) release manifest must be
# present, and each binary is verified (arch, and no duplicate dylib on macOS).
h2 "Checking release completeness"
missing=()
while IFS='|' read -r label triple tool features; do
    bin=$(target_binary "$triple")
    if [[ -f $bin ]]; then
        verify_binary "$bin" "$triple"
    else
        missing+=("$label ($triple)")
    fi
done < <(all_release_targets)
[[ ${#missing[@]} -eq 0 ]] || die "release incomplete — missing binaries: ${missing[*]}"
ok "all release targets present and verified"

# The non-binary artifacts must be there too.
for f in README.md CHANGELOG.md broot.1 version completion default-conf resources install.md; do
    [[ -e "build/$f" ]] || die "release artifact missing from build/: $f"
done
ok "artifacts present (completions, config, font, man page, changelog)"

# build the release archive
rm -f broot_*.zip
( cd build && zip -rq "../broot_$version.zip" -- * )
ok "created broot_$version.zip"

# copy it to the releases folder
mkdir -p "releases/broot_$version"
cp "broot_$version.zip" "releases/broot_$version/"
h1 "Release $version ready: broot_$version.zip"
