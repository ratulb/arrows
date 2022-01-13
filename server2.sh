#rm -rf /tmp/arrows.db

mkdir /tmp/server2

port=8181 RUST_BACKTRACE=full ARROWS_DB_PATH=/tmp/server2 cargo run --bin arrows -- --addr 0.0.0.0:8181

