#!/bin/bash
# Bump RC version for all crates
# Usage: ./scripts/bump-rc-version.sh

set -e

# Extract current version from medcodes Cargo.toml
CURRENT_VERSION=$(grep '^version = ' crates/medcodes/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

echo "Current version: $CURRENT_VERSION"

# Parse rc number
if [[ $CURRENT_VERSION =~ ^([0-9]+\.[0-9]+\.[0-9]+)-rc\.([0-9]+)$ ]]; then
    BASE_VERSION="${BASH_REMATCH[1]}"
    RC_NUM="${BASH_REMATCH[2]}"
    NEW_RC_NUM=$((RC_NUM + 1))
    NEW_VERSION="${BASE_VERSION}-rc.${NEW_RC_NUM}"
else
    echo "Error: Could not parse version $CURRENT_VERSION"
    exit 1
fi

echo "New version: $NEW_VERSION"

# Update all Cargo.toml files
sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" crates/medcodes/Cargo.toml
sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" crates/mimic-etl/Cargo.toml
sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" crates/clinical-tasks/Cargo.toml

# Update medcodes dependency versions
sed -i "s/medcodes = { version = \"$CURRENT_VERSION\"/medcodes = { version = \"$NEW_VERSION\"/" crates/mimic-etl/Cargo.toml
sed -i "s/medcodes = { version = \"$CURRENT_VERSION\"/medcodes = { version = \"$NEW_VERSION\"/" crates/clinical-tasks/Cargo.toml

echo "✓ Updated Cargo.toml files"

# Commit changes
git add crates/*/Cargo.toml
git commit -m "chore: bump to $NEW_VERSION for re-release" || true

# Create and push new tag
git tag -d "v$CURRENT_VERSION" 2>/dev/null || true
git push origin ":v$CURRENT_VERSION" 2>/dev/null || true
git tag "v$NEW_VERSION"
git push origin "v$NEW_VERSION"

echo "✓ Tagged and pushed v$NEW_VERSION"
echo "Release workflow will now trigger with new version"
