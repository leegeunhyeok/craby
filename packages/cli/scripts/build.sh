#!/bin/bash

set -e

if [ -z "$RUST_TARGET" ]; then
  echo "RUST_TARGET is not set. Building for current platform (DEBUG MODE)"
  yarn napi build --platform --esm --output-dir napi
else
  echo "Building for $RUST_TARGET"
  rustup target add $RUST_TARGET
  yarn napi build --platform --target $RUST_TARGET --esm --output-dir napi --release
fi

yarn tsup

echo "Done!"
