#!/bin/bash
# Build script for Python and JavaScript/WASM FFI bindings
# Usage: ./build-ffi.sh [python|wasm|all]

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found. Install from https://rustup.rs"
        exit 1
    fi
    log_success "Rust/Cargo: $(cargo --version)"

    # Check Python for Python builds
    if [ "$BUILD_PYTHON" = true ]; then
        if ! command -v python3 &> /dev/null; then
            log_error "Python3 not found"
            exit 1
        fi
        log_success "Python: $(python3 --version)"

        # Check maturin
        if ! command -v maturin &> /dev/null; then
            log_warning "maturin not found. Installing..."
            pip3 install maturin
        fi
        log_success "Maturin: $(maturin --version)"

        # Check numpy
        if ! python3 -c "import numpy" 2>/dev/null; then
            log_warning "numpy not found. Installing..."
            pip3 install numpy
        fi
        log_success "numpy installed"
    fi

    # Check wasm-pack for WASM builds
    if [ "$BUILD_WASM" = true ]; then
        if ! command -v wasm-pack &> /dev/null; then
            log_error "wasm-pack not found. Install from https://rustwasm.github.io/wasm-pack/"
            exit 1
        fi
        log_success "wasm-pack: $(wasm-pack --version)"

        # Check wasm32 target
        if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
            log_warning "wasm32 target not installed. Installing..."
            rustup target add wasm32-unknown-unknown
        fi
        log_success "wasm32 target installed"
    fi

    echo
}

# Build Python bindings
build_python() {
    log_info "Building Python bindings..."
    echo

    # LIS-Core
    log_info "Building lis-core Python bindings..."
    cd "$SCRIPT_DIR/lis-core"

    if maturin build --release --features python,jsil -o dist/; then
        log_success "lis-core Python bindings built successfully"
    else
        log_error "Failed to build lis-core Python bindings"
        exit 1
    fi
    cd "$SCRIPT_DIR"
    echo

    # SIL-Core
    log_info "Building sil-core Python bindings..."
    cd "$SCRIPT_DIR/sil-core"

    if maturin build --release --features python -o dist/; then
        log_success "sil-core Python bindings built successfully"
    else
        log_error "Failed to build sil-core Python bindings"
        exit 1
    fi
    cd "$SCRIPT_DIR"
    echo

    log_success "All Python bindings built successfully!"
    log_info "Python wheels available in:"
    echo "  • lis-core/dist/*.whl"
    echo "  • sil-core/dist/*.whl"
    echo
}

# Build WASM bindings
build_wasm() {
    log_info "Building WASM bindings..."
    echo

    # LIS-Core
    log_info "Building lis-core WASM bindings..."
    cd "$SCRIPT_DIR/lis-core"

    log_info "  → Building for bundler..."
    if wasm-pack build --target bundler --out-dir pkg --features wasm; then
        log_success "  Bundler build complete"
    else
        log_error "  Failed to build for bundler"
        exit 1
    fi

    log_info "  → Building for web..."
    if wasm-pack build --target web --out-dir pkg-web --features wasm; then
        log_success "  Web build complete"
    else
        log_error "  Failed to build for web"
        exit 1
    fi

    log_info "  → Building for Node.js..."
    if wasm-pack build --target nodejs --out-dir pkg-node --features wasm; then
        log_success "  Node.js build complete"
    else
        log_error "  Failed to build for Node.js"
        exit 1
    fi

    log_success "lis-core WASM bindings built successfully"
    cd "$SCRIPT_DIR"
    echo

    # SIL-Core
    log_info "Building sil-core WASM bindings..."
    cd "$SCRIPT_DIR/sil-core"

    log_info "  → Building for bundler..."
    if wasm-pack build --target bundler --out-dir pkg --features wasm; then
        log_success "  Bundler build complete"
    else
        log_error "  Failed to build for bundler"
        exit 1
    fi

    log_info "  → Building for web..."
    if wasm-pack build --target web --out-dir pkg-web --features wasm; then
        log_success "  Web build complete"
    else
        log_error "  Failed to build for web"
        exit 1
    fi

    log_info "  → Building for Node.js..."
    if wasm-pack build --target nodejs --out-dir pkg-node --features wasm; then
        log_success "  Node.js build complete"
    else
        log_error "  Failed to build for Node.js"
        exit 1
    fi

    log_success "sil-core WASM bindings built successfully"
    cd "$SCRIPT_DIR"
    echo

    log_success "All WASM bindings built successfully!"
    log_info "WASM packages available in:"
    echo "  • lis-core/pkg/"
    echo "  • lis-core/pkg-web/"
    echo "  • lis-core/pkg-node/"
    echo "  • sil-core/pkg/"
    echo "  • sil-core/pkg-web/"
    echo "  • sil-core/pkg-node/"
    echo
}

# Main script
main() {
    BUILD_PYTHON=false
    BUILD_WASM=false

    # Parse arguments
    if [ $# -eq 0 ]; then
        BUILD_PYTHON=true
        BUILD_WASM=true
    else
        case "$1" in
            python)
                BUILD_PYTHON=true
                ;;
            wasm)
                BUILD_WASM=true
                ;;
            all)
                BUILD_PYTHON=true
                BUILD_WASM=true
                ;;
            *)
                echo "Usage: $0 [python|wasm|all]"
                echo
                echo "Options:"
                echo "  python - Build only Python bindings"
                echo "  wasm   - Build only WASM bindings"
                echo "  all    - Build both (default)"
                exit 1
                ;;
        esac
    fi

    echo "═══════════════════════════════════════════════════════"
    echo "   FFI Build Script"
    echo "═══════════════════════════════════════════════════════"
    echo

    check_prerequisites

    if [ "$BUILD_PYTHON" = true ]; then
        build_python
    fi

    if [ "$BUILD_WASM" = true ]; then
        build_wasm
    fi

    echo "═══════════════════════════════════════════════════════"
    log_success "Build completed successfully!"
    echo "═══════════════════════════════════════════════════════"
    echo
    log_info "Next steps:"
    if [ "$BUILD_PYTHON" = true ]; then
        echo "  • Install Python packages:"
        echo "    pip install lis-core/dist/*.whl"
        echo "    pip install sil-core/dist/*.whl"
        echo
    fi
    if [ "$BUILD_WASM" = true ]; then
        echo "  • Test WASM packages:"
        echo "    Open examples/javascript_example.html in a browser"
        echo
    fi
    echo "  • See FFI_USAGE.md for usage examples"
    echo "  • See FFI_BUILD.md for detailed build instructions"
    echo
}

main "$@"
