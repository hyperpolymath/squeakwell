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
#   nickel typecheck container/deploy.k9.ncl
#   k9-svc validate container/deploy.k9.ncl
#   k9-svc deploy container/deploy.k9.ncl --env production

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
      "Starts or stops the compose stack described in container/compose.toml",
      "Writes recovery working directories under the mounted /data volume",
    ],

    # L5: The Muscle — Deployment Recipes
    recipes = {
      install = "just container-build",
      validate = "just container-verify",
      deploy = "just container-up",
      migrate = "just container-build && just container-up",
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
      # The VeriSimDB instance the recovery writes to (overridable at run time).
      verisimdb_url = "http://verisimdb:8080",
    },

    # Deployment strategy
    strategy = { type = "recreate" },
  },

  # ─────────────────────────────────────────────────────────────
  # Scripts (executed at Hunt level)
  # ─────────────────────────────────────────────────────────────
  scripts = {
    # Pre-deployment validation
    pre_deploy = m%"
#!/bin/sh
set -eu
echo "K9: Pre-deployment validation for squeakwell..."
cd container && selur-compose verify || podman compose --file compose.toml config
echo "K9: Validation passed."
"%,

    # Deployment script
    deploy = m%"
#!/bin/sh
set -eu
ENV="${1:-dev}"
echo "K9: Deploying squeakwell to $ENV environment..."
cd container
./ct-build.sh
selur-compose up --detach || podman compose --file compose.toml up --detach
echo "K9: Deployment to $ENV complete."
"%,

    # Rollback script
    rollback = m%"
#!/bin/sh
set -eu
echo "K9: Rolling back squeakwell deployment..."
cd container
selur-compose down || podman compose --file compose.toml down
echo "K9: Rollback complete."
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
