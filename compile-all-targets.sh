# WARNING: This script is NOT meant for normal installation, it's dedicated
# to the compilation of all supported targets. This is a long process and
# it involves specialized toolchains.
# For usual compilation do
#     cargo build --release
# or read all possible installation solutions on
# https://dystroy.org/broot/documentation/installation/

version=$(sed 's/version = "\([0-9.]\{1,\}\)"/\1/;t;d' Cargo.toml | head -1)
echo "=== Compilation of all targets for broot $version ==="
 
# clean previous build
echo "cleaning build"
rm -rf build
mkdir build

# build the linux version
echo "compiling the linux version"
cargo build --release
strip target/release/broot
mkdir build/x86_64-linux/
cp target/release/broot build/x86_64-linux/

# find and copy the completion scripts
# (they're built as part of the normal compilation)
echo "copying completion scripts"
mkdir build/completion
cp "$(broot -c ":gi;release;:focus;broot.bash;:parent;:pp" target)/"* build/completion

# build the windows version
# use cargo cross
echo "compiling the Windows version"
cross build --target x86_64-pc-windows-gnu --release
mkdir build/x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/broot.exe build/x86_64-pc-windows-gnu/

# build the Raspberry version
# use cargo cross
cross build --target armv7-unknown-linux-gnueabihf --release
mkdir build/armv7-unknown-linux-gnueabihf
cp target/armv7-unknown-linux-gnueabihf/release/broot build/armv7-unknown-linux-gnueabihf/

# build a musl version
cross build --release --target x86_64-unknown-linux-musl
mkdir build/x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/broot build/x86_64-unknown-linux-musl
