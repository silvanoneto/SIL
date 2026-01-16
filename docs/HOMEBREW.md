# Homebrew Release Process for sil-ecosystem

## Overview

This guide explains how to package and release the sil-ecosystem as a Homebrew formula.

## Prerequisites

- Homebrew installed
- Rust and Cargo installed
- Git access to the repository
- GitHub release rights

## Steps

### 1. Create a GitHub Release

First, create a GitHub release with the appropriate tag:

```bash
git tag v2026.1.16
git push origin v2026.1.16
```

Then create a release on GitHub with release notes.

### 2. Generate SHA256 for Source Archive

The formula needs the SHA256 hash of the source archive:

```bash
VERSION="2026.1.16"
curl -L https://github.com/silvanoneto/SIL/archive/refs/tags/v${VERSION}.tar.gz | sha256sum
```

### 3. Build Bottles (Precompiled Binaries)

To make installation faster, build bottles for different architectures:

```bash
# On Apple Silicon Mac (ARM64)
brew install --build-bottle /path/to/packaging/homebrew/sil-ecosystem.rb
brew bottle sil-ecosystem

# On Intel Mac (AMD64)
# Run the same commands on an Intel machine, or use cross-compilation
```

This generates `.bottle.tar.gz` files with SHA256 hashes.

### 4. Update the Formula

Update `packaging/homebrew/sil-ecosystem.rb` with:
- Source SHA256
- Bottle SHA256s for each architecture

```ruby
url "https://github.com/silvanoneto/SIL/archive/refs/tags/v2026.1.16.tar.gz"
sha256 "CALCULATED_SHA256"

bottle do
  root_url "https://github.com/silvanoneto/SIL/releases/download/v2026.1.16"
  sha256 cellar: :any, arm64_sonoma: "ARM64_SHA256"
  sha256 cellar: :any, amd64_sonoma: "AMD64_SHA256"
end
```

### 5. Create a Homebrew Tap

To distribute the formula, create a tap repository:

```bash
# Create a public GitHub repository
# Example: https://github.com/silvanoneto/homebrew-sil

# Clone the tap
git clone https://github.com/silvanoneto/homebrew-sil.git
cd homebrew-sil

# Copy the formula
mkdir -p Formula
cp ../XXI/packaging/homebrew/sil-ecosystem.rb Formula/

# Commit and push
git add Formula/sil-ecosystem.rb
git commit -m "Add sil-ecosystem formula v2026.1.16"
git push origin main
```

### 6. Installation for Users

Once the tap is published, users can install with:

```bash
# Add the tap
brew tap silvanoneto/sil

# Install the package
brew install sil-ecosystem

# Or directly without adding tap
brew install silvanoneto/sil/sil-ecosystem
```

## Automated Release Script

Use the provided script for semi-automated release:

```bash
./script/brew-release.sh
```

This script:
1. Downloads the source archive
2. Calculates SHA256
3. Builds bottles
4. Updates the formula

## Troubleshooting

### Formula Not Found

```bash
brew tap-new silvanoneto/sil
brew create --set-version 2026.1.16 https://github.com/silvanoneto/SIL/archive/refs/tags/v2026.1.16.tar.gz
```

### Build Failures

Check the build log:

```bash
brew install --build-bottle -s sil-ecosystem
```

The `-s` flag shows the source code and build output.

### Bottle Architecture Mismatch

Ensure you're building on the correct architecture:

```bash
uname -m  # Shows current architecture
```

## References

- [Homebrew Formula Documentation](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Tap Documentation](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [Bottle Documentation](https://docs.brew.sh/Bottles)
