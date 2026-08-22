#!/usr/bin/env bash
#
# Build, deploy, and initialize the LMS Soroban contract (#657).
#
# This script does the real thing. If any step fails it exits non-zero and
# says why — it does not print a plausible-looking contract address and
# continue. `scripts/deploy_contracts.sh` is a mock that echoes hardcoded
# addresses; do not use it to judge whether a deployment worked.
#
# Usage:
#   SOROBAN_ACCOUNT_SECRET=S... ./scripts/deploy_lms.sh [network]
#
# Environment:
#   SOROBAN_ACCOUNT_SECRET  (required) secret key funding the deployment
#   SOROBAN_NETWORK         network name, default "testnet"
#   LMS_ADMIN               address to initialize as the first administrator;
#                           defaults to the deploying account's own address
#   SKIP_INIT               set to 1 to deploy without initializing

set -euo pipefail

NETWORK="${1:-${SOROBAN_NETWORK:-testnet}}"
CONTRACT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../contract" && pwd)"
WASM_TARGET="wasm32-unknown-unknown"
WASM_NAME="lms.wasm"
WASM_PATH="${CONTRACT_DIR}/target/${WASM_TARGET}/release/${WASM_NAME}"
OUT_DIR="${CONTRACT_DIR}/../deployment/${NETWORK}"

die() {
    echo "ERROR: $*" >&2
    exit 1
}

# ---------------------------------------------------------------------------
# 0. Locate a CLI
# ---------------------------------------------------------------------------
# The tool was renamed from `soroban` to `stellar`; accept either so this
# works on both older and current installs.
if command -v stellar >/dev/null 2>&1; then
    CLI="stellar"
elif command -v soroban >/dev/null 2>&1; then
    CLI="soroban"
else
    die "neither 'stellar' nor 'soroban' CLI found on PATH.
       Install one with: cargo install --locked stellar-cli"
fi

echo "Using CLI: ${CLI} ($(${CLI} --version 2>&1 | head -1))"
echo "Network:   ${NETWORK}"

[ -n "${SOROBAN_ACCOUNT_SECRET:-}" ] \
    || die "SOROBAN_ACCOUNT_SECRET is not set. Deployment needs a funded key."

# ---------------------------------------------------------------------------
# 1. Build
# ---------------------------------------------------------------------------
echo ""
echo "==> Building lms for ${WASM_TARGET} (release)"

rustup target add "${WASM_TARGET}" >/dev/null 2>&1 || true

(cd "${CONTRACT_DIR}" && cargo build -p lms --target "${WASM_TARGET}" --release)

[ -f "${WASM_PATH}" ] || die "expected WASM artifact not found at ${WASM_PATH}"

echo "    Built $(wc -c < "${WASM_PATH}" | tr -d ' ') bytes: ${WASM_PATH}"

# ---------------------------------------------------------------------------
# 2. Optimize, when the CLI supports it
# ---------------------------------------------------------------------------
if ${CLI} contract optimize --help >/dev/null 2>&1; then
    echo ""
    echo "==> Optimizing WASM"

    ${CLI} contract optimize --wasm "${WASM_PATH}"

    OPTIMIZED="${WASM_PATH%.wasm}.optimized.wasm"

    if [ -f "${OPTIMIZED}" ]; then
        echo "    Optimized to $(wc -c < "${OPTIMIZED}" | tr -d ' ') bytes"
        WASM_PATH="${OPTIMIZED}"
    fi
else
    echo ""
    echo "==> Skipping optimize step (not supported by this CLI build)"
fi

# ---------------------------------------------------------------------------
# 3. Deploy
# ---------------------------------------------------------------------------
echo ""
echo "==> Deploying to ${NETWORK}"

CONTRACT_ID="$(
    ${CLI} contract deploy \
        --wasm "${WASM_PATH}" \
        --source-account "${SOROBAN_ACCOUNT_SECRET}" \
        --network "${NETWORK}"
)"

# A deploy that prints nothing usable is a failed deploy, whatever its exit
# code. Contract IDs are 56-character strings beginning with C.
[[ "${CONTRACT_ID}" =~ ^C[A-Z0-9]{55}$ ]] \
    || die "deploy did not return a valid contract ID. Got: '${CONTRACT_ID}'"

echo "    Contract ID: ${CONTRACT_ID}"

mkdir -p "${OUT_DIR}"
echo "${CONTRACT_ID}" > "${OUT_DIR}/lms_contract_address.txt"
echo "    Recorded in ${OUT_DIR}/lms_contract_address.txt"

# ---------------------------------------------------------------------------
# 4. Initialize
# ---------------------------------------------------------------------------
# Initialization is a one-time, irreversible claim on the administrator role.
# It must happen in the same operational step as deployment: between the two,
# the contract is live and uninitialized, and whoever calls `initialize`
# first becomes its permanent administrator.
if [ "${SKIP_INIT:-0}" = "1" ]; then
    echo ""
    echo "==> Skipping initialization (SKIP_INIT=1)"
    echo "    WARNING: the contract is deployed and uninitialized. Whoever"
    echo "    calls initialize() first becomes its administrator."
else
    ADMIN="${LMS_ADMIN:-}"

    if [ -z "${ADMIN}" ]; then
        ADMIN="$(${CLI} keys address "${SOROBAN_ACCOUNT_SECRET}" 2>/dev/null || true)"
    fi

    [ -n "${ADMIN}" ] \
        || die "could not determine the admin address. Set LMS_ADMIN explicitly."

    echo ""
    echo "==> Initializing with admin ${ADMIN}"

    ${CLI} contract invoke \
        --id "${CONTRACT_ID}" \
        --source-account "${SOROBAN_ACCOUNT_SECRET}" \
        --network "${NETWORK}" \
        -- initialize --admin "${ADMIN}"

    echo ""
    echo "==> Verifying initialization"

    INITIALIZED="$(
        ${CLI} contract invoke \
            --id "${CONTRACT_ID}" \
            --source-account "${SOROBAN_ACCOUNT_SECRET}" \
            --network "${NETWORK}" \
            -- is_initialized
    )"

    # Confirm against on-chain state rather than trusting the invoke's exit
    # code, so a silent failure cannot pass for success.
    [ "${INITIALIZED}" = "true" ] \
        || die "contract reports is_initialized=${INITIALIZED} after initialization"

    echo "    is_initialized: true"
    echo "${ADMIN}" > "${OUT_DIR}/lms_admin_address.txt"
fi

echo ""
echo "Done. LMS deployed to ${NETWORK} at ${CONTRACT_ID}"
