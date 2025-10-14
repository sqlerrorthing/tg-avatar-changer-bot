TDLIB_VERSION := 1.8.29
TDLIB_TARGET_DIR := $(abspath target/tdlib)
TDLIB_BASE_URL := https://github.com/FedericoBruzzone/tdlib-rs/releases/download/v1.1.0

OS := $(shell uname -s)
ARCH := $(shell uname -m)

ifeq ($(OS),Linux)
ifeq ($(ARCH),x86_64)
TDLIB_FILE := tdlib-$(TDLIB_VERSION)-linux-x86_64.zip
endif
ifeq ($(ARCH),aarch64)
TDLIB_FILE := tdlib-$(TDLIB_VERSION)-linux-aarch64.zip
endif
endif

ifeq ($(OS),Darwin)
ifeq ($(ARCH),x86_64)
TDLIB_FILE := tdlib-$(TDLIB_VERSION)-macos-x86_64.zip
endif
ifeq ($(ARCH),arm64)
TDLIB_FILE := tdlib-$(TDLIB_VERSION)-macos-aarch64.zip
endif
endif

ifeq ($(OS),MINGW64_NT-10.0)
TDLIB_FILE := tdlib-$(TDLIB_VERSION)-windows-x86_64.zip
endif

TDLIB_URL := $(TDLIB_BASE_URL)/$(TDLIB_FILE)

.PHONY: all tdlib build cargo clean

all: build

tdlib: $(TDLIB_TARGET_DIR)

$(TDLIB_TARGET_DIR):
	@echo "Creating target directory..."
	mkdir -p target
	@if [ ! -f target/$(TDLIB_FILE) ]; then \
		echo "Downloading $(TDLIB_FILE) ..."; \
		curl -L -o target/$(TDLIB_FILE) $(TDLIB_URL); \
	else \
		echo "$(TDLIB_FILE) already downloaded."; \
	fi
	@echo "Unzipping $(TDLIB_FILE) to $(TDLIB_TARGET_DIR)..."
	mkdir -p $(TDLIB_TARGET_DIR)
	unzip -o target/$(TDLIB_FILE) -d target

build: tdlib
	@echo "Building Rust project..."
	LOCAL_TDLIB_PATH=$(TDLIB_TARGET_DIR) cargo build $(filter-out $@,$(MAKECMDGOALS))

run: tdlib
	@echo "Running Rust project..."
	LOCAL_TDLIB_PATH=$(TDLIB_TARGET_DIR) cargo run $(filter-out $@,$(MAKECMDGOALS))

clean:
	@echo "Cleaning target directory..."
	cargo clean
	rm -rf target
