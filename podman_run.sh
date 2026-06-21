#!/bin/sh
# podman_run.sh - Run Repodesk in a container.
#
# Reentrant: stops and removes any existing container with the same name
# before starting a new one.
#
# Usage:
#   ./podman_run.sh [REPO_PATH]
#
# Examples:
#   ./podman_run.sh                    # uses current directory as workspace
#   ./podman_run.sh /path/to/repo     # uses explicit repository path
#
# The repository is mounted read-write into /workspace inside the container.
# On SELinux systems (Fedora, RHEL) the :z volume label is applied automatically.

set -e

IMAGE_NAME="repodesk"
IMAGE_TAG="latest"
CONTAINER_NAME="repodesk-ide"

# Resolve workspace path.
if [ -n "$1" ]; then
    WORKSPACE="$(cd "$1" && pwd)"
else
    WORKSPACE="$(pwd)"
fi

echo "==> Repodesk container run"
echo "    image:     ${IMAGE_NAME}:${IMAGE_TAG}"
echo "    container: ${CONTAINER_NAME}"
echo "    workspace: ${WORKSPACE}"
echo ""

# Verify podman is available.
if ! command -v podman > /dev/null 2>&1; then
    echo "ERROR: podman not found on PATH" >&2
    exit 1
fi

# Verify image exists.
if ! podman image exists "${IMAGE_NAME}:${IMAGE_TAG}" 2>/dev/null; then
    echo "ERROR: image ${IMAGE_NAME}:${IMAGE_TAG} not found." >&2
    echo "Build it first with: ./podman_build.sh" >&2
    exit 1
fi

# Verify workspace exists and is a directory.
if [ ! -d "${WORKSPACE}" ]; then
    echo "ERROR: workspace path does not exist or is not a directory: ${WORKSPACE}" >&2
    exit 1
fi

# Stop existing container if running (reentrant).
if podman container exists "${CONTAINER_NAME}" 2>/dev/null; then
    echo "==> Stopping existing container '${CONTAINER_NAME}' ..."
    podman stop "${CONTAINER_NAME}" 2>/dev/null || true
    echo "==> Removing existing container '${CONTAINER_NAME}' ..."
    podman rm "${CONTAINER_NAME}" 2>/dev/null || true
fi

# Detect SELinux and apply :z volume label if needed.
VOLUME_OPTS=""
if command -v getenforce > /dev/null 2>&1; then
    if getenforce 2>/dev/null | grep -q "Enforcing\|Permissive"; then
        VOLUME_OPTS=":z"
    fi
fi

echo "==> Starting container ..."
echo ""

# Run interactively with TTY.
# --rm is intentionally omitted so the container can be inspected after exit.
# Use podman_run.sh again to clean up and restart (reentrant).
podman run \
    --interactive \
    --tty \
    --name "${CONTAINER_NAME}" \
    --userns=keep-id \
    --group-add=keep-groups \
    --volume "${WORKSPACE}:/workspace${VOLUME_OPTS}" \
    "${IMAGE_NAME}:${IMAGE_TAG}"

EXIT_CODE=$?

echo ""
if [ "${EXIT_CODE}" -eq 0 ]; then
    echo "==> Repodesk exited cleanly."
else
    echo "==> Repodesk exited with code ${EXIT_CODE}."
fi

# Clean up container after exit.
podman rm "${CONTAINER_NAME}" 2>/dev/null || true
