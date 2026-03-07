#!/usr/bin/env bash
set -euo pipefail

BOLD='\033[1m'
DIM='\033[2m'
CYAN='\033[36m'
GREEN='\033[32m'
RED='\033[31m'
RESET='\033[0m'

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
INSTALL_DIR="/opt/sodium"
BIN_NAME="sodium"

echo ""
echo -e "  ${CYAN}${BOLD}⚛ SODIUM — deploy${RESET}"
echo -e "  ${DIM}──────────────────────────${RESET}"
echo ""

# ── Kill running instance ──────────────────────────────────────────────
if pkill -x "${BIN_NAME}" 2>/dev/null; then
    echo -e "  ${DIM}Killed running ${BIN_NAME}${RESET}"
    sleep 0.2
fi

# ── Version bump (YY.MM.DDII) ─────────────────────────────────────────
CARGO_TOML="${PROJECT_DIR}/Cargo.toml"
TODAY_PREFIX="$(date +%y).$(date +%-m).$(date +%-d)"
CURRENT_VERSION=$(grep -oP '^version = "\K[^"]+' "$CARGO_TOML")

if [[ "$CURRENT_VERSION" == ${TODAY_PREFIX}* ]]; then
    # Same day: increment the 2-digit suffix
    CURRENT_SEQ="${CURRENT_VERSION##${TODAY_PREFIX}}"
    NEXT_SEQ=$(printf "%02d" $(( 10#${CURRENT_SEQ} + 1 )))
else
    NEXT_SEQ="01"
fi

NEW_VERSION="${TODAY_PREFIX}${NEXT_SEQ}"
sed -i "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" "$CARGO_TOML"
echo -e "  ${DIM}Version: ${NEW_VERSION}${RESET}"

# ── Clean ───────────────────────────────────────────────────────────────
echo -e "  ${DIM}Cleaning target/...${RESET}"
rm -rf "${PROJECT_DIR}/target"

# ── Build release ───────────────────────────────────────────────────────
echo -e "  ${DIM}Building release...${RESET}"
cargo build --release --manifest-path "${PROJECT_DIR}/Cargo.toml"

# ── Install ─────────────────────────────────────────────────────────────
echo -e "  ${DIM}Installing to ${INSTALL_DIR}...${RESET}"
sudo mkdir -p "${INSTALL_DIR}"
sudo cp "${PROJECT_DIR}/target/release/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
sudo chmod +x "${INSTALL_DIR}/${BIN_NAME}"

# ── Symlink in PATH ────────────────────────────────────────────────────
if [[ ! -L "/usr/local/bin/${BIN_NAME}" ]] || [[ "$(readlink -f /usr/local/bin/${BIN_NAME})" != "${INSTALL_DIR}/${BIN_NAME}" ]]; then
    sudo ln -sf "${INSTALL_DIR}/${BIN_NAME}" "/usr/local/bin/${BIN_NAME}"
    echo -e "  ${DIM}Symlink: /usr/local/bin/${BIN_NAME} → ${INSTALL_DIR}/${BIN_NAME}${RESET}"
fi

echo ""
echo -e "  ${GREEN}${BOLD}Deployed.${RESET}"
echo -e "  ${DIM}Version: ${NEW_VERSION}${RESET}"
echo -e "  ${DIM}Binary : ${INSTALL_DIR}/${BIN_NAME}${RESET}"
echo -e "  ${DIM}Command: ${BIN_NAME}${RESET}"
echo ""

# ── Launch ──────────────────────────────────────────────────────────────
echo -e "  ${CYAN}Launching...${RESET}"
echo ""
exec "${INSTALL_DIR}/${BIN_NAME}"
