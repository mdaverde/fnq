export PATH="$(pwd):$PATH"
export FNQ_DIR="$(pwd)/fnqtestdir"

echo "Using test directory at $FNQ_DIR"

cargo run --quiet test_append
cargo run --quiet test_append
cargo run --quiet test_append
