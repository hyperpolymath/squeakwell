#!/bin/sh
# SPDX-License-Identifier: MPL-2.0
# squeakwell container entrypoint
#
# squeakwell is a batch recovery CLI, not a long-running service. The
# entrypoint checks the working volume, logs the run parameters, and execs
# the binary with the caller's arguments so that PID 1 is squeakwell and
# signals from `podman stop` reach it directly.

set -e

DATA_DIR="${APP_DATA_DIR:-/data}"

echo "Starting squeakwell..."
echo "  Data:      ${DATA_DIR}"
echo "  Log:       ${APP_LOG_FORMAT:-json}"
echo "  VeriSimDB: ${VERISIMDB_URL:-<unset: recovery runs offline against the workdir only>}"

# The VOLUME directive in the Containerfile creates /data, but a bind-mount
# might replace it with an empty directory owned by root.
if [ -d "${DATA_DIR}" ] && [ ! -w "${DATA_DIR}" ]; then
    echo "WARNING: ${DATA_DIR} is not writable by $(whoami)"
fi

exec /app/squeakwell "$@"
