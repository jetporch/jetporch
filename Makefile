all: bin
loc:
	loc
bin:
	# sh ./version.sh
	# RUSTFLAGS='-C target-feature=+crt-static' cargo build --release # --target x86_64-unknown-linux-gnu
	cargo build --release # --target x86_64-unknown-linux-gnu
m1:
	SDKROOT=`xcrun -sdk macosx --show-sdk-path` MACOSX_DEPLOYMENT_TARGET=13.3 cargo build --target=aarch64-apple-darwin

test: clean bin
	chmod +x target/release/jetp    
	#./target/release/jetp --mode ssh
	./target/release/jetp ssh --playbook /tmp/foo --inventory /tmp/foo
clean:
	rm -rf ./target
run:
	cargo run
	# ./target/release/hello-rust
contributors:
	git shortlog -sne --all

