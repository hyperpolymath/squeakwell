K9!
# SPDX-License-Identifier: MPL-2.0
# deploy.k9.ncl — squeakwell deployment component (Hunt level)
#
# k9-svc deployment specification with full pedigree (L1-L5).
# Security Level: 'Hunt (requires cryptographic handshake for execution).
#
# WARNING: This component can execute shell commands!
# It requires explicit authorisation via the Leash system.
#
# squeakwell is a command-line recovery tool, not a long-running service:
# the "deployment" here builds and verifies the container image that
# carries the CLI and runs it as a batch job against a VeriSimDB instance.
#
# Usage:
#   k9-svc validate container/deploy.k9.ncl
#   k9-svc deploy container/deploy.k9.ncl --env production
#
# Note: the first line is the K9 magic marker required by the K9 validator;
# plain `nickel typecheck` does not apply to K9 files (k9-svc strips the
# marker before handing the record to Nickel).

{
  # ─────────────────────────────────────────────────────────────
  # Pedigree (self-description across five layers)
  # ─────────────────────────────────────────────────────────────
  pedigree = {
    schema_version = "1.0.0",
    component_type = "deployment",

    # L1: The Snout — Identity
    metadata = {
      name = "squeakwell-deploy",
      version = "0.1.0",
      breed = "application/vnd.k9+nickel",
      magic_number = "K9!",
      description = "squeakwell container build, verify and batch-run component (Hunt level)",
      author = "Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>",
    },

    # L2: The Scent — Target Environment
    target = {
      os = 'Linux,
      is_edge = false,
      requires_podman = true,
      min_memory_mb = 256,
    },

    # L3: The Leash — Security
    security = {
      leash = 'Hunt,
      trust_level = "full-system-access",
      allow_network = true,
      allow_filesystem_write = true,
      allow_subprocess = true,
      signature_required = true,
      # In production, replace with a real Ed25519 signature.
      signature = "PLACEHOLDER-SIGNATURE-REQUIRED-FOR-HUNT",
    },

    # L4: The Gut — Self-Validation
    validation = {
      checksum = "sha256:placeholder",
      pedigree_version = "1.0.0",
      hunt_authorized = false,  # Must be set true after handshake
    },

    warnings = [
      "This component has full system access",
      "Only run from trusted sources with verified signatures",
      "Review all recipes before execution",
    ],
    side_effects = [
      "Builds the squeakwell container image via container/ct-build.sh",
      "Runs one recovery batch with podman against the VeriSimDB named by VERISIMDB_URL",
      "Removes ONLY the named batch volume squeakwell-data-<BATCH_ID> on rollback (never other batches)",
      "Writes recovery working directories under the mounted /data volume",
    ],

    # L5: The Muscle — Deployment Recipes
    recipes = {
      install = "container/ct-build.sh",
      validate = "podman run --rm ghcr.io/hyperpolymath/squeakwell:latest --version",
      deploy = "podman run --rm -v squeakwell-data-$BATCH_ID:/data -e VERISIMDB_URL ghcr.io/hyperpolymath/squeakwell:latest recover --workdir /data",
      migrate = "container/ct-build.sh && podman run --rm -v squeakwell-data-$BATCH_ID:/data -e VERISIMDB_URL ghcr.io/hyperpolymath/squeakwell:latest recover --workdir /data",
    },
  },

  # ─────────────────────────────────────────────────────────────
  # Deployment configuration
  # ─────────────────────────────────────────────────────────────
  deployment = {
    # Target environments (dev / staging / production)
    environments = {
      dev = { replicas = 1, memory = "256Mi", cpu = "100m", image_tag = "dev" },
      staging = { replicas = 1, memory = "512Mi", cpu = "250m", image_tag = "staging" },
      production = { replicas = 1, memory = "1Gi", cpu = "500m", image_tag = "latest" },
    },

    # Container configuration (batch job: no listening port)
    container = {
      image = "ghcr.io/hyperpolymath/squeakwell",
      entrypoint = "squeakwell",
      data_volume = "/data",
      # One named volume per batch: squeakwell-data-<BATCH_ID>. Rollback
      # removes only that volume, so earlier batches are never destroyed.
      batch_volume_prefix = "squeakwell-data-",
      # The VeriSimDB instance the recovery writes to is supplied at run time
      # through this environment variable (an https:// URL in production; TLS
      # is terminated by the VeriSimDB deployment, not by squeakwell).
      verisimdb_url_env = "VERISIMDB_URL",
    },

    # Deployment strategy
    strategy = { type = "recreate" },
  },

  # ─────────────────────────────────────────────────────────────
  # Scripts (executed at Hunt level)
  # ─────────────────────────────────────────────────────────────
  scripts = {
    # Pre-deployment validation: the image exists and the CLI answers.
    # container/compose.toml is the selur-compose equivalent of this component
    # (one batch, one volume); it is NOT invoked here.
    pre_deploy = m%"
#!/bin/sh
set -eu
TAG="${IMAGE_TAG:-latest}"
echo "K9: Pre-deployment validation for squeakwell:$TAG..."
podman image inspect "ghcr.io/hyperpolymath/squeakwell:$TAG" >/dev/null
podman run --rm "ghcr.io/hyperpolymath/squeakwell:$TAG" --version
echo "K9: Validation passed."
"%,

    # Deployment: build and verify the image, then run one recovery batch
    # against the VeriSimDB named by VERISIMDB_URL (must be set; https in
    # production). Each batch gets its own named volume
    # squeakwell-data-<BATCH_ID> (BATCH_ID defaults to a UTC timestamp) so a
    # rollback can discard exactly one batch.
    deploy = m%"
#!/bin/sh
set -eu
ENV="${1:-dev}"
TAG="${IMAGE_TAG:-latest}"
BATCH_ID="${BATCH_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
VOLUME="squeakwell-data-$BATCH_ID"
: "${VERISIMDB_URL:?VERISIMDB_URL must be set (https:// in production)}"
echo "K9: Building squeakwell image for $ENV..."
cd container && ./ct-build.sh && cd ..
echo "K9: Running recovery batch $BATCH_ID against $VERISIMDB_URL (volume $VOLUME)..."
podman run --rm -v "$VOLUME:/data" -e VERISIMDB_URL \
  "ghcr.io/hyperpolymath/squeakwell:$TAG" recover --workdir /data
echo "K9: Batch $BATCH_ID for $ENV complete. Roll back with: BATCH_ID=$BATCH_ID k9-svc rollback"
"%,

    # Rollback: a batch job leaves no running service; rolling back means
    # discarding the working volume of ONE named batch. BATCH_ID is required
    # so that no other batch can be removed by accident.
    rollback = m%"
#!/bin/sh
set -eu
: "${BATCH_ID:?BATCH_ID must name the batch to roll back (see the deploy output)}"
VOLUME="squeakwell-data-$BATCH_ID"
podman volume inspect "$VOLUME" >/dev/null
echo "K9: Rolling back squeakwell batch $BATCH_ID (removing volume $VOLUME only)..."
podman volume rm "$VOLUME"
echo "K9: Rollback of $BATCH_ID complete. Other batch volumes are untouched."
"%,
  },

  # Security check: this component requires Hunt level
  required_level = 'Hunt,

  # Warning for users
  warning = m%"
WARNING: This is a Hunt-level component.

It can execute shell commands and modify your system.
Before running, ensure you have:

1. Reviewed the deployment scripts above
2. Verified the signature (when implemented)
3. Explicitly authorised Hunt-level execution

Run with: k9-svc authorize container/deploy.k9.ncl && k9-svc deploy container/deploy.k9.ncl
"%,
}
