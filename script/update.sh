#!/usr/bin/env bash
# Framework Update Script

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

INSTALL_DIR="${HOME}/.sil"

print_step() {
    echo -e "${BOLD}${BLUE}==>${RESET} ${BOLD}$1${RESET}"
}

print_success() {
    echo -e "${GREEN}✓${RESET} $1"
}

echo -e "${BOLD}${CYAN}"
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              🔄 Framework Update                           ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo -e "${RESET}"
echo ""

if [ ! -d "$INSTALL_DIR" ]; then
    echo "It is not installed. Run install.sh first."
    exit 1
fi

cd "$INSTALL_DIR"

print_step "Pulling latest changes..."
git pull origin main
print_success "Repository updated"
echo ""

print_step "Rebuilding framework..."
cargo build --release --quiet
print_success "Build complete"
echo ""

print_step "Reinstalling binaries..."
cargo install --path lis-cli --force --quiet
cargo install --path lis-format --force --quiet
print_success "Binaries updated"
echo ""

echo -e "${BOLD}${GREEN}✨ Update complete!${RESET}"
echo ""
