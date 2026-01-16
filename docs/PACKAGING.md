# Debian and Arch Packaging

## Overview

The Makefile includes targets for building and managing Debian (.deb) and Arch Linux packages for the SIL ecosystem.

### Prerequisites

**For Debian packages:**
- Rust toolchain
- `cargo-deb`: `cargo install cargo-deb`

**For Arch packages:**
- Arch Linux or compatible system
- `makepkg` (part of base-devel)
- `pacman`

## Debian Package Targets

### Build all .deb packages

```bash
make deb-build
```

Builds .deb packages for:
- lis-cli
- lis-format
- lis-runtime
- lis-api
- sil-ecosystem

Packages are placed in `target/debian/`

### Build single .deb package

```bash
make deb-build-single PKG=lis-cli
```

### Install .deb packages

```bash
make deb-install
```

Requires sudo. Installs all built packages.

### Clean .deb artifacts

```bash
make deb-clean
```

## Arch Linux Package Targets

### Build all Arch packages

```bash
make arch-build
```

Builds packages from `packaging/aur/*/PKGBUILD`

### Build single Arch package

```bash
make arch-build-single PKG=sil-ecosystem
```

### Install Arch package

```bash
make arch-install PKG=sil-ecosystem
```

Requires sudo and a built `.pkg.tar.zst` file.

### Prepare for AUR submission

```bash
make arch-submit-aur
```

Generates `.SRCINFO` files and prepares for pushing to AUR.

### Clean Arch artifacts

```bash
make arch-clean
```

## Interactive Package Builder

Run the automated builder script:

```bash
./script/package-builder.sh
```

Features:
- Interactive menu
- Dependency checking
- Multi-package building
- GitHub integration
- Version management

## Workflow Example

### 1. Update Version

Edit `sil-ecosystem/Cargo.toml`:
```toml
[package]
version = "2026.1.16"
```

### 2. Build Packages

```bash
# Build all packages
make deb-build
make arch-build

# Or use the interactive script
./script/package-builder.sh
```

### 3. Test Locally

```bash
# Debian (Ubuntu/Debian systems)
make deb-install

# Arch
make arch-install PKG=sil-ecosystem
```

### 4. Create GitHub Release

```bash
git tag v2026.1.16
git push origin v2026.1.16
```

### 5. Upload Artifacts

Add built packages to GitHub release:
- `target/debian/*.deb`
- `packaging/aur/*/sil-ecosystem-*.pkg.tar.zst`

### 6. Publish to AUR (for Arch)

```bash
make arch-submit-aur

# Then push to AUR
cd packaging/aur/sil-ecosystem
git push aur master
```

## Package Dependencies

### Debian

Specified in `Cargo.toml` `[package.metadata.deb]` section:

```toml
[package.metadata.deb]
depends = [
  "lis-cli (>= 2026.1.16)",
  "lis-format (>= 2026.1.16)",
  "lis-runtime (>= 2026.1.16)",
  "lis-api (>= 2026.1.16)"
]
```

### Arch

Specified in `packaging/aur/*/PKGBUILD`:

```bash
depends=(
  'lis-cli'
  'lis-format'
  'lis-runtime'
  'lis-api'
)
```

## Troubleshooting

### cargo-deb not found

```bash
cargo install cargo-deb
```

### makepkg not found

Requires Arch Linux or compatible. On other systems, use WSL2 or Docker:

```bash
docker run --rm -v $(pwd):/build archlinux:latest makepkg -C /build/packaging/aur/sil-ecosystem
```

### Version mismatch

The Makefile extracts version from `sil-ecosystem/Cargo.toml`. Ensure all `Cargo.toml` files have consistent versions.

## Release Checklist

- [ ] Update version in all `Cargo.toml` files
- [ ] Run `make deb-build`
- [ ] Run `make arch-build`
- [ ] Test installations locally
- [ ] Create GitHub release with tag
- [ ] Upload .deb and .pkg.tar.zst files
- [ ] Update AUR for Arch packages
- [ ] Update Homebrew tap
