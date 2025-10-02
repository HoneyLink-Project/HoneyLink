#!/usr/bin/env bash
# HoneyLink Vault Development Setup Script (WSL/Linux)
#
# This script sets up a local HashiCorp Vault instance for development.
# WARNING: This is for DEVELOPMENT ONLY. Never use -dev mode in production.
#
# Prerequisites:
#   - HashiCorp Vault CLI installed
#   - curl, jq
#
# Usage:
#   ./scripts/setup-vault-dev.sh [--vault-version 1.15.0] [--port 8200]

set -euo pipefail

# Default configuration
VAULT_VERSION="${VAULT_VERSION:-1.15.0}"
VAULT_PORT="${VAULT_PORT:-8200}"
VAULT_ADDR="http://127.0.0.1:${VAULT_PORT}"
VAULT_TOKEN="root"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

log_info() { echo -e "${CYAN}==> $*${NC}"; }
log_success() { echo -e "${GREEN}✓ $*${NC}"; }
log_warning() { echo -e "${YELLOW}! $*${NC}"; }
log_error() { echo -e "${RED}✗ $*${NC}"; }

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --vault-version)
            VAULT_VERSION="$2"
            shift 2
            ;;
        --port)
            VAULT_PORT="$2"
            VAULT_ADDR="http://127.0.0.1:${VAULT_PORT}"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [--vault-version VERSION] [--port PORT]"
            echo ""
            echo "Options:"
            echo "  --vault-version VERSION   Vault version to use (default: 1.15.0)"
            echo "  --port PORT               Port to run Vault on (default: 8200)"
            echo "  --help                    Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

log_info "HoneyLink Vault Development Setup"
log_info "  Version: ${VAULT_VERSION}"
log_info "  Port: ${VAULT_PORT}"

# Check if vault is installed
if ! command -v vault &> /dev/null; then
    log_error "Vault CLI not found. Installing..."

    # Detect OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        VAULT_OS="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        VAULT_OS="darwin"
    else
        log_error "Unsupported OS: $OSTYPE"
        exit 1
    fi

    # Detect architecture
    ARCH=$(uname -m)
    case $ARCH in
        x86_64)
            VAULT_ARCH="amd64"
            ;;
        aarch64|arm64)
            VAULT_ARCH="arm64"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    VAULT_ZIP="vault_${VAULT_VERSION}_${VAULT_OS}_${VAULT_ARCH}.zip"
    VAULT_URL="https://releases.hashicorp.com/vault/${VAULT_VERSION}/${VAULT_ZIP}"

    log_info "Downloading Vault from: ${VAULT_URL}"

    TEMP_DIR=$(mktemp -d)
    trap "rm -rf ${TEMP_DIR}" EXIT

    curl -fsSL "${VAULT_URL}" -o "${TEMP_DIR}/${VAULT_ZIP}"

    if command -v unzip &> /dev/null; then
        unzip -q "${TEMP_DIR}/${VAULT_ZIP}" -d "${TEMP_DIR}"
    else
        log_error "unzip not found. Please install unzip."
        exit 1
    fi

    # Install to /usr/local/bin (requires sudo) or ~/bin
    if [[ -w /usr/local/bin ]]; then
        mv "${TEMP_DIR}/vault" /usr/local/bin/
    else
        mkdir -p "$HOME/bin"
        mv "${TEMP_DIR}/vault" "$HOME/bin/"
        log_warning "Installed to ~/bin. Add to PATH: export PATH=\"\$HOME/bin:\$PATH\""
    fi

    log_success "Vault CLI installed"
fi

# Verify installation
INSTALLED_VERSION=$(vault version | grep -oP 'v\K[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
log_success "Vault CLI found: v${INSTALLED_VERSION}"

# Check if Vault is already running
if pgrep -f "vault server -dev" > /dev/null; then
    log_warning "Vault dev server is already running. Stopping..."
    pkill -f "vault server -dev" || true
    sleep 2
fi

# Create data directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "${SCRIPT_DIR}")"
DATA_DIR="${PROJECT_ROOT}/data/vault-dev"
mkdir -p "${DATA_DIR}"
log_success "Created data directory: ${DATA_DIR}"

# Start Vault in dev mode
log_info "Starting Vault dev server on port ${VAULT_PORT}..."
log_warning "Development mode is INSECURE - root token: root"

# Start Vault in background
nohup vault server \
    -dev \
    -dev-root-token-id=root \
    -dev-listen-address="127.0.0.1:${VAULT_PORT}" \
    > "${DATA_DIR}/vault.log" 2>&1 &

VAULT_PID=$!
echo $VAULT_PID > "${DATA_DIR}/vault.pid"
log_success "Vault server started (PID: ${VAULT_PID})"
log_info "  Logs: ${DATA_DIR}/vault.log"
log_info "  To stop: kill \$(cat ${DATA_DIR}/vault.pid)"

# Wait for Vault to be ready
log_info "Waiting for Vault to be ready..."
MAX_RETRIES=10
RETRY_COUNT=0

while [[ $RETRY_COUNT -lt $MAX_RETRIES ]]; do
    if curl -sf "${VAULT_ADDR}/v1/sys/health" > /dev/null 2>&1; then
        log_success "Vault is ready"
        break
    fi
    RETRY_COUNT=$((RETRY_COUNT + 1))
    sleep 1
done

if [[ $RETRY_COUNT -eq $MAX_RETRIES ]]; then
    log_error "Vault failed to start after ${MAX_RETRIES} attempts"
    log_info "Check logs: cat ${DATA_DIR}/vault.log"
    exit 1
fi

# Configure environment
export VAULT_ADDR="${VAULT_ADDR}"
export VAULT_TOKEN="${VAULT_TOKEN}"

log_info "Configuring Vault for HoneyLink key hierarchy..."

# Enable KV v2 secrets engine
log_info "  • Enabling KV v2 secrets engine..."
vault secrets enable -path=honeylink kv-v2

# Create key hierarchy paths according to spec/security/encryption.md
KEY_PATHS=(
    "honeylink/k_root"
    "honeylink/k_service"
    "honeylink/k_profile"
    "honeylink/k_telemetry"
)

log_info "  • Creating key hierarchy paths..."
for path in "${KEY_PATHS[@]}"; do
    path_name="${path##*/}"
    log_info "    - ${path_name}"

    # Create initial placeholder
    vault kv put "${path}" \
        description="HoneyLink ${path_name} hierarchy" \
        created_at="$(date -Iseconds)" \
        environment="development" \
        placeholder=true \
        > /dev/null
done

# Create policy for HoneyLink services
log_info "  • Creating access policy..."
POLICY_FILE="${DATA_DIR}/honeylink-dev-policy.hcl"
cat > "${POLICY_FILE}" <<'EOF'
# HoneyLink Development Policy
# Allows full access to honeylink/* secrets for development

path "honeylink/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

path "honeylink/data/*" {
  capabilities = ["create", "read", "update", "delete", "list"]
}

path "honeylink/metadata/*" {
  capabilities = ["list", "read", "delete"]
}

# Allow token renewal
path "auth/token/renew-self" {
  capabilities = ["update"]
}

# Allow token lookup
path "auth/token/lookup-self" {
  capabilities = ["read"]
}
EOF

vault policy write honeylink-dev "${POLICY_FILE}"

log_success ""
log_success "✓ Vault development environment setup complete!"
echo ""
log_info "Quick Start:"
log_info "  1. Set environment variables:"
log_info "     export VAULT_ADDR=\"${VAULT_ADDR}\""
log_info "     export VAULT_TOKEN=\"${VAULT_TOKEN}\""
log_info "     # Or source: source ${PROJECT_ROOT}/.vault-dev.env"
echo ""
log_info "  2. Test connection:"
log_info "     vault status"
echo ""
log_info "  3. View secrets:"
log_info "     vault kv list honeylink"
echo ""
log_info "  4. Stop Vault:"
log_info "     kill \$(cat ${DATA_DIR}/vault.pid)"
echo ""
log_warning "⚠ SECURITY REMINDER:"
log_warning "  • This is a DEVELOPMENT setup with root token 'root'"
log_warning "  • Data is stored in memory and will be lost on restart"
log_warning "  • Never use -dev mode or root tokens in production"
log_warning "  • For production setup, see docs/VAULT_SETUP.md"

# Save connection info to file
ENV_FILE="${PROJECT_ROOT}/.vault-dev.env"
cat > "${ENV_FILE}" <<EOF
# HoneyLink Vault Development Environment
# Source this file: source .vault-dev.env

export VAULT_ADDR="${VAULT_ADDR}"
export VAULT_TOKEN="${VAULT_TOKEN}"
export VAULT_NAMESPACE=""
EOF

log_success "✓ Environment variables saved to: ${ENV_FILE}"
