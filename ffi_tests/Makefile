RELEASE =? 0

HEADERS = generated.h generated.cs
RUST_STATIC_LIB = libffi_tests.a
RUST_DYNAMIC_LIB = libffi_tests.dylib

C_BINARY = c_binary
TARGET_DIR = target

ifeq "$(RELEASE)" "1"
CARGO_RELEASE = --release
TARGET_DIR := $(TARGET_DIR)/release
else
CARGO_RELEASE =
TARGET_DIR := $(TARGET_DIR)/debug
endif

RUST_SOURCES = Cargo.toml $(shell find src/ -type f -name '*.rs')

.PHONY: default all test build clean print_headers

default: test

all: test clean

test: $(HEADERS) build tests/main.rs tests/c/main.c tests/csharp/Tests.cs
	cargo test $(CARGO_RELEASE) -- --test-threads=1

build: $(RUST_SOURCES)
	cargo build $(CARGO_RELEASE)
	ln -sf "$(TARGET_DIR)"/$(RUST_STATIC_LIB)
	ln -sf "$(TARGET_DIR)"/$(RUST_DYNAMIC_LIB)

$(HEADERS): $(RUST_SOURCES)
	cargo test --features generate-headers -- \
		--exact generate_headers \
		--nocapture

print_headers:
	HEADERS_TO_STDOUT=1 \
	cargo test --features generate-headers -- \
		--exact generate_headers \
		--nocapture

clean:
	cargo clean
	(cd tests/csharp && dotnet clean) ||:
	rm -f $(RUST_STATIC_LIB) $(RUST_DYNAMIC_LIB) $(C_BINARY)

distclean: clean
	rm -f $(HEADERS)
