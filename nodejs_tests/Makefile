RELEASE ?= 0

HEADERS = generated.h generated.cs
RUST_DYNAMIC_LIB = libnodejs_tests.dylib

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

test: build tests/main.js
	cp -L $(RUST_DYNAMIC_LIB) tests/index.node
	(cd tests && node main.js)

build: $(RUST_SOURCES)
	cargo build $(CARGO_RELEASE)
	ln -sf "$(TARGET_DIR)"/$(RUST_DYNAMIC_LIB)

clean:
	cargo clean
	rm -f $(RUST_DYNAMIC_LIB)
