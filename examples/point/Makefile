
main: main.c rustlib rust_points.h
	cc $< -o $@ -L ../../target/debug -l point -l pthread -l dl -lm
	
.PHONY: rustlib
rustlib:
	cargo build # --release

rust_points.h:
	cargo test --features c-headers -- generate_headers

clean:
	cargo clean
	$(RM) main
