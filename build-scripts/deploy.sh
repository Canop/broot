#!/usr/bin/env bash
#
# Build the release, copy it to the download directory, then run the deploy hook.
# Machine-specific settings live in build-scripts/_local.sh (gitignored):
#   BROOT_DOWNLOAD_DIR  (required) where build/ and the zip are copied
#   BROOT_DEPLOY_HOOK   (optional) command run afterwards to publish

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

[[ -n ${BROOT_DOWNLOAD_DIR:-} ]] || die "BROOT_DOWNLOAD_DIR is not set — configure it in build-scripts/_local.sh"

# build the release zip (and the build/ directory)
"$here/release.sh"

version=$(broot_version)

h1 "Deploying $version to $BROOT_DOWNLOAD_DIR"
mkdir -p "$BROOT_DOWNLOAD_DIR"
safe_wipe "$BROOT_DOWNLOAD_DIR"
cp -r build/* "$BROOT_DOWNLOAD_DIR/"
cp "broot_$version.zip" "$BROOT_DOWNLOAD_DIR/"
if [[ -n ${BROOT_DEPLOY_HOOK:-} ]]; then
    h2 "Running deploy hook"
    eval "$BROOT_DEPLOY_HOOK"
fi
ok "deployed $version"
