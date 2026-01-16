#!/bin/bash
# ==============================================================================
# Docker Build & Deployment Helper Script
# ==============================================================================
# Usage: ./docker-build.sh [command] [options]
# ==============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME="sil"
VERSION="latest"
REGISTRY=""

# Print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Show usage
show_usage() {
    cat << EOF
Docker Build & Deployment Helper

Usage: $0 [command] [options]

Commands:
  build [target]          Build Docker image
                          Targets: runtime (default), development, all

  build-multi [arch]      Build for specific architecture
                          Arch: arm64, amd64, multi (both)

  run [program]           Run LIS program in container

  dev                     Start development environment

  test                    Run tests in container

  push [registry]         Push image to registry

  clean                   Remove all images and containers

  help                    Show this help message

Options:
  --tag TAG               Set image tag (default: latest)
  --registry URL          Set registry URL for push
  --no-cache              Build without cache

Examples:
  # Build runtime image
  $0 build

  # Build for ARM64 (Raspberry Pi)
  $0 build-multi arm64

  # Build multi-arch and push to registry
  $0 build-multi multi --registry ghcr.io/user

  # Run a program
  $0 run programs/hello.lis

  # Start development environment
  $0 dev

  # Clean up
  $0 clean

EOF
}

# Build runtime image
build_runtime() {
    local no_cache=""
    if [[ "$1" == "--no-cache" ]]; then
        no_cache="--no-cache"
    fi

    print_info "Building runtime image: ${IMAGE_NAME}:${VERSION}"
    docker build \
        ${no_cache} \
        --target runtime \
        -t ${IMAGE_NAME}:${VERSION} \
        -t ${IMAGE_NAME}:runtime \
        .

    print_success "Runtime image built successfully"
    docker images ${IMAGE_NAME}
}

# Build development image
build_development() {
    local no_cache=""
    if [[ "$1" == "--no-cache" ]]; then
        no_cache="--no-cache"
    fi

    print_info "Building development image: ${IMAGE_NAME}:dev"
    docker build \
        ${no_cache} \
        --target development \
        -t ${IMAGE_NAME}:dev \
        .

    print_success "Development image built successfully"
    docker images ${IMAGE_NAME}
}

# Build for specific architecture
build_multi_arch() {
    local arch=$1

    # Check if buildx is available
    if ! docker buildx version &> /dev/null; then
        print_error "Docker buildx is not available"
        print_info "Install with: docker buildx install"
        exit 1
    fi

    # Create builder if not exists
    if ! docker buildx inspect sil-builder &> /dev/null; then
        print_info "Creating buildx builder..."
        docker buildx create --name sil-builder --use
        docker buildx inspect --bootstrap
    else
        docker buildx use sil-builder
    fi

    case $arch in
        arm64)
            print_info "Building for ARM64 (linux/arm64)..."
            docker buildx build \
                --platform linux/arm64 \
                --target runtime \
                -t ${IMAGE_NAME}:${VERSION}-arm64 \
                --load \
                .
            print_success "ARM64 image built successfully"
            ;;

        amd64)
            print_info "Building for AMD64 (linux/amd64)..."
            docker buildx build \
                --platform linux/amd64 \
                --target runtime \
                -t ${IMAGE_NAME}:${VERSION}-amd64 \
                --load \
                .
            print_success "AMD64 image built successfully"
            ;;

        multi)
            print_info "Building multi-architecture image (ARM64 + AMD64)..."

            if [[ -z "$REGISTRY" ]]; then
                print_error "Registry URL required for multi-arch build"
                print_info "Use: $0 build-multi multi --registry <url>"
                exit 1
            fi

            docker buildx build \
                --platform linux/amd64,linux/arm64 \
                --target runtime \
                -t ${REGISTRY}/${IMAGE_NAME}:${VERSION} \
                --push \
                .
            print_success "Multi-arch image built and pushed successfully"
            ;;

        *)
            print_error "Unknown architecture: $arch"
            print_info "Supported: arm64, amd64, multi"
            exit 1
            ;;
    esac
}

# Run LIS program
run_program() {
    local program=$1

    if [[ -z "$program" ]]; then
        print_error "Program path required"
        print_info "Usage: $0 run <program.lis>"
        exit 1
    fi

    if [[ ! -f "$program" ]]; then
        print_error "Program not found: $program"
        exit 1
    fi

    local dir=$(dirname "$program")
    local file=$(basename "$program")

    print_info "Running: $program"
    docker run --rm \
        -v "$(pwd)/${dir}:/data:ro" \
        ${IMAGE_NAME}:${VERSION} \
        lis run "/data/${file}"
}

# Start development environment
start_dev() {
    print_info "Starting development environment..."

    if ! docker images ${IMAGE_NAME}:dev &> /dev/null; then
        print_warning "Development image not found, building..."
        build_development
    fi

    print_info "Launching interactive shell..."
    docker run --rm -it \
        -v "$(pwd):/workspace:rw" \
        -v sil-cargo-cache:/home/sil/.cargo/registry \
        -v sil-cargo-git:/home/sil/.cargo/git \
        -v sil-target-cache:/workspace/target \
        ${IMAGE_NAME}:dev
}

# Run tests
run_tests() {
    print_info "Running tests in container..."

    docker run --rm \
        -v "$(pwd):/workspace:ro" \
        ${IMAGE_NAME}:dev \
        cargo test --all

    print_success "Tests completed"
}

# Push image to registry
push_image() {
    if [[ -z "$REGISTRY" ]]; then
        print_error "Registry URL required"
        print_info "Usage: $0 push --registry <url>"
        exit 1
    fi

    print_info "Tagging image for registry: ${REGISTRY}"
    docker tag ${IMAGE_NAME}:${VERSION} ${REGISTRY}/${IMAGE_NAME}:${VERSION}

    print_info "Pushing to registry..."
    docker push ${REGISTRY}/${IMAGE_NAME}:${VERSION}

    print_success "Image pushed successfully"
}

# Clean up
clean_images() {
    print_warning "This will remove all images and containers"
    read -p "Are you sure? (y/N) " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "Removing containers..."
        docker ps -a | grep sil | awk '{print $1}' | xargs -r docker rm -f

        print_info "Removing images..."
        docker images | grep sil | awk '{print $3}' | xargs -r docker rmi -f

        print_info "Removing volumes..."
        docker volume ls | grep sil | awk '{print $2}' | xargs -r docker volume rm

        print_success "Cleanup completed"
    else
        print_info "Cleanup cancelled"
    fi
}

# Main script
main() {
    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed"
        exit 1
    fi

    # Parse arguments
    local command=""
    local target="runtime"

    while [[ $# -gt 0 ]]; do
        case $1 in
            build|build-multi|run|dev|test|push|clean|help)
                command=$1
                shift
                ;;
            --tag)
                VERSION=$2
                shift 2
                ;;
            --registry)
                REGISTRY=$2
                shift 2
                ;;
            --no-cache)
                NO_CACHE="--no-cache"
                shift
                ;;
            *)
                target=$1
                shift
                ;;
        esac
    done

    # Execute command
    case $command in
        build)
            case $target in
                runtime)
                    build_runtime $NO_CACHE
                    ;;
                development|dev)
                    build_development $NO_CACHE
                    ;;
                all)
                    build_runtime $NO_CACHE
                    build_development $NO_CACHE
                    ;;
                *)
                    print_error "Unknown target: $target"
                    show_usage
                    exit 1
                    ;;
            esac
            ;;

        build-multi)
            build_multi_arch $target
            ;;

        run)
            run_program $target
            ;;

        dev)
            start_dev
            ;;

        test)
            run_tests
            ;;

        push)
            push_image
            ;;

        clean)
            clean_images
            ;;

        help|"")
            show_usage
            ;;

        *)
            print_error "Unknown command: $command"
            show_usage
            exit 1
            ;;
    esac
}

# Run main
main "$@"
