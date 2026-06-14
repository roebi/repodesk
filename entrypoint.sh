#!/bin/sh
# entrypoint.sh - Repodesk container entrypoint
# Repository must be mounted at /workspace.
# Usage: podman run -it -v /path/to/repo:/workspace repodesk

set -e

WORKSPACE="/workspace"
REPODESK_BIN="/usr/local/bin/repodesk"

# Verify the binary exists (guards against failed COPY in Dockerfile).
if [ ! -x "${REPODESK_BIN}" ]; then
    echo "ERROR: repodesk binary not found at ${REPODESK_BIN}" >&2
    exit 1
fi

# Verify workspace is mounted and is a directory.
if [ ! -d "${WORKSPACE}" ]; then
    echo "ERROR: /workspace is not mounted." >&2
    echo "Usage: podman run -it -v /path/to/repo:/workspace repodesk" >&2
    exit 1
fi

# Set a default git identity if not already configured.
# Required for git commit operations inside the container.
if ! git config --global user.email > /dev/null 2>&1; then
    git config --global user.email "repodesk@container"
    git config --global user.name "Repodesk"
fi

# Mark workspace as safe for git (podman volume mount ownership).
git config --global --add safe.directory "${WORKSPACE}"

cd "${WORKSPACE}"

# Launch Repodesk with the workspace as repo root.
exec "${REPODESK_BIN}" "${WORKSPACE}"
