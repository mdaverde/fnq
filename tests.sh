#!/usr/bin/env bash
export PATH="$(pwd):$PATH"
export FNQ_DIR="$(pwd)/fnqdir"

echo "Using test directory at $FNQ_DIR"

APPEND_FILE="$(pwd)/append.txt"
[[ -f "$APPEND_FILE" ]] && trash "$APPEND_FILE"

for i in {1..10}; do
  sleep_count=$((20 - i))
  cargo --quiet run -- --quiet test_append "$i" "$sleep_count"
done