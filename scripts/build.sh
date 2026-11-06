#!/usr/bin/env bash
# Build complet de gninou : serveur Kotlin (Gradle) -> jar embarqué -> client (Cargo).
#
# Usage :
#   scripts/build.sh            # build debug
#   scripts/build.sh --release  # build optimisé
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODE="${1:-}"

echo "==> 1/2 Compilation du serveur (Gradle)"
if [ -x "$ROOT/server/gradlew" ]; then
    (cd "$ROOT/server" && ./gradlew jar)
else
    (cd "$ROOT/server" && gradle jar)
fi

echo "==> 2/2 Compilation du client (Cargo, serveur embarqué)"
CARGO_FLAGS=()
[ "$MODE" = "--release" ] && CARGO_FLAGS+=(--release)
GNINOU_BUILD_SERVER=1 cargo build --manifest-path "$ROOT/client/Cargo.toml" "${CARGO_FLAGS[@]}"

echo "==> Terminé."
