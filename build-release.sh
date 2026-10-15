#!/usr/bin/env bash
# Build release binary for distribution
# Produces a statically linked binary targeting musl libc for maximum portability

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE}=== PDF Compressor Release Build ===${NC}"
echo

# Check for musl target
if ! rustup target list --installed | grep -q "x86_64-unknown-linux-musl"; then
    echo -e "${YELLOW}Installing musl target...${NC}"
    rustup target add x86_64-unknown-linux-musl
fi

# Check for musl toolchain
if ! command -v musl-gcc &> /dev/null; then
    echo -e "${YELLOW}musl-gcc not found. Installing musl-tools...${NC}"
    # Try to detect package manager
    if command -v pacman &> /dev/null; then
        sudo pacman -S --needed musl
    elif command -v apt-get &> /dev/null; then
        sudo apt-get update && sudo apt-get install -y musl-tools
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y musl-gcc
    elif command -v zypper &> /dev/null; then
        sudo zypper install -y musl-gcc
    else
        echo -e "${RED}Could not detect package manager. Please install musl-tools manually.${NC}"
        exit 1
    fi
fi

# Set up environment for static linking
export CC=musl-gcc
export CXX=musl-g++
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc

# Build
echo -e "${BLUE}Building release binary (musl static)...${NC}"
cargo build --release --target x86_64-unknown-linux-musl --bin pdf-compressor

BINARY_PATH="target/x86_64-unknown-linux-musl/release/pdf-compressor"

if [[ ! -f "$BINARY_PATH" ]]; then
    echo -e "${RED}Build failed: binary not found at $BINARY_PATH${NC}"
    exit 1
fi

# Strip binary (optional, saves ~1-2MB)
if command -v strip &> /dev/null; then
    echo -e "${BLUE}Stripping binary...${NC}"
    strip "$BINARY_PATH"
fi

# Verify static linking
echo -e "${BLUE}Verifying static linking...${NC}"
if ldd "$BINARY_PATH" 2>&1 | grep -q "not a dynamic executable"; then
    echo -e "${GREEN}✓ Fully statically linked${NC}"
else
    echo -e "${YELLOW}Warning: Binary has dynamic dependencies:${NC}"
    ldd "$BINARY_PATH"
fi

# Show binary info
echo
echo -e "${BLUE}Binary info:${NC}"
ls -lh "$BINARY_PATH"
file "$BINARY_PATH"

# Quick functionality test
echo
echo -e "${BLUE}Running smoke test...${NC}"
if "$BINARY_PATH" --version > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Binary runs successfully${NC}"
else
    echo -e "${RED}✗ Binary failed to run${NC}"
    exit 1
fi

# Test compression with a minimal PDF
TEST_PDF="/tmp/test_minimal.pdf"
cat > "$TEST_PDF" <<'EOF'
%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>
endobj
xref
0 4
0000000000 65535 f 
0000000009 00000 n 
0000000058 00000 n 
0000000115 00000 n 
trailer
<< /Size 4 /Root 1 0 R >>
startxref
193
%%EOF
EOF

OUTPUT_PDF="/tmp/test_output.pdf"
if "$BINARY_PATH" compress -i "$TEST_PDF" -o "$OUTPUT_PDF" --preset screen --force > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Compression test passed${NC}"
    rm -f "$TEST_PDF" "$OUTPUT_PDF"
else
    echo -e "${RED}✗ Compression test failed${NC}"
    rm -f "$TEST_PDF" "$OUTPUT_PDF"
    exit 1
fi

echo
echo -e "${GREEN}=== Build successful! ===${NC}"
echo -e "Binary: ${BLUE}$BINARY_PATH${NC}"
echo
echo "To install system-wide:"
echo "  sudo cp $BINARY_PATH /usr/local/bin/pdf-compressor"
echo
echo "Or copy to any directory in your PATH."