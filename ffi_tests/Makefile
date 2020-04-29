RELEASE=0

HEADER=generated.h
RUST_BINARY=libffi_tests.a
C_BINARY=c_binary
TARGET_DIR=target

ifeq "$(RELEASE)" "1"
CARGO_RELEASE=--release
TARGET_DIR:=$(TARGET_DIR)/release
else
CARGO_RELEASE=
TARGET_DIR:=$(TARGET_DIR)/debug
endif

RUST_SOURCES=Cargo.toml $(shell find src/ -type f -name '*.rs')

.PHONY: default all test build clean print_headers

default: test

all: test clean

test: $(HEADER) build tests/*
	cargo test $(CARGO_RELEASE)

build: $(RUST_SOURCES)
	cargo build $(CARGO_RELEASE)
	ln -sf "$(TARGET_DIR)/$(RUST_BINARY)"

$(HEADER): $(RUST_SOURCES)
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
	rm -f "$(RUST_BINARY)" "$(C_BINARY)"

distclean: clean
	rm -f "$(HEADER)"
