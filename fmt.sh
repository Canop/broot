# I consider the changes of cargo fmt as suggestions.
# So my practice is to pick them, or not, one by one, in meld.
# This script builds a formatted version of the source then
# opens the two folders in meld (destination being my original
# code)
rm -rf src-mine
cp -r src src-mine
cargo fmt
mv src src-fmt
mv src-mine src
meld src-fmt src
