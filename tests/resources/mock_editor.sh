#!/usr/bin/env sh
# This is a known-good mock editor script.
# It writes fixed content to the file path passed as the first argument ($1).
echo "integration test content" > "$1"