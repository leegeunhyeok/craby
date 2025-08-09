#!/bin/bash

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0-alpha.1"
    exit 1
fi

# Get the current version from the package.json
CURRENT_VERSION=$(jq -r '.version' packages/cli/package.json)
if [ "$CURRENT_VERSION" == "$VERSION" ]; then
    echo "Version $VERSION is already set"
    exit 0
fi

echo "Updating version to $CURRENT_VERSION -> $VERSION"

# Update the version in the package.json
jq --arg version "$VERSION" '.version = $version' packages/cli/package.json > tmp.json && mv tmp.json packages/cli/package.json

# Commit the changes
git add packages/cli/package.json
git commit -m "$VERSION"
