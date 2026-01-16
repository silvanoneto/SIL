#!/bin/bash
# Script to prepare Homebrew release for sil-ecosystem

set -e

VERSION="2026.1.16"
REPO="silvanoneto/SIL"
ARCHIVE_NAME="v${VERSION}.tar.gz"
GITHUB_URL="https://github.com/${REPO}/archive/refs/tags/${ARCHIVE_NAME}"

echo "🍺 Preparing Homebrew release for sil-ecosystem v${VERSION}"
echo ""

# Step 1: Download and calculate SHA256
echo "📥 Downloading source archive..."
curl -L -o "/tmp/${ARCHIVE_NAME}" "${GITHUB_URL}"

echo "🔐 Calculating SHA256..."
SOURCE_SHA256=$(sha256sum "/tmp/${ARCHIVE_NAME}" | awk '{print $1}')
echo "Source SHA256: ${SOURCE_SHA256}"
echo ""

# Step 2: Build bottles for different architectures
echo "🔨 Building bottles..."
echo ""

# For ARM64 (Apple Silicon)
echo "Building for ARM64 (Apple Silicon)..."
brew uninstall -f sil-ecosystem 2>/dev/null || true
brew install --build-bottle /Users/silvis/Public/XXI/packaging/homebrew/sil-ecosystem.rb
BOTTLE_ARM64=$(brew bottle --json sil-ecosystem | grep -o '"arm64_sonoma":.*' | cut -d'"' -f4)
echo "ARM64 SHA256: ${BOTTLE_ARM64}"
echo ""

# For AMD64 (Intel)
echo "Note: To build AMD64 bottle, run on an Intel Mac or use cross-compilation"
echo "AMD64 SHA256: PLACEHOLDER_AMD64_SHA256"
echo ""

# Step 3: Update formula with SHA256s
echo "📝 Updating sil-ecosystem.rb with SHA256s..."
sed -i.bak \
  -e "s/sha256 \"PLACEHOLDER_SHA256\"/sha256 \"${SOURCE_SHA256}\"/g" \
  -e "s/arm64_sonoma: \"PLACEHOLDER_ARM64_SHA256\"/arm64_sonoma: \"${BOTTLE_ARM64}\"/g" \
  /Users/silvis/Public/XXI/packaging/homebrew/sil-ecosystem.rb

rm /Users/silvis/Public/XXI/packaging/homebrew/sil-ecosystem.rb.bak

echo "✅ Formula updated!"
echo ""
echo "📋 Next steps:"
echo "1. Create a GitHub release for tag v${VERSION}"
echo "2. Upload the bottled formula to the release"
echo "3. Publish to a Homebrew tap (e.g., homebrew-sil)"
echo ""
