# This script compiles broot for the local system
#
# After compilation, broot can be found in target/release
#
# If you're not a developer but just want to install broot to use it,
# you'll probably prefer one of the options listed at
#   https://dystroy.org/broot/install
#
# Depending on your system, it's possible one of the 'features'
# won't compile for you. You may remove them (see features.md)
#
# The line below can be safely executed on systems which don't
# support sh scripts.

cargo build --release --features "clipboard"

