#! /bin/bash

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0-alpha.1"
    exit 1
fi

BRANCH_NAME="release/cargo-v$VERSION"
ORIGIN_BRANCH=$(git branch --show-current)

function restore() {
  git switch $ORIGIN_BRANCH
}

(
  trap restore EXIT

  git switch -c $BRANCH_NAME
  git push -u origin $BRANCH_NAME

  cargo release version $VERSION --execute
  git add -A
  git commit -m "chore: cargo release $VERSION"

  cargo release --workspace --execute
)
