#!/usr/bin/env bash

# Clean up build artifacts and temporary directories
echo "Cleaning up build artifacts and temporary directories..."
cargo clean
# Remove persisted data
find . -name "target" -type d -prune -exec rm -rf {} \;
find . -name "dist" -type d -prune -exec rm -rf {} \;
echo "Cleanup complete!"