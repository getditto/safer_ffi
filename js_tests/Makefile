RELEASE ?= 0

HEADERS = generated.h generated.cs

UNAME_S := $(shell uname -s | tr A-Z a-z)
ifeq ($(UNAME_S),linux)
	DYLIB := so
else
	ifeq ($(UNAME_S),darwin)
		DYLIB := dylib
	else
		$(error This Makefile can only be run on Linux or Darwin systems)
	endif
endif

RUST_DYNAMIC_LIB ?= libjs_tests.$(DYLIB)

TARGET_DIR = target

ifeq "$(RELEASE)" "1"
CARGO_RELEASE = --release
TARGET_DIR := $(TARGET_DIR)/release
else
CARGO_RELEASE =
TARGET_DIR := $(TARGET_DIR)/debug
endif

# RUST_SOURCES = Cargo.toml $(shell find src/ -type f -name '*.rs')

default: test
.PHONY: default

.PHONY: all
all: test clean

.PHONY: build-node
build-node:
	cargo build $(CARGO_RELEASE)
	ln -sf "$(TARGET_DIR)"/$(RUST_DYNAMIC_LIB)

tests/rust.node: build-node
	cp -L "$(RUST_DYNAMIC_LIB)" $@

.PHONY: test
test: test-node test-web
	cargo check --no-default-features

.PHONY: test-node
test-node: tests/node-entrypoint.js tests/rust.node
	@echo
	@echo '=== Node.js tests ==='
	@echo
	(cd tests && \
	        node --version && \
	        RUST_BACKTRACE=full \
	        node \
	                --trace-uncaught \
	                --unhandled-rejections=strict \
	        node-entrypoint.js \
	)

# Alias
.PHONY: test-wasm
test-wasm: test-web

.PHONY: test-web
test-web: index.html tests/web-entrypoint.mjs
	@echo
	@echo '=== Wasm tests ==='
	@echo
	cargo build --target wasm32-unknown-unknown --release
	@echo
	@echo 'The API tests are not automated yet. Please run the following command:'
	@echo
	@echo "    (cd \"$(shell pwd)\" && wasm-pack build --target web && python3 -m http.server 13337)"
	@echo
	@echo "and then connect to http://localhost:13337"

.PHONY: clean
clean:
	cargo clean
	rm -f $(RUST_DYNAMIC_LIB)
