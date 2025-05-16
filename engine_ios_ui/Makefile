# Makefile for Rusty Plugin Workflow iOS Integration

# Configuration
RELEASE_FLAG ?= 
# Use environment variable RELEASE_BUILD=1 to enable release mode, empty for debug
ifdef RELEASE_BUILD
	RELEASE_FLAG = --release
endif

# Default target
.PHONY: all
all: clean_build build_ios

# Build the iOS libraries and prepare for Xcode integration
.PHONY: build_ios
build_ios:
	@echo "Building iOS libraries with $(if $(RELEASE_FLAG),release,debug) mode..."
	cd .. && ./b_ios.sh $(RELEASE_FLAG)
	@echo "iOS build completed. Libraries ready for Xcode integration."

# Clean only build artifacts
.PHONY: clean_build
clean_build:
	@echo "Cleaning build artifacts..."
	cd .. && ./clean_ios_builds.sh --build
	@echo "Build artifacts cleaned."

# Clean all iOS-related build artifacts (engine libraries and build directories)
.PHONY: clean_ios
clean_ios:
	@echo "Cleaning all iOS build artifacts..."
	cd .. && ./clean_ios_builds.sh --all
	@echo "All iOS build artifacts cleaned."

# Clean only the Rust target files
.PHONY: clean_target
clean_target:
	@echo "Cleaning Rust target files..."
	cd .. && ./clean_ios_builds.sh --target
	@echo "Rust target files cleaned."

# Open the Xcode project
.PHONY: xcode
xcode:
	@echo "Opening Xcode project..."
	open EngineIOSUI.xcodeproj

# Full rebuild (clean and build)
.PHONY: rebuild
rebuild: clean_ios build_ios
	@echo "Full rebuild completed."

# Setup the Xcode project with proper settings
.PHONY: setup
setup:
	@echo "Setting up Xcode project..."
	./setup_xcode_project.sh
	@echo "Setup completed."

# Display help information
.PHONY: help
help:
	@echo "Rusty Plugin Workflow iOS Build Makefile"
	@echo "---------------------------------------"
	@echo "Targets:"
	@echo "  all        : Default target - clean build directory and rebuild iOS libraries"
	@echo "  build_ios  : Build iOS libraries and copy to correct locations"
	@echo "  clean_build: Clean only build directory artifacts"
	@echo "  clean_ios  : Clean all iOS build artifacts (libraries and build directories)"
	@echo "  clean_target: Clean only Rust target files"
	@echo "  xcode      : Open the Xcode project"
	@echo "  rebuild    : Full rebuild (clean all and build)"
	@echo "  setup      : Run setup script to configure Xcode project"
	@echo "  help       : Show this help message"
	@echo ""
	@echo "Environment variables:"
	@echo "  RELEASE_BUILD=1 : Build in release mode (default is debug mode)"
	@echo ""
	@echo "Examples:"
	@echo "  make                   # Build in debug mode"
	@echo "  make RELEASE_BUILD=1   # Build in release mode"
	@echo "  make clean_ios         # Clean all iOS artifacts"
	@echo "  make rebuild           # Clean and rebuild"
