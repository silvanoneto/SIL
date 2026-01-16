# ==============================================================================
# Makefile for Docker & Kubernetes Operations
# ==============================================================================

.PHONY: help build build-dev build-multi test clean docker-push k8s-deploy k8s-clean install wasm wasm-lis wasm-sil brew-release deb-build arch-build

# Colors
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
NC := \033[0m

# Configuration
IMAGE_NAME := sil
VERSION := latest
REGISTRY ?=
NAMESPACE := sil-system
PKG_VERSION := $(shell grep "^version" sil-ecosystem/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')

# ==============================================================================
# Help
# ==============================================================================

help: ## Show this help message
	@echo "$(BLUE) Docker & Kubernetes Operations$(NC)"
	@echo ""
	@echo "$(GREEN)Available targets:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(YELLOW)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(GREEN)Examples:$(NC)"
	@echo "  make build              # Build runtime image"
	@echo "  make build-multi        # Build multi-arch (ARM64 + AMD64)"
	@echo "  make k8s-deploy         # Deploy to Kubernetes"
	@echo "  make install            # Install locally via cargo"

# ==============================================================================
# Docker Targets
# ==============================================================================

build: ## Build Docker runtime image
	@echo "$(BLUE)Building runtime image: $(IMAGE_NAME):$(VERSION)$(NC)"
	docker build --target runtime -t $(IMAGE_NAME):$(VERSION) -t $(IMAGE_NAME):runtime .
	@echo "$(GREEN)✓ Build complete$(NC)"
	@docker images $(IMAGE_NAME)

build-dev: ## Build Docker development image
	@echo "$(BLUE)Building development image: $(IMAGE_NAME):dev$(NC)"
	docker build --target development -t $(IMAGE_NAME):dev .
	@echo "$(GREEN)✓ Build complete$(NC)"

build-multi: ## Build multi-architecture image (requires registry)
	@if [ -z "$(REGISTRY)" ]; then \
		echo "$(RED)Error: REGISTRY not set$(NC)"; \
		echo "Usage: make build-multi REGISTRY=ghcr.io/user"; \
		exit 1; \
	fi
	@echo "$(BLUE)Building multi-arch image for ARM64 + AMD64$(NC)"
	docker buildx create --name sil-builder --use 2>/dev/null || docker buildx use sil-builder
	docker buildx build \
		--platform linux/amd64,linux/arm64 \
		--target runtime \
		-t $(REGISTRY)/$(IMAGE_NAME):$(VERSION) \
		--push .
	@echo "$(GREEN)✓ Multi-arch build complete and pushed$(NC)"

build-arm64: ## Build for ARM64 only
	@echo "$(BLUE)Building ARM64 image$(NC)"
	docker buildx create --name sil-builder --use 2>/dev/null || docker buildx use sil-builder
	docker buildx build \
		--platform linux/arm64 \
		--target runtime \
		-t $(IMAGE_NAME):$(VERSION)-arm64 \
		--load .
	@echo "$(GREEN)✓ ARM64 build complete$(NC)"

# ==============================================================================
# Test Targets
# ==============================================================================

test: ## Run tests in Docker container
	@echo "$(BLUE)Running tests...$(NC)"
	docker run --rm -v $(PWD):/workspace:ro $(IMAGE_NAME):dev cargo test --all
	@echo "$(GREEN)✓ Tests passed$(NC)"

test-local: ## Run tests locally
	@echo "$(BLUE)Running tests locally...$(NC)"
	cargo test --all
	@echo "$(GREEN)✓ Tests passed$(NC)"

bench: ## Run benchmarks
	@echo "$(BLUE)Running benchmarks...$(NC)"
	cargo bench --all
	@echo "$(GREEN)✓ Benchmarks complete$(NC)"

# ==============================================================================
# Run Targets
# ==============================================================================

run-dev: ## Start development environment
	@echo "$(BLUE)Starting development environment...$(NC)"
	docker run --rm -it \
		-v $(PWD):/workspace:rw \
		-v sil-cargo-cache:/home/sil/.cargo/registry \
		-v sil-cargo-git:/home/sil/.cargo/git \
		-v sil-target-cache:/workspace/target \
		$(IMAGE_NAME):dev

run-example: ## Run an example program (specify PROGRAM=path/to/file.lis)
	@if [ -z "$(PROGRAM)" ]; then \
		echo "$(RED)Error: PROGRAM not set$(NC)"; \
		echo "Usage: make run-example PROGRAM=lis-cli/examples/simple.lis"; \
		exit 1; \
	fi
	@echo "$(BLUE)Running: $(PROGRAM)$(NC)"
	docker run --rm \
		-v $(PWD)/$(dir $(PROGRAM)):/data:ro \
		$(IMAGE_NAME):$(VERSION) \
		lis run /data/$(notdir $(PROGRAM))

# ==============================================================================
# Docker Compose Targets
# ==============================================================================

compose-up: ## Start services with docker-compose
	@echo "$(BLUE)Starting docker-compose services...$(NC)"
	docker-compose up -d
	@echo "$(GREEN)✓ Services started$(NC)"
	docker-compose ps

compose-down: ## Stop docker-compose services
	@echo "$(BLUE)Stopping docker-compose services...$(NC)"
	docker-compose down
	@echo "$(GREEN)✓ Services stopped$(NC)"

compose-logs: ## View docker-compose logs
	docker-compose logs -f

compose-scale: ## Scale swarm nodes (specify NODES=5)
	@if [ -z "$(NODES)" ]; then \
		NODES=5; \
	fi
	@echo "$(BLUE)Scaling to $(NODES) nodes...$(NC)"
	docker-compose up --scale sil-node=$(NODES) -d sil-node
	@echo "$(GREEN)✓ Scaled to $(NODES) nodes$(NC)"

# ==============================================================================
# Kubernetes Targets
# ==============================================================================

k8s-deploy: ## Deploy to Kubernetes
	@echo "$(BLUE)Deploying to Kubernetes...$(NC)"
	kubectl apply -f k8s/deployment.yaml
	@echo "$(GREEN)✓ Deployed successfully$(NC)"
	@echo ""
	@echo "$(YELLOW)Checking status:$(NC)"
	kubectl get all -n $(NAMESPACE)

k8s-status: ## Check Kubernetes deployment status
	@echo "$(BLUE)Kubernetes Status$(NC)"
	@echo ""
	@echo "$(YELLOW)Namespace:$(NC)"
	kubectl get namespace $(NAMESPACE) 2>/dev/null || echo "Namespace not found"
	@echo ""
	@echo "$(YELLOW)Pods:$(NC)"
	kubectl get pods -n $(NAMESPACE) -o wide 2>/dev/null || echo "No pods found"
	@echo ""
	@echo "$(YELLOW)Services:$(NC)"
	kubectl get svc -n $(NAMESPACE) 2>/dev/null || echo "No services found"
	@echo ""
	@echo "$(YELLOW)PVCs:$(NC)"
	kubectl get pvc -n $(NAMESPACE) 2>/dev/null || echo "No PVCs found"

k8s-logs: ## View Kubernetes logs
	@echo "$(BLUE)Viewing logs...$(NC)"
	kubectl logs -n $(NAMESPACE) -l app=sil --tail=100 -f

k8s-describe: ## Describe Kubernetes resources
	@echo "$(BLUE)Describing resources...$(NC)"
	kubectl describe all -n $(NAMESPACE)

k8s-shell: ## Get shell in a pod
	@POD=$$(kubectl get pods -n $(NAMESPACE) -l app=sil -o jsonpath='{.items[0].metadata.name}' 2>/dev/null); \
	if [ -z "$$POD" ]; then \
		echo "$(RED)No pods found$(NC)"; \
		exit 1; \
	fi; \
	echo "$(BLUE)Connecting to pod: $$POD$(NC)"; \
	kubectl exec -it -n $(NAMESPACE) $$POD -- /bin/bash

k8s-scale: ## Scale Kubernetes deployment (specify REPLICAS=3)
	@if [ -z "$(REPLICAS)" ]; then \
		REPLICAS=3; \
	fi
	@echo "$(BLUE)Scaling to $(REPLICAS) replicas...$(NC)"
	kubectl scale deployment/sil-runtime -n $(NAMESPACE) --replicas=$(REPLICAS)
	@echo "$(GREEN)✓ Scaled to $(REPLICAS) replicas$(NC)"

k8s-clean: ## Remove Kubernetes deployment
	@echo "$(YELLOW)Warning: This will remove all resources from Kubernetes$(NC)"
	@read -p "Continue? [y/N] " confirm; \
	if [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]; then \
		echo "$(BLUE)Removing resources...$(NC)"; \
		kubectl delete -f k8s/deployment.yaml; \
		echo "$(GREEN)✓ Resources removed$(NC)"; \
	else \
		echo "$(YELLOW)Cancelled$(NC)"; \
	fi

# ==============================================================================
# Registry Targets
# ==============================================================================

docker-login: ## Login to Docker registry
	@if [ -z "$(REGISTRY)" ]; then \
		echo "$(RED)Error: REGISTRY not set$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Logging in to $(REGISTRY)...$(NC)"
	docker login $(REGISTRY)

docker-push: ## Push image to registry
	@if [ -z "$(REGISTRY)" ]; then \
		echo "$(RED)Error: REGISTRY not set$(NC)"; \
		echo "Usage: make docker-push REGISTRY=ghcr.io/user"; \
		exit 1; \
	fi
	@echo "$(BLUE)Pushing to $(REGISTRY)...$(NC)"
	docker tag $(IMAGE_NAME):$(VERSION) $(REGISTRY)/$(IMAGE_NAME):$(VERSION)
	docker push $(REGISTRY)/$(IMAGE_NAME):$(VERSION)
	@echo "$(GREEN)✓ Pushed successfully$(NC)"

# ==============================================================================
# WASM Build Targets
# ==============================================================================

wasm: wasm-lis wasm-sil ## Build all WASM packages for web
	@echo "$(GREEN)✓ All WASM packages built$(NC)"

wasm-lis: ## Build lis-core WASM for web
	@echo "$(BLUE)Building lis-core WASM...$(NC)"
	cd lis-core && wasm-pack build --target web --out-dir pkg --features wasm
	@echo "$(GREEN)✓ lis-core WASM ready (use lis-core/web/index.js for browser)$(NC)"

wasm-sil: ## Build sil-core WASM for web
	@echo "$(BLUE)Building sil-core WASM...$(NC)"
	cd sil-core && wasm-pack build --target web --out-dir pkg --features wasm
	@echo "$(GREEN)✓ sil-core WASM ready (use sil-core/web/index.js for browser)$(NC)"

# ==============================================================================
# Homebrew Release Targets
# ==============================================================================

brew-release: ## Prepare Homebrew release (requires GitHub release tag)
	@echo "$(BLUE)Preparing Homebrew release...$(NC)"
	@bash script/brew-release.sh
	@echo "$(GREEN)✓ Homebrew formula updated$(NC)"

brew-create-tap: ## Create Homebrew tap repository
	@echo "$(BLUE)Creating Homebrew tap repository...$(NC)"
	@if [ ! -d "homebrew-sil" ]; then \
		echo "$(YELLOW)Creating tap directory...$(NC)"; \
		mkdir -p homebrew-sil/Formula; \
		cd homebrew-sil && git init && git remote add origin https://github.com/SIL-XXI/homebrew-sil.git || true; \
		cp ../packaging/homebrew/sil-ecosystem.rb Formula/; \
		echo "$(GREEN)✓ Tap repository created$(NC)"; \
	else \
		echo "$(YELLOW)Tap already exists$(NC)"; \
	fi

brew-install-local: ## Install locally from formula (development)
	@echo "$(BLUE)Installing from local formula...$(NC)"
	brew install --build-bottle packaging/homebrew/sil-ecosystem.rb
	@echo "$(GREEN)✓ Installed successfully$(NC)"

brew-uninstall: ## Uninstall sil-ecosystem
	@echo "$(BLUE)Uninstalling sil-ecosystem...$(NC)"
	brew uninstall -f sil-ecosystem
	@echo "$(GREEN)✓ Uninstalled$(NC)"

# ==============================================================================
# Debian Package Targets
# ==============================================================================

deb-build: ## Build .deb packages for all components
	@echo "$(BLUE)Building Debian packages...$(NC)"
	@for dir in lis-cli lis-format lis-runtime lis-api sil-ecosystem; do \
		echo "$(YELLOW)Building $$dir...$(NC)"; \
		cargo deb --manifest-path $$dir/Cargo.toml --release; \
	done
	@echo "$(GREEN)✓ .deb packages built$(NC)"
	@ls -lh target/debian/*.deb

deb-build-single: ## Build .deb for specific package (specify PKG=lis-cli)
	@if [ -z "$(PKG)" ]; then \
		echo "$(RED)Error: PKG not set$(NC)"; \
		echo "Usage: make deb-build-single PKG=lis-cli"; \
		exit 1; \
	fi
	@echo "$(BLUE)Building $(PKG).deb...$(NC)"
	cargo deb --manifest-path $(PKG)/Cargo.toml --release
	@echo "$(GREEN)✓ Package built$(NC)"
	@ls -lh target/debian/$(PKG)*.deb

deb-install: ## Install .deb packages locally (requires sudo)
	@echo "$(YELLOW)Installing .deb packages locally...$(NC)"
	@if [ ! -f "target/debian/sil-ecosystem"*.deb ]; then \
		echo "$(YELLOW)Building packages first...$(NC)"; \
		make deb-build; \
	fi
	@echo "$(BLUE)Installing via sudo...$(NC)"
	sudo dpkg -i target/debian/*.deb
	@echo "$(GREEN)✓ Packages installed$(NC)"

deb-clean: ## Clean .deb build artifacts
	@echo "$(BLUE)Cleaning .deb artifacts...$(NC)"
	rm -rf target/debian/
	@echo "$(GREEN)✓ Cleaned$(NC)"

# ==============================================================================
# Arch Package Targets
# ==============================================================================

arch-build: ## Build Arch packages (PKGBUILD)
	@echo "$(BLUE)Building Arch packages...$(NC)"
	@for dir in packaging/aur/*/; do \
		pkg=$$(basename $$dir); \
		if [ -f "$$dir/PKGBUILD" ]; then \
			echo "$(YELLOW)Building $$pkg...$(NC)"; \
			cd $$dir && makepkg --noconfirm && cd - > /dev/null; \
		fi; \
	done
	@echo "$(GREEN)✓ Arch packages built$(NC)"
	@find packaging/aur -name "*.pkg.tar.zst" -exec ls -lh {} \;

arch-build-single: ## Build specific Arch package (specify PKG=sil-ecosystem)
	@if [ -z "$(PKG)" ]; then \
		echo "$(RED)Error: PKG not set$(NC)"; \
		echo "Usage: make arch-build-single PKG=sil-ecosystem"; \
		exit 1; \
	fi
	@if [ ! -d "packaging/aur/$(PKG)" ]; then \
		echo "$(RED)Error: Package not found$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Building $(PKG) Arch package...$(NC)"
	cd packaging/aur/$(PKG) && makepkg --noconfirm
	@echo "$(GREEN)✓ Package built$(NC)"
	@ls -lh packaging/aur/$(PKG)/*.pkg.tar.zst 2>/dev/null || echo "Build may have failed"

arch-install: ## Install Arch package locally (requires sudo)
	@if [ -z "$(PKG)" ]; then \
		echo "$(RED)Error: PKG not set$(NC)"; \
		echo "Usage: make arch-install PKG=sil-ecosystem"; \
		exit 1; \
	fi
	@if [ ! -f "packaging/aur/$(PKG)"/*.pkg.tar.zst ]; then \
		echo "$(YELLOW)Building package first...$(NC)"; \
		make arch-build-single PKG=$(PKG); \
	fi
	@echo "$(BLUE)Installing $(PKG)...$(NC)"
	sudo pacman -U --noconfirm packaging/aur/$(PKG)/*.pkg.tar.zst
	@echo "$(GREEN)✓ Package installed$(NC)"

arch-submit-aur: ## Prepare Arch packages for AUR submission
	@echo "$(BLUE)Preparing AUR submission...$(NC)"
	@for dir in packaging/aur/*/; do \
		pkg=$$(basename $$dir); \
		echo "$(YELLOW)Processing $$pkg...$(NC)"; \
		if [ -f "$$dir/PKGBUILD" ]; then \
			cd $$dir && \
			git init 2>/dev/null || true && \
			git add PKGBUILD .SRCINFO 2>/dev/null || makepkg --printsrcinfo > .SRCINFO && \
			git add .SRCINFO && \
			cd - > /dev/null; \
		fi; \
	done
	@echo "$(GREEN)✓ AUR submission ready$(NC)"
	@echo "$(YELLOW)Next: git push to aur.archlinux.org:$(PKG).git$(NC)"

arch-clean: ## Clean Arch build artifacts
	@echo "$(BLUE)Cleaning Arch artifacts...$(NC)"
	@find packaging/aur -type f \( -name "*.pkg.tar.zst" -o -name ".SRCINFO" \) -delete
	@echo "$(GREEN)✓ Cleaned$(NC)"

# ==============================================================================
# Local Build Targets
# ==============================================================================

install: ## Install locally via cargo
	@echo "$(BLUE)Installing locally...$(NC)"
	cargo install --path lis-cli
	@echo "$(GREEN)✓ Installed successfully$(NC)"
	@echo ""
	@which lis

build-local: ## Build locally (release)
	@echo "$(BLUE)Building release locally...$(NC)"
	cargo build --release --bin lis
	@echo "$(GREEN)✓ Build complete: target/release/lis$(NC)"

build-local-debug: ## Build locally (debug)
	@echo "$(BLUE)Building debug locally...$(NC)"
	cargo build --bin lis
	@echo "$(GREEN)✓ Build complete: target/debug/lis$(NC)"

# ==============================================================================
# Cleanup Targets
# ==============================================================================

clean: ## Clean local build artifacts
	@echo "$(BLUE)Cleaning build artifacts...$(NC)"
	cargo clean
	@echo "$(GREEN)✓ Clean complete$(NC)"

docker-clean: ## Remove Docker images and containers
	@echo "$(YELLOW)Warning: This will remove all Docker images and containers$(NC)"
	@read -p "Continue? [y/N] " confirm; \
	if [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]; then \
		echo "$(BLUE)Removing containers...$(NC)"; \
		docker ps -a | grep sil | awk '{print $$1}' | xargs -r docker rm -f 2>/dev/null || true; \
		echo "$(BLUE)Removing images...$(NC)"; \
		docker images | grep sil | awk '{print $$3}' | xargs -r docker rmi -f 2>/dev/null || true; \
		echo "$(BLUE)Removing volumes...$(NC)"; \
		docker volume ls | grep sil | awk '{print $$2}' | xargs -r docker volume rm 2>/dev/null || true; \
		echo "$(GREEN)✓ Docker cleanup complete$(NC)"; \
	else \
		echo "$(YELLOW)Cancelled$(NC)"; \
	fi

clean-all: clean docker-clean ## Clean everything (local + docker)

# ==============================================================================
# Info Targets
# ==============================================================================

info: ## Show project information
	@echo "$(BLUE)Project Information$(NC)"
	@echo ""
	@echo "$(YELLOW)Image:$(NC)          $(IMAGE_NAME):$(VERSION)"
	@echo "$(YELLOW)Registry:$(NC)       $(REGISTRY)"
	@echo "$(YELLOW)Namespace:$(NC)      $(NAMESPACE)"
	@echo ""
	@echo "$(YELLOW)Docker Images:$(NC)"
	@docker images $(IMAGE_NAME) 2>/dev/null || echo "  No images found"
	@echo ""
	@echo "$(YELLOW)Running Containers:$(NC)"
	@docker ps | grep sil || echo "  No containers running"
	@echo ""
	@echo "$(YELLOW)Kubernetes Pods:$(NC)"
	@kubectl get pods -n $(NAMESPACE) 2>/dev/null || echo "  Not deployed to Kubernetes"

version: ## Show versions
	@echo "$(BLUE)Versions$(NC)"
	@echo ""
	@echo "$(YELLOW)Rust:$(NC)           $$(rustc --version 2>/dev/null || echo 'Not installed')"
	@echo "$(YELLOW)Cargo:$(NC)          $$(cargo --version 2>/dev/null || echo 'Not installed')"
	@echo "$(YELLOW)Docker:$(NC)         $$(docker --version 2>/dev/null || echo 'Not installed')"
	@echo "$(YELLOW)Docker Compose:$(NC) $$(docker-compose --version 2>/dev/null || echo 'Not installed')"
	@echo "$(YELLOW)Kubectl:$(NC)        $$(kubectl version --client --short 2>/dev/null || echo 'Not installed')"
	@echo "$(YELLOW)Buildx:$(NC)         $$(docker buildx version 2>/dev/null || echo 'Not installed')"

# ==============================================================================
# Default Target
# ==============================================================================

.DEFAULT_GOAL := help
