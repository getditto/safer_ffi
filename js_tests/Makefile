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

RUST_DYNAMIC_LIB ?= libnodejs_tests.$(DYLIB)

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

.PHONY: test-setup
test-setup: build tests/main.mjs
	cp -L $(RUST_DYNAMIC_LIB) tests/index.node

.PHONY: test
test: test-node test-web
	cargo check --no-default-features

.PHONY: test-node
test-node: tests/node.js test-setup
	@echo
	@echo '=== Node.js tests ==='
	@echo
	(cd tests && RUST_BACKTRACE=full node --trace-uncaught node.js)

.PHONY: test-web
test-web: index.html test-setup
	@echo
	@echo '=== Wasm tests ==='
	@echo
	@echo 'These tests are not automated yet. Please run the following command:'
	@echo
	@echo "    (cd \"$(shell pwd)\" && wasm-pack build --target web && python3 -m http.server 13337)"
	@echo
	@echo "and then connect to http://localhost:13337"

.PHONY: build
build: # $(RUST_SOURCES)
	cargo build $(CARGO_RELEASE)
	ln -sf "$(TARGET_DIR)"/$(RUST_DYNAMIC_LIB)

.PHONY: clean
clean:
	cargo clean
	rm -f $(RUST_DYNAMIC_LIB)
