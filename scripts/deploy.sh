#!/usr/bin/env bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print step information
print_step() {
    echo -e "${GREEN}==>${NC} $1"
}

# Function to print error and exit
error() {
    echo -e "${RED}Error:${NC} $1"
    exit 1
}

# Function to print warning
warning() {
    echo -e "${YELLOW}Warning:${NC} $1"
}

# Check if environment argument is provided
if [ $# -eq 0 ]; then
    error "Please provide an environment (staging/production)"
fi

ENVIRONMENT=$1
VALID_ENVIRONMENTS=("staging" "production")

# Validate environment
if [[ ! " ${VALID_ENVIRONMENTS[@]} " =~ " ${ENVIRONMENT} " ]]; then
    error "Invalid environment. Must be one of: ${VALID_ENVIRONMENTS[*]}"
fi

# Load environment variables
if [ -f ".env" ]; then
    source .env
else
    error ".env file not found"
fi

print_step "Deploying to $ENVIRONMENT environment..."

# Build the application
print_step "Building application..."
poetry install --no-dev
poetry run python -m build

# Run tests
print_step "Running tests..."
poetry run pytest

# Build Docker image if Dockerfile exists
if [ -f "Dockerfile" ]; then
    print_step "Building Docker image..."
    docker build -t "${PROJECT_NAME}:${ENVIRONMENT}" .
fi

# Deploy based on environment
case $ENVIRONMENT in
    staging)
        print_step "Deploying to staging..."
        if [ -n "${STAGING_HOST:-}" ]; then
            # Example: Deploy to staging server
            rsync -avz --delete dist/ "${STAGING_HOST}:/var/www/${PROJECT_NAME}"
        else
            warning "STAGING_HOST not set, skipping deployment"
        fi
        ;;
    production)
        print_step "Deploying to production..."
        if [ -n "${PRODUCTION_HOST:-}" ]; then
            # Example: Deploy to production server
            rsync -avz --delete dist/ "${PRODUCTION_HOST}:/var/www/${PROJECT_NAME}"
        else
            warning "PRODUCTION_HOST not set, skipping deployment"
        fi
        ;;
esac

print_step "Deployment complete! 🚀"