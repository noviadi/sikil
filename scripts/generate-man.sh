#!/bin/bash

# Generate man page for sikil
# The man page is automatically generated during release builds.
# This script forces regeneration for development.

set -e

echo "Generating sikil.1 man page..."
SIKIL_GENERATE_MAN=1 cargo build --quiet

echo "Man page generated: sikil.1"
echo ""
echo "To install the man page system-wide:"
echo "  sudo cp sikil.1 /usr/local/share/man/man1/"
echo "  sudo mandb"
