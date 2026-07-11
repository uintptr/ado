#!/bin/bash
set -euo pipefail

REPO="uintptr/ado"
INSTALL_DIR="${HOME}/.local/bin"
TMP_DIR=""

cleanup() {
    if [ -n "${TMP_DIR}" ] && [ -d "${TMP_DIR}" ]; then
        rm -rf "${TMP_DIR}"
    fi
}
trap cleanup EXIT INT TERM

# Detect the release asset name for this platform.
# macOS ships as a single universal binary (ado-darwin); Linux is amd64 only.
detect_asset() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="darwin" ;;
        *)
            echo "Error: Unsupported operating system: $(uname -s)" >&2
            exit 1
            ;;
    esac

    if [ "${os}" = "darwin" ]; then
        # Universal binary, no arch suffix
        echo "ado-darwin"
        return
    fi

    case "$(uname -m)" in
        x86_64|amd64) arch="amd64" ;;
        *)
            echo "Error: Unsupported architecture for ${os}: $(uname -m)" >&2
            exit 1
            ;;
    esac

    echo "ado-${os}-${arch}"
}

# Get latest release tag from GitHub
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

main() {
    local asset version ado_url

    echo "Installing ado..."

    asset=$(detect_asset)
    echo "Detected asset: ${asset}"

    version=$(get_latest_version)
    if [ -z "$version" ]; then
        echo "Error: Could not determine latest version" >&2
        exit 1
    fi
    echo "Latest version: ${version}"

    ado_url="https://github.com/${REPO}/releases/download/${version}/${asset}"

    # Create install directory if it doesn't exist
    mkdir -p "${INSTALL_DIR}"

    # Download binary
    TMP_DIR=$(mktemp -d)

    echo "Downloading ado from ${ado_url}..."
    if ! curl -fsSL -o "${TMP_DIR}/ado" "${ado_url}"; then
        echo "Error: Failed to download ado" >&2
        exit 1
    fi

    # Install binary
    chmod +x "${TMP_DIR}/ado"
    mv "${TMP_DIR}/ado" "${INSTALL_DIR}/ado"

    echo "Successfully installed ado to ${INSTALL_DIR}"

    # Check if install dir is in PATH
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        echo ""
        echo "Note: ${INSTALL_DIR} is not in your PATH."
        echo "Add it by running:"
        echo "  echo 'export PATH=\"\${HOME}/.local/bin:\${PATH}\"' >> ~/.bashrc"
        echo "  source ~/.bashrc"
    fi
}

main
