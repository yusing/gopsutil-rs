# gopsutil-rs Makefile
# Builds Rust static library for Linux amd64/arm64

.PHONY: all clean rust-amd64 rust-arm64 rust test

# Default target
all: rust

# Architecture detection
UNAME_M := $(shell uname -m)
ifeq ($(UNAME_M),x86_64)
    NATIVE_ARCH := amd64
    RUST_NATIVE_TARGET := x86_64-unknown-linux-gnu
else ifeq ($(UNAME_M),aarch64)
    NATIVE_ARCH := arm64
    RUST_NATIVE_TARGET := aarch64-unknown-linux-gnu
else
    $(error Unsupported architecture: $(UNAME_M))
endif

# Build directories
RUST_DIR := rs
TARGET_DIR := target
LIB_DIR := $(TARGET_DIR)/lib

# Rust targets
RUST_AMD64_TARGET := x86_64-unknown-linux-gnu
RUST_ARM64_TARGET := aarch64-unknown-linux-gnu

# Library names
LIB_NAME := libgopsutil_rs.so
AMD64_LIB := $(LIB_DIR)/amd64/$(LIB_NAME)
ARM64_LIB := $(LIB_DIR)/arm64/$(LIB_NAME)

# Create directories
$(LIB_DIR)/amd64 $(LIB_DIR)/arm64 $(LIB_DIR)/native:
	@mkdir -p $@

# Rust targets setup (install if not present)
rust-setup:
	@rustup target list --installed | grep -q $(RUST_AMD64_TARGET) || rustup target add $(RUST_AMD64_TARGET)
	@rustup target list --installed | grep -q $(RUST_ARM64_TARGET) || rustup target add $(RUST_ARM64_TARGET)


go-run:
	@echo "Running main.go"
	go run -ldflags='-checklinkname=0' cmd/gopsutil/main.go

# Build Rust library for native architecture
rust: $(LIB_DIR)/native
	@echo "Building Rust library for native architecture ($(NATIVE_ARCH))..."
	cd $(RUST_DIR) && cargo build --release --target $(RUST_NATIVE_TARGET)
	cp $(RUST_DIR)/target/$(RUST_NATIVE_TARGET)/release/$(LIB_NAME) $(LIB_DIR)/native/$(LIB_NAME)

rust-native: rust

lint:
	@echo "Linting code..."
	cd $(RUST_DIR) && cargo clippy -- -D warnings

# Run tests
test: rust-$(NATIVE_ARCH)
	@echo "Running Rust tests..."
	cd $(RUST_DIR) && cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(TARGET_DIR)
	cd $(RUST_DIR) && cargo clean

# Development helpers
dev-deps:
	@echo "Installing development dependencies..."
	cd $(RUST_DIR) && cargo fetch

fmt:
	@echo "Formatting code..."
	cd $(RUST_DIR) && cargo fmt

# Show build info
info:
	@echo "Build Information:"
	@echo "  Native Architecture: $(NATIVE_ARCH)"
	@echo "  Rust Native Target:  $(RUST_NATIVE_TARGET)"
	@echo "  Rust Version:       $(shell rustc --version)"
