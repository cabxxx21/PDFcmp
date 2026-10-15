#!/bin/bash
# install.sh - Simple installer for pdfcmp PDF compressor

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== PDF Compressor (pdfcmp) Installer ===${NC}\n"

# Check Rust/Cargo
echo "Checking prerequisites..."
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Cargo not found${NC}"
    echo "Install Rust first: https://rustup.rs"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
echo -e "${GREEN}✓ Cargo found${NC}"

# Check Ghostscript (optional but recommended)
if command -v gs &> /dev/null; then
    echo -e "${GREEN}✓ Ghostscript found (compression backend ready)${NC}"
else
    echo -e "${YELLOW}⚠ Ghostscript not found${NC}"
    echo "  Recommended for better compression. Install via:"
    echo "    Ubuntu/Debian: sudo apt install ghostscript"
    echo "    Fedora: sudo dnf install ghostscript"
    echo "    Arch: sudo pacman -S ghostscript"
    echo "  (MuPDF fallback will still work)"
    echo ""
fi

# Build
echo -e "\n${YELLOW}Building pdfcmp...${NC}"
cargo build --release

if [ ! -f "target/release/pdfcmp" ]; then
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Build successful${NC}"

# Determine install location
INSTALL_DIR=""
if [ -d "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
elif [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# Install
echo -e "\n${YELLOW}Installing to $INSTALL_DIR...${NC}"
cp target/release/pdfcmp "$INSTALL_DIR/pdfcmp"
chmod +x "$INSTALL_DIR/pdfcmp"

# Verify PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo -e "${YELLOW}⚠ $INSTALL_DIR is not in PATH${NC}"
    echo "Add this to your shell config (~/.bashrc or ~/.zshrc):"
    echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
    NEED_PATH=1
else
    NEED_PATH=0
fi

# Verify installation
echo -e "\n${YELLOW}Verifying installation...${NC}"
if "$INSTALL_DIR/pdfcmp" --version &> /dev/null || "$INSTALL_DIR/pdfcmp" --help &> /dev/null; then
    echo -e "${GREEN}✓ Installation successful!${NC}"
else
    echo -e "${YELLOW}⚠ Installation done, but couldn't verify${NC}"
fi

# Summary
echo -e "\n${GREEN}=== Installation Complete ===${NC}"
echo "Binary location: $INSTALL_DIR/pdfcmp"
if [ $NEED_PATH -eq 1 ]; then
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "1. Add to PATH (see above)"
    echo "2. Reload shell: source ~/.bashrc  (or ~/.zshrc)"
    echo "3. Try: pdfcmp --help"
else
    echo ""
    echo -e "${YELLOW}Try it now:${NC}"
    echo "  pdfcmp --help"
    echo "  pdfcmp document.pdf"
fi
