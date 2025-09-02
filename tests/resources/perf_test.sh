#!/bin/bash

# This script performs a performance test on the `medi` note-taking application.
# It generates a specified number of notes, times a search operation, and cleans up afterward.
set -e
# --- Configuration ---
MEDI_BINARY="../../target/release/medi" # Path to the medi binary; adjust if necessary
NOTE_COUNT=${1:-1000} # Default to 1000 notes if no argument is given
SEARCH_TERM="performance"

# --- Helper Functions ---
function generate_notes() {
    echo "--- Generating ${NOTE_COUNT} notes... ---"
    local start_time=$(date +%s)
    for i in $(seq 1 $NOTE_COUNT); do
        NOTE_KEY="perf-test-$i"
        # Generate ~50KB of random-looking text
        NOTE_CONTENT=$(head -c 50000 /dev/urandom | base64)

        # Inject the search term into one of the notes
        if [ $i -eq $((NOTE_COUNT / 2)) ]; then
            NOTE_CONTENT="${NOTE_CONTENT} ${SEARCH_TERM}"
        fi

        # Create the note using the pipe method for speed
        echo "$NOTE_CONTENT" | $MEDI_BINARY new "$NOTE_KEY" > /dev/null

        # Print progress
        if (( $i % 100 == 0 )); then
            echo "Created $i / $NOTE_COUNT notes..."
        fi
    done
    local end_time=$(date +%s)
    local elapsed=$((end_time - start_time))
    echo "Note generation complete. Time taken: ${elapsed} seconds."
}

function time_search() {
    echo -e "\n--- Rebuilding search index... ---"
    time $MEDI_BINARY reindex

    echo -e "\n--- Timing search for the term '${SEARCH_TERM}'... ---"
    # The `time` command will measure the execution time of the search
    time $MEDI_BINARY search "$SEARCH_TERM"
}

function cleanup() {
    echo -e "\n--- Cleaning up ${NOTE_COUNT} test notes... ---"
    for i in $(seq 1 $NOTE_COUNT); do
        NOTE_KEY="perf-test-$i"
        $MEDI_BINARY delete "$NOTE_KEY" --force > /dev/null
        if (( $i % 100 == 0 )); then
            echo "Deleted $i / $NOTE_COUNT notes..."
        fi
    done
    echo "Cleanup complete."
}

# --- Main Script Logic ---
# Ensure the binary is built in release mode
if [ ! -f "$MEDI_BINARY" ]; then
    echo "Building release binary..."
    cargo build --release
fi

generate_notes
time_search
cleanup