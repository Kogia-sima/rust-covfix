#!/bin/bash

set -ex

if [ "$TRAVIS_RUST_VERSION" = "nightly" ] && [ -z "$TRAVIS_TAG" ]; then
  export CARGO_INCREMENTAL=0
  export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Zno-landing-pads"
  export CARGO_OPTIONS="--all-features"

  wget https://github.com/mozilla/grcov/releases/download/v0.5.7/grcov-linux-x86_64.tar.bz2
  tar xvf grcov-linux-x86_64.tar.bz2
else
  export CARGO_OPTIONS="--features backtrace"
fi

cargo build $CARGO_OPTIONS
cargo test $CARGO_OPTIONS

if [ "$TRAVIS_RUST_VERSION" = "nightly" ] && [ -z "$TRAVIS_TAG" ]; then
  zip -0 ccov.zip `find . \( -name "rust_covfix*.gc*" -o -name "test-*.gc*" \) -print`
  ./grcov ccov.zip -s . -t lcov --llvm --branch --ignore-not-existing --ignore "/*" --ignore "tests/*" -o lcov.info
  ./target/debug/rust-covfix lcov.info -o lcov.info
  bash <(curl -s https://codecov.io/bash) -f lcov.info
fi
