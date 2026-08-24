#!/usr/bin/env bash
#
# Build the release and rsync it into the download directory on the server.
#
# Nothing goes through ~/dev/www/dystroy: that tree is a per-machine mirror of
# the whole site, so pushing it from one machine republishes stale copies of
# whatever another machine deployed. Each project sends its own subtree.
#
# Machine-specific settings live in build-scripts/_local.sh (gitignored):
#   BROOT_DEPLOY_TARGET  (required) rsync destination of the download directory

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

[[ -n ${BROOT_DEPLOY_TARGET:-} ]] || die "BROOT_DEPLOY_TARGET is not set — configure it in build-scripts/_local.sh"
# A mistyped destination would spray the binaries over the site itself.
[[ $BROOT_DEPLOY_TARGET == *:*/broot/download ]] \
    || die "BROOT_DEPLOY_TARGET should end in /broot/download, got '$BROOT_DEPLOY_TARGET'"

# build the release zip (and the build/ directory)
"$here/release.sh"

version=$(broot_version)

h1 "Deploying $version to $BROOT_DEPLOY_TARGET"
# Everything must be world-readable to be served; -a then carries the modes over.
# rsync's --chmod=D...,F... syntax isn't an option: macOS ships openrsync, which
# only takes a plain mode.
chmod -R a+rX build "broot_$version.zip"
# No --delete: the zips of previous versions stay downloadable.
rsync -av build/ "$BROOT_DEPLOY_TARGET/"
rsync -av "broot_$version.zip" "$BROOT_DEPLOY_TARGET/"
ok "deployed $version"
