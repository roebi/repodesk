#!/bin/sh
# podman_build.sh - Build the Repodesk container image.
#
# Reentrant: safe to run multiple times.
# Removes any existing image with the same tag before building.
#
# Usage:
#   ./podman_build.sh [VERSION]
#
# Examples:
#   ./podman_build.sh           # tags as repodesk:latest
#   ./podman_build.sh 0.2.0    # tags as repodesk:0.2.0 and repodesk:latest

set -e

IMAGE_NAME="repodesk"
VERSION="${1:-latest}"
DOCKERFILE="Dockerfile"

# Resolve script directory so the script works from any working directory.
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "${SCRIPT_DIR}"

echo "==> Repodesk container build"
echo "    image:      ${IMAGE_NAME}"
echo "    version:    ${VERSION}"
echo "    dockerfile: ${DOCKERFILE}"
echo ""

# Verify Dockerfile exists.
if [ ! -f "${DOCKERFILE}" ]; then
    echo "ERROR: ${DOCKERFILE} not found in ${SCRIPT_DIR}" >&2
    exit 1
fi

# Verify podman is available.
if ! command -v podman > /dev/null 2>&1; then
    echo "ERROR: podman not found on PATH" >&2
    exit 1
fi

# Remove existing image(s) with this name to ensure a clean rebuild.
# Suppress error if image does not exist (reentrant).
echo "==> Removing existing image(s) for ${IMAGE_NAME} ..."
podman rmi "${IMAGE_NAME}:latest" 2>/dev/null || true
if [ "${VERSION}" != "latest" ]; then
    podman rmi "${IMAGE_NAME}:${VERSION}" 2>/dev/null || true
fi

# Build the image.
echo "==> Building ${IMAGE_NAME}:${VERSION} ..."
if [ "${VERSION}" = "latest" ]; then
    podman build \
        --tag "${IMAGE_NAME}:latest" \
        --file "${DOCKERFILE}" \
        .
else
    podman build \
        --tag "${IMAGE_NAME}:${VERSION}" \
        --tag "${IMAGE_NAME}:latest" \
        --file "${DOCKERFILE}" \
        .
fi

echo ""
echo "==> Build complete."
podman image inspect "${IMAGE_NAME}:latest" \
    --format "    size:   {{.Size}} bytes" 2>/dev/null || true
echo "    tag:    ${IMAGE_NAME}:latest"
if [ "${VERSION}" != "latest" ]; then
    echo "    tag:    ${IMAGE_NAME}:${VERSION}"
fi
echo ""
echo "Run with:"
echo "    ./podman_run.sh [/path/to/repo]"
