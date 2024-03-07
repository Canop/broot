version=$(./version.sh)
mkdir -p releases/broot_${version}

cd build
# make one zip file for each architecture 
# cargo binstall needs that
# see default format https://github.com/cargo-bins/cargo-binstall/blob/main/SUPPORT.md#defaults
find . -maxdepth 1 -type d | grep -v -e "resources" -e "completion" -e "default-conf" -e '^\.$' | cut -c 3- |xargs -I {} zip -rj ../releases/broot_${version}/broot-{}-v${version}.zip {}
cd -
