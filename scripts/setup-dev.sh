#!/bin/bash
# Development environment setup script
# Usage: ./scripts/setup-dev.sh

set -e

echo "🚀 Setting up clinical-rs development environment..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Must be run from the clinical-rs root directory"
    exit 1
fi

# Install pre-commit if not present
if ! command -v pre-commit &> /dev/null; then
    echo "📦 Installing pre-commit..."
    pip install pre-commit
fi

# Install cargo-audit if not present
if ! command -v cargo-audit &> /dev/null; then
    echo "📦 Installing cargo-audit..."
    cargo install cargo-audit --locked
fi

# Install cargo-deny if not present
if ! command -v cargo-deny &> /dev/null; then
    echo "📦 Installing cargo-deny..."
    cargo install cargo-deny --locked
fi

# Install cargo-nextest if not present
if ! command -v cargo-nextest &> /dev/null; then
    echo "📦 Installing cargo-nextest..."
    cargo install cargo-nextest --locked
fi

# Install git-cliff if not present
if ! command -v git-cliff &> /dev/null; then
    echo "📦 Installing git-cliff..."
    cargo install git-cliff@^2.12 --locked
fi

# Install pre-commit hooks
echo "🔧 Installing pre-commit hooks..."
pre-commit install

# Install git hooks for conventional commits
echo "🔧 Setting up conventional commit hooks..."
cat > .git/hooks/commit-msg << 'EOF'
#!/bin/bash
# Enforce conventional commit format
commit_regex='^(feat|fix|docs|style|refactor|perf|test|ci|chore|revert)(\(.+\))?: .{1,50}'

if ! grep -qE "$commit_regex" "$1"; then
    echo "❌ Invalid commit message format!"
    echo ""
    echo "Commit message must follow conventional commits:"
    echo "  feat: add new feature"
    echo "  fix: resolve bug"
    echo "  docs: update documentation"
    echo "  chore: maintenance task"
    echo ""
    echo "See: https://www.conventionalcommits.org/"
    exit 1
fi
EOF

chmod +x .git/hooks/commit-msg

# Verify Rust toolchain
echo "🔧 Verifying Rust toolchain..."
if ! rustc --version | grep -q "1.94.0"; then
    echo "⚠️  Warning: Expected Rust 1.94.0, found $(rustc --version)"
    echo "   Consider using rust-toolchain.toml to pin the version"
fi

# Run initial checks
echo "🧪 Running initial checks..."
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check

echo ""
echo "✅ Development environment setup complete!"
echo ""
echo "📋 Available commands:"
echo "  pre-commit run --all-files     # Run all hooks on all files"
echo "  cargo fmt                       # Format code"
echo "  cargo clippy                     # Run clippy"
echo "  cargo nextest run                # Run tests"
echo "  cargo audit                      # Security audit"
echo "  cargo deny check                 # License/security check"
echo ""
echo "🎯 Ready to develop clinical-rs!"
