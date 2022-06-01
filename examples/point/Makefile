
point:
	cargo build # --release
	cargo test --features c-headers -- generate_headers
	cc main.c -o main -L ../../target/debug -l point -l pthread -l dl -lm

clean:
	cargo clean
	$(RM) main
