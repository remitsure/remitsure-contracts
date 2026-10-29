#!/usr/bin/env bash
# Deploys the RemitSure contract to testnet and prints the contract id.
# Put the id in remitsure-backend and remitsure-frontend as REMITSURE_CONTRACT_ID.
set -euo pipefail

IDENTITY="${STELLAR_IDENTITY:-remitsure-admin}"
NETWORK="${STELLAR_NETWORK:-testnet}"
WASM="target/wasm32v1-none/release/remitsure_remittance.wasm"

if ! stellar keys address "$IDENTITY" >/dev/null 2>&1; then
  echo "Creating and funding identity $IDENTITY"
  stellar keys generate "$IDENTITY" --network "$NETWORK" --fund
fi

ADMIN="$(stellar keys address "$IDENTITY")"

stellar contract deploy \
  --wasm "$WASM" \
  --source-account "$IDENTITY" \
  --network "$NETWORK" \
  --alias remitsure \
  -- --admin "$ADMIN"
