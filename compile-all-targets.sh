# WARNING: This script is NOT meant for normal installation, it's dedicated
# to the compilation of all supported targets, from a linux machine.
# This is a long process and it involves specialized toolchains.
# For usual compilation do
#     cargo build --release
# or read all possible installation solutions on
# https://dystroy.org/broot/documentation/installation/

H1="\n\e[30;104;1m\e[2K\n\e[A" # style first header
H2="\n\e[30;104m\e[1K\n\e[A" # style second header
EH="\e[00m\n\e[2K" # end header

version=$(sed 's/version = "\([0-9.]\{1,\}\(-[a-z]\+\)\?\)"/\1/;t;d' Cargo.toml | head -1)
echo -e "${H1}Compilation of all targets for broot $version${EH}"
 
# clean previous build
rm -rf build
mkdir build
echo "   build cleaned"

# build the linux version
echo -e "${H2}Compiling the linux version${EH}"
cargo build --release --features "clipboard"
strip target/release/broot
mkdir build/x86_64-linux/
cp target/release/broot build/x86_64-linux/

# find and copy the completion scripts
# (they're built as part of the normal compilation so must come after the linux version)
echo -e "${H2}copying completion scripts${EH}"
mkdir build/completion
cp "$(broot -c ":gi;release;:focus;broot.bash;:parent;:pp" target)/"* build/completion
echo "   Done"

# copy the default conf
echo -e "${H2}copying default configuration${EH}"
cp resources/default-conf.hjson build
echo "   Done"

# add the resource (the icons font)
echo -e "${H2}copying vscode-icon font${EH}"
mkdir build/resources
cp resources/icons/vscode/vscode.ttf build/resources
echo "the font file comes from https://github.com/vscode-icons/vscode-icons/ and is licensed as MIT" > build/resources/README.md
echo "   Done"

# build the windows version
# use cargo cross
echo -e "${H2}Compiling the Windows version${EH}"
cross build --target x86_64-pc-windows-gnu --release --features "clipboard"
mkdir build/x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/broot.exe build/x86_64-pc-windows-gnu/

# build the Raspberry version
# use cargo cross
echo -e "${H2}Compiling the Raspberry version${EH}"
cross build --target armv7-unknown-linux-gnueabihf --release
mkdir build/armv7-unknown-linux-gnueabihf
cp target/armv7-unknown-linux-gnueabihf/release/broot build/armv7-unknown-linux-gnueabihf/

# build the Android version
# use cargo cross
echo -e "${H2}Compiling the Android version${EH}"
cross build --target aarch64-linux-android --release --features "clipboard"
mkdir build/aarch64-linux-android
cp target/aarch64-linux-android/release/broot build/aarch64-linux-android/

# build a musl version
echo -e "${H2}Compiling the MUSL version${EH}"
cross build --release --target x86_64-unknown-linux-musl
mkdir build/x86_64-unknown-linux-musl
cp target/x86_64-unknown-linux-musl/release/broot build/x86_64-unknown-linux-musl
