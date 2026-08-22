# build-scripts

These scripts are only useful for building the distributed broot binaries hosted
on the deployment server. If you just want to install or build broot for
yourself, you don't need anything here — see <https://dystroy.org/broot/install>.

They may be run from anywhere; each resolves paths against the repo root.

## Building a release

No single machine can build every target (macOS needs a Mac, `armv7-musl` needs
Linux), so a release is built on both and assembled through a staging server.

1. Commit, then on **each** machine (Mac and Linux):

       ./build-scripts/build-all-targets.sh

   Builds this host's targets and pushes `build/` to the staging server, keyed by
   `<version>-<commit>`. Targets already staged under that key are skipped, so a
   re-run after a failure only builds what's missing; `--force` rebuilds them all.
   Any new commit changes the key, so everything is rebuilt.

2. On one machine, assemble and package:

       ./build-scripts/release.sh

   Fetches the staged artifacts, checks every target is present, verifies each
   binary, and produces `broot_<version>.zip`.

3. Publish:

       ./build-scripts/deploy.sh

Staging and deploy settings come from `build-scripts/_local.sh` (see below).
Without it, `release.sh` builds locally on a single host and `deploy.sh` won't run.

A dirty tree doesn't block either step, but both ask first: `build-all-targets.sh`
offers to stage it anyway (recorded in a `<version>-<commit>.dirty` marker beside
the staging dir, and already-staged targets are then rebuilt rather than reused),
and `release.sh` reports every host that staged uncommitted work before packaging.
Set `BROOT_YES=1` to answer yes to all of it without a terminal.

## Machine-local config (`_local.sh`)

`build-scripts/_local.sh` holds per-machine settings and is **gitignored**, so it
never travels through git — **recreate it on each machine** that builds or deploys.
It's sourced by `_common.sh`.

| Variable | Used by | Required | Meaning |
|----------|---------|----------|---------|
| `BROOT_STAGE_HOST` | build-all-targets.sh, release.sh | for staged releases | ssh host every machine can reach; enables push/fetch of a multi-host release. Unset ⇒ single-host local builds. |
| `BROOT_STAGE_DIR` | build-all-targets.sh, release.sh | no — default `broot-staging` | staging dir on the server, relative to your ssh login home (or absolute, with a leading `/`). |
| `BROOT_DOWNLOAD_DIR` | deploy.sh | yes, to deploy | dir the built `build/` and the zip are copied into. |
| `BROOT_DEPLOY_HOOK` | deploy.sh | no | command run after copying, to publish (e.g. a website deploy script). |
| `BROOT_VM_SHARED` | win-deploy.sh | no — default `~/dev/storage/vm/shared` | folder shared with the Windows VM. |
| `BROOT_PUB_DIR` | termux-deploy.sh | no — default `$BROOT_WWW_DIR/pub` | destination for the Android/Termux binary. |
| `BROOT_WWW_DIR` | termux-deploy.sh, convenience | no — default `~/dev/www/dystroy` | base path used to derive the others. |

Set only what a machine actually needs (e.g. `BROOT_VM_SHARED` only where you run
`win-deploy.sh`). A full example:

```bash
# build-scripts/_local.sh  — per machine, gitignored

# Staged multi-host release builds (set on both the Mac and the Linux box):
BROOT_STAGE_HOST=dystroy.org
BROOT_STAGE_DIR=staging/broot-staging        # relative to ssh home, or absolute

# Publishing, on whichever machine runs deploy.sh:
BROOT_WWW_DIR="$HOME/dev/www/dystroy"
BROOT_DOWNLOAD_DIR="$BROOT_WWW_DIR/broot/download"
BROOT_DEPLOY_HOOK="$BROOT_WWW_DIR/deploy.sh"

# Only if you use these on this machine:
# BROOT_VM_SHARED="$HOME/dev/storage/vm/shared"    # win-deploy.sh
# BROOT_PUB_DIR="$BROOT_WWW_DIR/pub"               # termux-deploy.sh
```

## Other scripts

- `build-target.sh <filter>` — build one target (`--list` to see them), e.g.
  `./build-scripts/build-target.sh aarch64-apple-darwin`
- `build.sh` — quick local `cargo build --release --features clipboard,sixel`
- `win-deploy.sh`, `termux-deploy.sh` — build and push a single Windows / Android binary
- `fix-win-toolchain.sh` — Linux-only mingw fixup (unused with the current setup)
- `_common.sh`, `_targets.sh` — sourced libraries, not run directly
