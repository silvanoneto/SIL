class SilEcosystem < Formula
  desc "Meta package for SIL/LIS CLI tools"
  homepage "https://github.com/silvanoneto/SIL"
  url "https://github.com/silvanoneto/SIL/archive/refs/tags/v2026.1.16.tar.gz"
  sha256 "PLACEHOLDER_SHA256"  # Will be replaced with actual SHA256
  license "AGPL-3.0-only"

  bottle do
    root_url "https://github.com/silvanoneto/SIL/releases/download/v2026.1.16"
    sha256 cellar: :any,                 arm64_sonoma: "PLACEHOLDER_ARM64_SHA256"
    sha256 cellar: :any,                 amd64_sonoma: "PLACEHOLDER_AMD64_SHA256"
  end

  depends_on "rust" => :build
  depends_on "pkg-config" => :build

  def install
    # Build LIS CLI tools
    system "cargo", "build", "--release", 
           "--manifest-path", "lis-cli/Cargo.toml",
           "--locked"
    system "cargo", "build", "--release", 
           "--manifest-path", "lis-format/Cargo.toml",
           "--locked"
    system "cargo", "build", "--release", 
           "--manifest-path", "lis-runtime/Cargo.toml",
           "--locked"
    system "cargo", "build", "--release", 
           "--manifest-path", "lis-api/Cargo.toml",
           "--locked"

    # Install binaries
    bin.install "target/release/lis-cli"
    bin.install "target/release/lis-format"
    bin.install "target/release/lis-runtime"
    bin.install "target/release/lis-api"

    # Install documentation
    doc.install "sil-ecosystem/README.md"
    doc.install "docs"
  end

  test do
    system "#{bin}/lis-cli", "--version"
    system "#{bin}/lis-format", "--version"
    system "#{bin}/lis-runtime", "--version"
    system "#{bin}/lis-api", "--version"
  end
end
