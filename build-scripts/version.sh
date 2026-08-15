#!/usr/bin/env bash
# Print the broot version, read from the main Cargo.toml.
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"
broot_version
