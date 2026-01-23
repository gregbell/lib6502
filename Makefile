# C64 Emulator Makefile
# Ensures consistent builds and server startup

.PHONY: all build clean serve demo

# Default target: build and serve
all: demo

# Build WASM module from c64-emu crate
build:
	@echo "Building WASM module..."
	cd c64-emu && wasm-pack build --target web
	@echo "WASM build complete. Output in c64-emu/pkg/"
	@ls -lh c64-emu/pkg/c64_emu_bg.wasm

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf c64-emu/pkg
	rm -rf c64-demo/pkg
	rm -rf c64-demo/c64-emu
	cargo clean -p c64-emu

# Start development server (serves from repo root so imports work)
serve:
	@echo "Starting server at http://localhost:8000/c64-demo/"
	@echo "Press Ctrl+C to stop"
	python3 -m http.server 8000

# Build and serve (the main development workflow)
demo: build serve
