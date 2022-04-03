# extract the version from the Cargo.toml file
version=$(sed 's/^version = "\([^\"]*\)"/\1/;t;d' Cargo.toml | head -1)

echo "$version"
