#!/bin/bash
# Creto Enablement Layer - Sibling Repository Symlinks Setup
#
# This script creates symlinks to the sibling Creto repositories
# that the Enablement Layer depends on for development.
#
# Prerequisites:
#   - Clone all Creto repos in the same parent directory
#   - Directory structure expected:
#     GitHub/
#     ├── Enablement/     (this repo)
#     ├── Sovereign/      (Platform Layer)
#     ├── Authorization/  (creto-authz)
#     ├── Memory/         (creto-memory)
#     ├── Storage/        (creto-storage)
#     └── Vault/          (creto-vault)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEPS_DIR="$PROJECT_ROOT/deps"

echo "Setting up Creto sibling repository symlinks..."
echo "Project root: $PROJECT_ROOT"
echo ""

# Create deps directory
mkdir -p "$DEPS_DIR"

# Define sibling repos and their symlink names
declare -A REPOS=(
    ["sovereign"]="../../Sovereign"
    ["creto-authz"]="../../Authorization"
    ["creto-memory"]="../../Memory"
    ["creto-storage"]="../../Storage"
    ["creto-vault"]="../../Vault"
)

# Create symlinks
for name in "${!REPOS[@]}"; do
    target="${REPOS[$name]}"
    link="$DEPS_DIR/$name"

    # Check if target exists
    if [ -d "$PROJECT_ROOT/$target" ]; then
        # Remove existing symlink if present
        rm -f "$link"
        ln -s "$target" "$link"
        echo "✅ $name -> $target"
    else
        echo "❌ $name: Target not found ($target)"
        echo "   Please clone the repository to: $PROJECT_ROOT/$target"
    fi
done

echo ""
echo "=== Verification ==="

# Verify each symlink
ERRORS=0
for name in "${!REPOS[@]}"; do
    link="$DEPS_DIR/$name"
    if [ -L "$link" ] && [ -d "$link" ]; then
        count=$(ls "$link" 2>/dev/null | wc -l | tr -d ' ')
        echo "✅ $name: $count items accessible"
    else
        echo "❌ $name: Symlink broken or missing"
        ERRORS=$((ERRORS + 1))
    fi
done

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "✅ All symlinks configured successfully!"
    echo ""
    echo "Sibling repos available at:"
    echo "  deps/sovereign/src/       - Platform Layer (crypto, consensus, vault, authz)"
    echo "  deps/creto-authz/         - Authorization Layer"
    echo "  deps/creto-memory/crates/ - Memory Layer (38+ crates)"
    echo "  deps/creto-storage/       - Storage Layer"
    echo "  deps/creto-vault/crates/  - Vault Layer (secrets management)"
else
    echo "⚠️  $ERRORS symlinks failed. Check that all repos are cloned."
    exit 1
fi
