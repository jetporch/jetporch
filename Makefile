all: bin
bin:
	RUSTFLAGS='-C target-feature=+crt-static' cargo build --release # --target x86_64-unknown-linux-gnu
test: clean bin
	chmod +x target/release/jetp    
	#./target/release/jetp --mode ssh
	./target/release/jetp ssh --playbook /tmp/foo --inventory /tmp/foo
clean:
	rm -rf ./target
run:
	cargo run
	# ./target/release/hello-rust

