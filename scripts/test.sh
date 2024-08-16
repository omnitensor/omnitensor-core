#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status
set -u  # Treat unset variables as an error when substituting

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_color() {
    printf "${2}${1}${NC}\n"
}

# Function to run Rust tests
run_rust_tests() {
    print_color "Running Rust unit tests..." "$YELLOW"
    cargo test --lib || return 1
    print_color "Rust unit tests passed successfully." "$GREEN"
}

# Function to run integration tests
run_integration_tests() {
    print_color "Running integration tests..." "$YELLOW"
    cargo test --test '*' || return 1
    print_color "Integration tests passed successfully." "$GREEN"
}

# Function to run benchmarks
run_benchmarks() {
    print_color "Running benchmarks..." "$YELLOW"
    cargo bench || return 1
    print_color "Benchmarks completed successfully." "$GREEN"
}

# Function to check code style
check_code_style() {
    print_color "Checking code style..." "$YELLOW"
    cargo fmt -- --check || return 1
    print_color "Code style check passed." "$GREEN"
}

# Function to run clippy for linting
run_clippy() {
    print_color "Running Clippy for linting..." "$YELLOW"
    cargo clippy -- -D warnings || return 1
    print_color "Clippy checks passed." "$GREEN"
}

# Main execution
main() {
    print_color "Starting OmniTensor Core test suite..." "$YELLOW"

    check_code_style || { print_color "Code style check failed!" "$RED"; exit 1; }
    run_clippy || { print_color "Clippy checks failed!" "$RED"; exit 1; }
    run_rust_tests || { print_color "Rust tests failed!" "$RED"; exit 1; }
    run_integration_tests || { print_color "Integration tests failed!" "$RED"; exit 1; }
    run_benchmarks || { print_color "Benchmarks failed!" "$RED"; exit 1; }

    print_color "All tests passed successfully!" "$GREEN"
}

# Run main function
main