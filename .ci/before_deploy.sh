#!/usr/bin/env bash

set -ex

binary_name="$PROJECT_NAME"
package_name="$PROJECT_NAME-$TRAVIS_OS_NAME-x86_64"

cargo build --target "$TARGET" --release --verbose
mkdir -p "$package_name"
cp "target/$TARGET/release/$binary_name" "$package_name/"
strip "$package_name/$binary_name"

cp README.md "$package_name"
cp LICENSE "$package_name"

tar zcvf "${package_name}.tar.gz" "$package_name"
rm -rf "$package_name"

mkdir dist
mv "${package_name}.tar.gz" dist/
