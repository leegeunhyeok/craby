#!/bin/bash

set -e

function to_cjs() {
  mv napi/index.js napi/index.cjs
  mv napi/index.d.ts napi/index.d.cts
}

if [ -z "$RUST_TARGET" ]; then
  echo "RUST_TARGET is not set. Building for current platform (DEBUG MODE)"
  yarn napi build --platform napi
  to_cjs
else
  echo "Building for $RUST_TARGET"
  rustup target add $RUST_TARGET
  yarn napi build --platform --target $RUST_TARGET --release napi
  to_cjs
fi

yarn tsup

echo "Done!"
