#!/bin/bash

set -e

if [ -z "$RUST_TARGET" ]; then
  echo "RUST_TARGET is not set"
  exit 1
fi

echo "Building for $RUST_TARGET"
rustup target add $RUST_TARGET
yarn napi build --platform --target $RUST_TARGET --release napi
yarn tsup

echo "Done!"
