#!/bin/sh

# generate the docs for github pages.
# https://dev.to/deciduously/prepare-your-rust-api-docs-for-github-pages-2n5i

cargo doc --no-deps
rm -rf ./docs
# If you use this script, change the name of the crate here.
echo "<meta http-equiv=\"refresh\" content=\"0; url=hnefatafl\">" > target/doc/index.html
cp -r target/doc ./docs
