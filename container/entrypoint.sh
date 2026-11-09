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
# Never echo VERISIMDB_URL itself: a URL may carry userinfo credentials or a
# token in its query string, and container logs are widely readable. Log the
# scheme and host only (userinfo, path and query stripped).
redact_url() {
    _url="$1"
    case "$_url" in
        *://*) _scheme="${_url%%://*}://"; _rest="${_url#*://}" ;;
        *)     _scheme="";                 _rest="$_url" ;;
    esac
    # Cut path, query and fragment first: an '@' inside them (e.g.
    # ?token=a@secret) must not be mistaken for the userinfo separator.
    _rest="${_rest%%/*}"
    _rest="${_rest%%\?*}"
    _rest="${_rest%%#*}"
    case "$_rest" in
        *@*) _rest="${_rest##*@}" ;;
        *) ;;
    esac
    printf '%s%s\n' "$_scheme" "$_rest"
}

if [ -z "${VERISIMDB_URL+set}" ]; then
    echo "  VeriSimDB: <unset> WARNING: the client falls back to http://localhost:8080 and"
    echo "             persistence errors are not fatal, so session and phase events"
    echo "             may not be recorded anywhere. Set VERISIMDB_URL for a real batch."
elif [ -z "${VERISIMDB_URL}" ]; then
    echo "  VeriSimDB: <set but empty> WARNING: VERISIMDB_URL is set to the empty string"
    echo "             (usually a compose file passing through an unset variable)."
    echo "             The client treats it as unset and falls back to http://localhost:8080;"
    echo "             session and phase events may not be recorded anywhere."
else
    echo "  VeriSimDB: $(redact_url "${VERISIMDB_URL}") (host only; userinfo, path and query not logged)"
fi
if [ -n "${SQUEAKWELL_BATCH_ID:-}" ]; then
    echo "  Batch:     ${SQUEAKWELL_BATCH_ID}"
else
    echo "  Batch:     <none> (session id derived from the workdir path; set SQUEAKWELL_BATCH_ID"
    echo "             so concurrent batches do not share one VeriSimDB session)"
fi

# The VOLUME directive in the Containerfile creates /data, but a bind-mount
# might replace it with an empty directory owned by root.
if [ -d "${DATA_DIR}" ] && [ ! -w "${DATA_DIR}" ]; then
    echo "WARNING: ${DATA_DIR} is not writable by $(whoami)"
fi

exec /app/squeakwell "$@"
