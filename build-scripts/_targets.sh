# _targets.sh — the list of release targets and the per-target build engine,
# shared by build-all-targets.sh and build-target.sh.
#
# Not meant to be run directly; source it:
#   source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_targets.sh"

source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/_common.sh"

NAME=broot

# Each target builds into its own cargo target dir under here, so native,
# Docker (cross) and zig builds never share host artifacts. This replaces the
# old global `cargo clean` between targets (and keeps incremental caching).
CACHE=.build-cache

# How to build the macOS binary:
#   auto   -> native when running on macOS, else skip
#   native -> native cargo build (must run on a Mac; uses the real Apple linker,
#             which fixes the duplicate-libiconv crash of issue #1194)
#   zig    -> Docker + cargo-zigbuild (currently produces a broken binary, #1194)
#   skip   -> don't build the macOS binary
DARWIN_METHOD=${DARWIN_METHOD:-auto}
if [[ $DARWIN_METHOD == auto ]]; then
    if [[ $(host_os) == Darwin ]]; then DARWIN_METHOD=native; else DARWIN_METHOD=skip; fi
fi

# The release matrix, one "label|triple|tool|features|host" row per line.
#   tool: cross | zig | zigmac | ndk | native
#   host: optional — restrict to a host OS (uname -s, e.g. Linux, Darwin). Empty = any.
#   Lines that are blank or start with # are ignored — comment a row out to disable a target.
# macOS is not listed here; it's handled separately via DARWIN_METHOD.
_matrix_rows() {
    while IFS= read -r line; do
        [[ $line =~ ^[[:space:]]*(#|$) ]] && continue
        printf '%s\n' "$line"
    done <<'EOF'
x86-64 GLIBC|x86_64-unknown-linux-gnu|zig|clipboard,sixel|
MUSL|x86_64-unknown-linux-musl|zig|sixel|
ARM 32|armv7-unknown-linux-gnueabihf|zig|sixel|
ARM 32 MUSL|armv7-unknown-linux-musleabi|cross|sixel|Linux
ARM 64|aarch64-unknown-linux-gnu|zig|sixel|
ARM 64 MUSL|aarch64-unknown-linux-musl|zig|sixel|
Windows|x86_64-pc-windows-gnu|zig|clipboard,sixel|
GLIBC 2.28|x86_64-unknown-linux-gnu.2.28|zig|sixel|
Android x86_64|x86_64-linux-android|ndk|clipboard,sixel|
# NetBSD/amd64|x86_64-unknown-netbsd|cross|sixel|Linux   uncomment to build NetBSD (zig can't target it; needs a Linux host with working cross)
EOF
}

# The macOS row for the current DARWIN_METHOD (empty when skipped).
_darwin_row() {
    case $DARWIN_METHOD in
        native) echo "macOS|aarch64-apple-darwin|native|sixel" ;;
        zig)    echo "macOS|aarch64-apple-darwin|zigmac|sixel" ;;
        skip)   : ;;
    esac
}

# Targets to build on THIS host: matrix rows whose host matches (or is empty),
# plus the macOS row per DARWIN_METHOD. Emits "label|triple|tool|features".
all_targets() {
    local host label triple tool features want
    host=$(host_os)
    _matrix_rows | while IFS='|' read -r label triple tool features want; do
        [[ -z $want || $want == "$host" ]] || continue
        printf '%s|%s|%s|%s\n' "$label" "$triple" "$tool" "$features"
    done
    _darwin_row
}

# Every target that belongs in a full release, regardless of host — used by
# release.sh to check completeness. Commented rows (e.g. NetBSD) are excluded;
# macOS is always expected. Emits "label|triple|tool|features".
all_release_targets() {
    _matrix_rows | while IFS='|' read -r label triple tool features want; do
        printf '%s|%s|%s|%s\n' "$label" "$triple" "$tool" "$features"
    done
    echo "macOS|aarch64-apple-darwin|native|sixel"
}

# The binary path inside build/ for a triple: build/<triple>/broot[.exe]
target_binary() { # target_binary <triple>
    local triple=$1 exe=$NAME
    [[ $triple == *windows* ]] && exe="$NAME.exe"
    printf 'build/%s/%s\n' "$triple" "$exe"
}

# Check a freshly built binary exists, and (on macOS) that it has no duplicate
# linked dylib — the signature of issue #1194.
verify_binary() { # verify_binary <path> <triple>
    local bin=$1 triple=$2
    [[ -f $bin ]] || die "expected binary not produced: $bin"
    have file && info "$(file -b "$bin")"
    if [[ $triple == *-apple-darwin ]] && have otool; then
        local dups
        dups=$(otool -L "$bin" | sed -n 's/^[[:space:]]\{1,\}\([^ ]*\).*/\1/p' | sort | uniq -d)
        [[ -z $dups ]] || die "duplicate linked dylib(s) — this is issue #1194:"$'\n'"$dups"
        ok "no duplicate dylibs"
    fi
}

# Build one target described by a "label|triple|tool|features" row and copy the
# resulting binary into build/<triple>/.
build_row() { # build_row "<label>|<triple>|<tool>|<features>"
    local label triple tool features
    IFS='|' read -r label triple tool features <<< "$1"

    local tdir="$CACHE/$triple" exe=$NAME bin feat=()
    [[ -n $features ]] && feat=(--features "$features")
    [[ $triple == *windows* ]] && exe="$NAME.exe"

    h2 "$label   (target=$triple, tool=$tool, features='${features:-none}')"
    case $tool in
        cross)
            need cross "cargo install cross"
            ensure_container_engine
            RUSTFLAGS="" cross build --release --target "$triple" \
                --target-dir "$tdir" ${feat[@]+"${feat[@]}"}
            bin="$tdir/$triple/release/$exe" ;;
        native)
            need cargo "install the Rust toolchain — https://rustup.rs"
            cargo build --release --target "$triple" \
                --target-dir "$tdir" ${feat[@]+"${feat[@]}"}
            bin="$tdir/$triple/release/$exe" ;;
        zig)
            need cargo-zigbuild "cargo install cargo-zigbuild  (and install zig)"
            rustup target add "${triple%%.*}" >/dev/null   # ensure rust-std for the target (idempotent)
            cargo zigbuild --release --target "$triple" \
                --target-dir "$tdir" ${feat[@]+"${feat[@]}"}
            bin="$tdir/${triple%%.*}/release/$exe" ;; # zigbuild drops any .glibc suffix from the dir name
        zigmac)
            ensure_container_engine
            warn "building macOS with zig; the output may be broken, see #1194"
            "${CROSS_CONTAINER_ENGINE:-docker}" run --rm -v "$PWD:/io" -w /io ghcr.io/rust-cross/cargo-zigbuild \
                cargo zigbuild --release --target "$triple" --target-dir "$tdir"
            bin="$tdir/$triple/release/$exe" ;;
        ndk)
            need cargo-ndk "cargo install cargo-ndk"
            # honour an explicit ANDROID_NDK_HOME (e.g. on Linux); else fall back to a brew-installed NDK
            if [[ -z ${ANDROID_NDK_HOME:-} ]] && have brew; then
                ANDROID_NDK_HOME="$(brew --prefix)/share/android-ndk"
            fi
            [[ -d ${ANDROID_NDK_HOME:-} ]] || die "ANDROID_NDK_HOME is not set to a valid NDK (got '${ANDROID_NDK_HOME:-}'); install one (macOS: brew install --cask android-ndk)"
            export ANDROID_NDK_HOME
            rustup target add "$triple" >/dev/null   # ensure rust-std for the target (idempotent)
            cargo ndk -t "${triple%-linux-android}" build --release \
                --target-dir "$tdir" ${feat[@]+"${feat[@]}"}
            bin="$tdir/$triple/release/$exe" ;;
        *) die "unknown build tool '$tool' for target $label" ;;
    esac

    verify_binary "$bin" "$triple"
    mkdir -p "build/$triple"
    cp "$bin" "build/$triple/"
    ok "$label → build/$triple/$exe"
}
