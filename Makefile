bin:
	RUSTFLAGS='-C target-feature=+crt-static' cargo build --release # --target x86_64-unknown-linux-gnu
run:
	cargo run
	# ./target/release/hello-rust

