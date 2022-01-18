rm -rf /tmp/arrows.db
RUST_BACKTRACE=full cargo run --bin arrows -- -i user --addr 127.0.0.1:7171 -d /tmp

