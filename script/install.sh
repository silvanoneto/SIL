#!/usr/bin/env bash
# Framework Installation Script
# Installs SIL-Core, LIS Compiler, and all tools

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# Configuration
_REPO="https://github.com/silvanoneto/SIL.git"
INSTALL_DIR="${HOME}/.sil"
CARGO_BIN="${HOME}/.cargo/bin"

# Functions
print_header() {
    echo -e "${BOLD}${CYAN}"
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║              🌀 Framework Installation                     ║"
    echo "║     SIL (Signal Intermediate Language)                         ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo -e "${RESET}"
}

print_step() {
    echo -e "${BOLD}${BLUE}==>${RESET} ${BOLD}$1${RESET}"
}

print_success() {
    echo -e "${GREEN}✓${RESET} $1"
}

print_error() {
    echo -e "${RED}✗${RESET} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${RESET} $1"
}

check_command() {
    if command -v "$1" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

check_prerequisites() {
    print_step "Checking prerequisites..."

    local missing=()

    # Check Rust
    if check_command rustc; then
        local rust_version=$(rustc --version | cut -d' ' -f2)
        print_success "Rust $rust_version"
    else
        missing+=("rust")
        print_error "Rust not found"
    fi

    # Check Cargo
    if check_command cargo; then
        local cargo_version=$(cargo --version | cut -d' ' -f2)
        print_success "Cargo $cargo_version"
    else
        missing+=("cargo")
        print_error "Cargo not found"
    fi

    # Check Git
    if check_command git; then
        local git_version=$(git --version | cut -d' ' -f3)
        print_success "Git $git_version"
    else
        missing+=("git")
        print_error "Git not found"
    fi

    if [ ${#missing[@]} -ne 0 ]; then
        echo ""
        print_error "Missing prerequisites: ${missing[*]}"
        echo ""
        echo "Please install:"
        for cmd in "${missing[@]}"; do
            case $cmd in
                rust|cargo)
                    echo "  • Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    ;;
                git)
                    echo "  • Git: https://git-scm.com/downloads"
                    ;;
            esac
        done
        exit 1
    fi

    echo ""
}

clone_repository() {
    print_step "Cloning repository..."

    if [ -d "$INSTALL_DIR" ]; then
        print_warning "Directory $INSTALL_DIR already exists"
        read -p "Remove and reinstall? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$INSTALL_DIR"
        else
            print_error "Installation cancelled"
            exit 1
        fi
    fi

    # For local development, just copy the current directory
    if [ -f "$(pwd)/Cargo.toml" ]; then
        print_warning "Installing from local directory (development mode)"
        cp -r "$(pwd)" "$INSTALL_DIR"
        print_success "Copied to $INSTALL_DIR"
    else
        git clone "$SIL_REPO" "$INSTALL_DIR"
        print_success "Cloned to $INSTALL_DIR"
    fi

    echo ""
}

build_framework() {
    print_step "Building Framework..."

    cd "$INSTALL_DIR"

    echo "  Building sil-core..."
    cargo build --release -p sil-core --quiet
    print_success "sil-core built"

    echo "  Building lis-core..."
    cargo build --release -p lis-core --quiet
    print_success "lis-core built"

    echo "  Building lis-cli..."
    cargo build --release -p lis-cli --quiet
    print_success "lis-cli built"

    echo "  Building lis-format..."
    cargo build --release -p lis-format --quiet
    print_success "lis-format built"

    echo ""
}

install_binaries() {
    print_step "Installing binaries..."

    cd "$INSTALL_DIR"

    # Install LIS CLI
    if [ -f "target/release/lis" ]; then
        cargo install --path lis-cli --force --quiet
        print_success "lis installed to $CARGO_BIN/lis"
    fi

    # Install LIS Formatter
    if [ -f "target/release/lis-format" ]; then
        cargo install --path lis-format --force --quiet
        print_success "lis-format installed to $CARGO_BIN/lis-format"
    fi

    # Check if cargo bin is in PATH
    if [[ ":$PATH:" != *":$CARGO_BIN:"* ]]; then
        print_warning "Cargo bin directory not in PATH"
        echo "  Add this to your shell config (~/.bashrc, ~/.zshrc, etc.):"
        echo "    export PATH=\"\$HOME/.cargo/bin:\$PATH\""
    fi

    echo ""
}

create_config() {
    print_step "Creating configuration..."

    mkdir -p "$HOME/.config/sil"

    cat > "$HOME/.config/sil/config.toml" <<EOF
# Framework Configuration

[sil]
# Default SIL mode
mode = "SIL-128"

# Enable GPU acceleration
enable_gpu = true

# Enable NPU acceleration
enable_npu = true

[lis]
# Compiler optimization level (O0, O1, O2, O3)
optimization = "O2"

# Target SIL mode
target_mode = "SIL-128"

[format]
# Indentation style (spaces or tabs)
indent_style = "spaces"

# Number of spaces for indentation
indent_size = 4

# Maximum line width
max_width = 100

[paths]
# Installation directory
install_dir = "$INSTALL_DIR"

# Examples directory
examples_dir = "$INSTALL_DIR/lis-cli/examples"
EOF

    print_success "Config created at $HOME/.config/sil/config.toml"
    echo ""
}

verify_installation() {
    print_step "Verifying installation..."

    local all_good=true

    # Check lis command
    if check_command lis; then
        local lis_version=$(lis --version 2>&1 | head -1 | awk '{print $2}')
        print_success "lis $lis_version"
    else
        print_error "lis command not found"
        all_good=false
    fi

    # Check lis-format command
    if check_command lis-format; then
        local format_version=$(lis-format --version 2>&1 | head -1 | awk '{print $2}')
        print_success "lis-format $format_version"
    else
        print_error "lis-format command not found"
        all_good=false
    fi

    echo ""

    if [ "$all_good" = true ]; then
        return 0
    else
        return 1
    fi
}

print_completion() {
    echo -e "${BOLD}${GREEN}"
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║            ✨ Installation Complete! ✨                         ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo -e "${RESET}"
    echo ""
    echo -e "${BOLD}Quick Start:${RESET}"
    echo ""
    echo "  # Create a new LIS program"
    echo "  $ cat > hello.lis <<EOF"
    echo "  fn main() {"
    echo "      let x = 42;"
    echo "      return x;"
    echo "  }"
    echo "  EOF"
    echo ""
    echo "  # Compile and run"
    echo "  $ lis run hello.lis"
    echo ""
    echo "  # Format code"
    echo "  $ lis-format hello.lis"
    echo ""
    echo "  # Check syntax"
    echo "  $ lis check hello.lis"
    echo ""
    echo "  # Get help"
    echo "  $ lis --help"
    echo "  $ lis info"
    echo ""
    echo -e "${BOLD}Documentation:${RESET}"
    echo "  • Examples: $INSTALL_DIR/lis-cli/examples/"
    echo "  • Config: $HOME/.config/sil/config.toml"
    echo "  • Repo: $INSTALL_DIR"
    echo ""
    echo -e "${BOLD}VSCode Extension:${RESET}"
    echo "  $ cd $INSTALL_DIR/sil-vscode"
    echo "  $ npm install"
    echo "  $ npm run compile"
    echo "  $ code --install-extension ."
    echo ""
    echo -e "${CYAN}\"We are the swarm. We are the vapor. We are the edge.\"${RESET}"
    echo -e "${CYAN}理信 (Lǐxìn) - Where logic and information are indistinguishable.${RESET}"
    echo ""
}

print_uninstall_info() {
    echo ""
    echo -e "${BOLD}To uninstall:${RESET}"
    echo "  $ rm -rf $INSTALL_DIR"
    echo "  $ cargo uninstall lis lis-format"
    echo "  $ rm -rf $HOME/.config/sil"
    echo ""
}

# Main installation flow
main() {
    print_header

    check_prerequisites
    clone_repository
    build_framework
    install_binaries
    create_config

    if verify_installation; then
        print_completion
        print_uninstall_info
    else
        print_error "Installation verification failed"
        echo "Please check the errors above and try again."
        exit 1
    fi
}

# Run installation
main "$@"
