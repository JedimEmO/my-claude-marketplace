#!/usr/bin/env bash
#
# Auto-discovers all .md files under plugins/ and generates an mdbook.
# Usage:
#   ./build-book.sh          # build the book
#   ./build-book.sh serve    # live-reload dev server
#
set -euo pipefail
cd "$(dirname "$0")"

BOOK_SRC="book-src"

# Clean and recreate book-src
rm -rf "$BOOK_SRC"
mkdir -p "$BOOK_SRC"

# Generate SUMMARY.md by walking plugins/
python3 generate-summary.py > "$BOOK_SRC/SUMMARY.md"

# Copy the repo README as the book intro
if [ -f README.md ]; then
    cp README.md "$BOOK_SRC/index.md"
fi

# Mirror directory structure and symlink individual .md files
# so mdbook can resolve all paths from SUMMARY.md
find plugins -name '*.md' -not -path '*/\.git/*' | while read -r md; do
    # Strip leading "plugins/" and get the relative path for book-src
    rel="${md#plugins/}"
    target_dir="$BOOK_SRC/$(dirname "$rel")"
    mkdir -p "$target_dir"
    ln -snf "$(realpath "$md")" "$target_dir/$(basename "$md")"
done

# Run mdbook
if [ "${1:-}" = "serve" ]; then
    mdbook serve
elif [ "${1:-}" = "clean" ]; then
    rm -rf book-out "$BOOK_SRC"
    echo "Cleaned."
else
    mdbook build
    echo "Book built in book-out/"
fi
