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
   `<version>-<commit>`.

2. On one machine, assemble and package:

       ./build-scripts/release.sh

   Fetches the staged artifacts, checks every target is present, verifies each
   binary, and produces `broot_<version>.zip`.

3. Publish:

       ./build-scripts/deploy.sh

Staging server and other machine-local settings live in `build-scripts/_local.sh`
(gitignored). Without it, `release.sh` just builds locally on a single host.

More specifically, those two vars are needed (here with example values):

```bash
BROOT_STAGE_HOST=dystroy.org
BROOT_STAGE_DIR=staging/broot-staging
```

## Other scripts

- `build-target.sh <filter>` — build one target (`--list` to see them), e.g.
  `./build-scripts/build-target.sh aarch64-apple-darwin`
- `build.sh` — quick local `cargo build --release --features clipboard`
- `win-deploy.sh`, `termux-deploy.sh` — build and push a single Windows / Android binary
- `fix-win-toolchain.sh` — Linux-only mingw fixup (unused with the current setup)
- `_common.sh`, `_targets.sh` — sourced libraries, not run directly
