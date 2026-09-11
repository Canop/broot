#!/usr/bin/env bash

set -Eeuo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

command -v ddoc >/dev/null || { echo "ddoc not found — see https://dystroy.org/ddoc" >&2; exit 1; }

ddoc

# deploy directly on the server: going through ~/dev/www/dystroy would republish
# that machine's stale copy of every other project
chmod -R a+rX site
rsync -av site/ dys@dystroy.org:prod/www.dystroy.org/broot/
