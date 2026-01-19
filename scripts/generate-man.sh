#!/bin/bash

# Generate man page for sikil
# This script regenerates the sikil.1 man page from the CLI definitions

set -e

echo "Generating sikil.1 man page..."
cargo run --example generate_man_page --quiet

echo "Man page generated successfully: sikil.1"
echo ""
echo "To install the man page system-wide:"
echo "  sudo cp sikil.1 /usr/local/share/man/man1/"
echo "  sudo mandb"
