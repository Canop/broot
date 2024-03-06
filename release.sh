# build a new release of broot
# This isn't used for normal compilation (see https://dystroy.org/broot for instruction)
# but for the building of the official releases
version=$(./version.sh)

echo "Building release $version"

# make the build directory and compile for all targets
./compile-all-targets.sh

# add the readme and changelog in the build directory
echo "This is broot. More info and installation instructions on https://dystroy.org/broot" > build/README.md
cp CHANGELOG.md build

# add the man page and fix its date and version
cp man/page build/broot.1
sed -i "s/#version/$version/g" build/broot.1
sed -i "s/#date/$(date +'%Y\/%m\/%d')/g" build/broot.1

# publish version number
echo "$version" > build/version

# prepare the release archive
rm broot_*.zip
cd build
find . -type d -empty -delete
zip -r "../broot_$version.zip" *
rm broot*.zip
# make one zip file for each architecture (cargo binstall needs that)
find . -type d -name "broot_*" -exec zip -rj "{}.zip" "{}" \; 
cd -

# copy it to releases folder
mkdir -p releases/broot_$version
cp "broot_$version.zip" releases/broot_$version
cp build/broot_*.zip releases/broot_$version
rm broot*.zip
rm build/broot_*.zip
