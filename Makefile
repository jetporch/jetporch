all: bin
loc:
	loc
bin:
	cargo build --release # --target x86_64-unknown-linux-gnu

clean:
	rm -rf ./target
run:
	cargo run

contributors:
	git shortlog -sne --all

