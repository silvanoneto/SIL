#!/bin/bash
# Script para construir e publicar pacotes .deb e Arch

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

VERSION=$(grep "^version" sil-ecosystem/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
PACKAGES=("lis-cli" "lis-format" "lis-runtime" "lis-api" "sil-ecosystem")

print_header() {
    echo -e "\n${BLUE}==== $1 ====${NC}\n"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}→ $1${NC}"
}

# Verificar dependências
check_deps() {
    print_header "Verificando dependências"
    
    local missing=0
    
    if ! command -v cargo &> /dev/null; then
        print_error "cargo não encontrado"
        missing=1
    else
        print_success "cargo instalado"
    fi
    
    if ! cargo install --list | grep -q "cargo-deb"; then
        print_info "cargo-deb não encontrado, instalando..."
        cargo install cargo-deb
    else
        print_success "cargo-deb instalado"
    fi
    
    if [ $missing -eq 1 ]; then
        print_error "Dependências faltando!"
        exit 1
    fi
}

# Construir pacotes .deb
build_deb() {
    print_header "Construindo pacotes .deb (v${VERSION})"
    
    for pkg in "${PACKAGES[@]}"; do
        print_info "Construindo $pkg..."
        cargo deb --manifest-path "$pkg/Cargo.toml" --release
    done
    
    print_success ".deb packages construídos"
    print_info "Arquivos:"
    ls -lh target/debian/*.deb
}

# Construir pacotes Arch
build_arch() {
    print_header "Construindo pacotes Arch (v${VERSION})"
    
    if ! command -v makepkg &> /dev/null; then
        print_error "makepkg não encontrado (requer Arch Linux ou pacman)"
        return 1
    fi
    
    for pkg_dir in packaging/aur/*/; do
        pkg=$(basename "$pkg_dir")
        if [ -f "$pkg_dir/PKGBUILD" ]; then
            print_info "Construindo $pkg..."
            cd "$pkg_dir"
            makepkg --noconfirm --force
            cd - > /dev/null
        fi
    done
    
    print_success "Pacotes Arch construídos"
    print_info "Arquivos:"
    find packaging/aur -name "*.pkg.tar.zst" -exec ls -lh {} \;
}

# Publicar no GitHub
publish_github() {
    print_header "Publicando no GitHub"
    
    if [ -z "$(git status --porcelain)" ]; then
        print_info "Criando tag v${VERSION}..."
        git tag "v${VERSION}" || print_error "Tag já existe"
        git push origin "v${VERSION}"
        print_success "Tag publicada"
    else
        print_error "Repositório tem mudanças não commitadas"
        return 1
    fi
    
    print_info "Criando GitHub Release..."
    print_info "URL: https://github.com/silvanoneto/SIL/releases/new?tag=v${VERSION}"
}

# Menu interativo
main() {
    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════╗"
    echo "║   SIL Ecosystem Package Builder        ║"
    echo "║   v${VERSION}                           ║"
    echo "╚════════════════════════════════════════╝"
    echo -e "${NC}"
    
    PS3=$'\n'"$(echo -e ${YELLOW})Escolha uma opção:$(echo -e ${NC}) "
    options=(
        "Construir .deb packages"
        "Construir pacotes Arch"
        "Construir ambos (deb + arch)"
        "Publicar no GitHub"
        "Verificar dependências"
        "Sair"
    )
    
    select opt in "${options[@]}"; do
        case $REPLY in
            1)
                check_deps
                build_deb
                ;;
            2)
                build_arch || print_error "Falha ao construir Arch"
                ;;
            3)
                check_deps
                build_deb
                build_arch || print_error "Falha ao construir Arch"
                ;;
            4)
                publish_github
                ;;
            5)
                check_deps
                ;;
            6)
                print_info "Saindo..."
                exit 0
                ;;
            *)
                print_error "Opção inválida"
                ;;
        esac
        
        read -p "$(echo -e ${YELLOW})Pressione Enter para continuar...$(echo -e ${NC})"
    done
}

# Executar
main
