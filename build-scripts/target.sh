#!/usr/bin/env bash
# Print the host target triple, as reported by rustc.
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"
host_target
