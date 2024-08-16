#!/bin/bash

set -e

# Configuration
RUST_ENV=${RUST_ENV:-production}
DEPLOY_TARGET=${DEPLOY_TARGET:-mainnet}
LOG_FILE="./deploy_$(date +%Y%m%d_%H%M%S).log"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "[$(date +'%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Check if required tools are installed
check_dependencies() {
    command -v cargo >/dev/null 2>&1 || { log "${RED}Error: cargo is not installed.${NC}"; exit 1; }
    command -v rustc >/dev/null 2>&1 || { log "${RED}Error: rustc is not installed.${NC}"; exit 1; }
}

# Build the project
build_project() {
    log "${GREEN}Building OmniTensor Core for $RUST_ENV environment...${NC}"
    cargo build --release
    if [ $? -ne 0 ]; then
        log "${RED}Build failed. Check the logs for more information.${NC}"
        exit 1
    fi
}

# Run tests
run_tests() {
    log "${GREEN}Running tests...${NC}"
    cargo test --release
    if [ $? -ne 0 ]; then
        log "${RED}Tests failed. Aborting deployment.${NC}"
        exit 1
    fi
}

# Deploy to target environment
deploy() {
    log "${GREEN}Deploying to $DEPLOY_TARGET...${NC}"
    
    case $DEPLOY_TARGET in
        mainnet)
            # Add mainnet-specific deployment steps here
            log "Deploying to mainnet..."
            ;;
        testnet)
            # Add testnet-specific deployment steps here
            log "Deploying to testnet..."
            ;;
        *)
            log "${RED}Unknown deploy target: $DEPLOY_TARGET${NC}"
            exit 1
            ;;
    esac

    # Simulating a deployment process
    sleep 5

    log "${GREEN}Deployment completed successfully.${NC}"
}

# Main execution
main() {
    log "Starting deployment process for OmniTensor Core"
    
    check_dependencies
    build_project
    run_tests
    deploy

    log "Deployment process completed."
}

# Run the script
main