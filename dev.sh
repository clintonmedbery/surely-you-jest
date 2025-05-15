#!/bin/bash

# Check if a path argument was provided
if [ "$#" -eq 0 ]; then
    echo "Usage: ./dev.sh <path-to-tests-directory>"
    exit 1
fi

TEST_PATH="$1"

# Start cargo-watch to continuously rebuild and run the project
cargo watch -x "run -- $TEST_PATH"