#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/prepare_fedora_rpm_review_candidate.sh --tag <git-tag> [--output-dir <dir>]

Creates Fedora RPM review-candidate source evidence:
  - tagged upstream source archive
  - cargo vendor archive and cargo dependency inventory
  - npm dependency cache archive and npm dependency inventory
  - SHA-256 checksums
  - license evidence summary

This helper prepares declared source inputs. It does not claim the RPM is
Fedora-review-ready by itself; the candidate still needs an offline rpmbuild or
mock validation using only the generated inputs.
USAGE
}

tag=""
output_dir=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      tag="${2:-}"
      shift 2
      ;;
    --output-dir)
      output_dir="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$tag" ]]; then
  echo "Missing required --tag <git-tag>" >&2
  usage >&2
  exit 2
fi

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

required_commands=(cargo git npm python3 sha256sum tar)
for command_name in "${required_commands[@]}"; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 1
  fi
done

if ! git rev-parse --verify --quiet "$tag^{commit}" >/dev/null; then
  echo "Tag not found: $tag" >&2
  exit 1
fi

if [[ -n "$(git status --porcelain)" ]]; then
  echo "Working tree must be clean before creating release evidence." >&2
  exit 1
fi

spec_file="$repo_root/packaging/fedora/redshield-architect-workbench.spec"
version="$(awk '/^Version:/ { print $2; exit }' "$spec_file")"
release_root="${output_dir:-$repo_root/target/fedora-rpm-review/$tag}"
work_dir="$release_root/work"
artifact_dir="$release_root/artifacts"
manifest_dir="$release_root/manifests"
source_name="redshield-architect-$version"

rm -rf "$release_root"
mkdir -p "$artifact_dir" "$manifest_dir" "$work_dir"

source_archive="$artifact_dir/$source_name.tar.gz"
cargo_vendor_archive="$artifact_dir/redshield-architect-$version-cargo-vendor.tar.gz"
npm_cache_archive="$artifact_dir/redshield-architect-$version-npm-cache.tar.gz"

git archive \
  --format=tar.gz \
  --prefix="$source_name/" \
  --output="$source_archive" \
  "$tag"

cargo_vendor_dir="$work_dir/cargo-vendor"
mkdir -p "$cargo_vendor_dir"
cargo vendor \
  --locked \
  --versioned-dirs \
  --sync "$repo_root/web/src-tauri/Cargo.toml" \
  "$cargo_vendor_dir" \
  > "$manifest_dir/cargo-vendor-config.toml"
tar -C "$work_dir" -czf "$cargo_vendor_archive" cargo-vendor

npm_work_dir="$work_dir/npm-web"
mkdir -p "$npm_work_dir"
cp "$repo_root/web/package.json" "$repo_root/web/package-lock.json" "$npm_work_dir/"
npm ci \
  --prefix "$npm_work_dir" \
  --ignore-scripts \
  --cache "$work_dir/npm-cache" \
  --prefer-online \
  --no-audit \
  --fund=false
tar -C "$work_dir" -czf "$npm_cache_archive" npm-cache

cargo metadata \
  --locked \
  --format-version 1 \
  --all-features \
  > "$manifest_dir/cargo-metadata.json"

cargo metadata \
  --locked \
  --format-version 1 \
  --manifest-path "$repo_root/web/src-tauri/Cargo.toml" \
  > "$manifest_dir/cargo-tauri-metadata.json"

python3 - \
  "$manifest_dir/cargo-metadata.json" \
  "$manifest_dir/cargo-tauri-metadata.json" \
  "$manifest_dir/cargo-dependency-inventory.tsv" \
  "$manifest_dir/cargo-license-summary.tsv" <<'PY'
import json
import sys
from collections import Counter

root_metadata_path, tauri_metadata_path, inventory_path, license_path = sys.argv[1:5]
packages_by_id = {}
for metadata_path in (root_metadata_path, tauri_metadata_path):
    metadata = json.load(open(metadata_path, encoding="utf-8"))
    for package in metadata["packages"]:
        packages_by_id[package["id"]] = package

packages = sorted(packages_by_id.values(), key=lambda p: (p["name"], p["version"], p["id"]))
licenses = Counter()

with open(inventory_path, "w", encoding="utf-8") as inventory:
    inventory.write("name\tversion\tlicense\trepository\n")
    for package in packages:
        license_expr = package.get("license") or "UNKNOWN"
        licenses[license_expr] += 1
        inventory.write(
            f"{package['name']}\t{package['version']}\t{license_expr}\t{package.get('repository') or ''}\n"
        )

with open(license_path, "w", encoding="utf-8") as summary:
    summary.write("license\tpackage_count\n")
    for license_expr, count in sorted(licenses.items()):
        summary.write(f"{license_expr}\t{count}\n")
PY

python3 - "$repo_root/web/package-lock.json" "$manifest_dir/npm-dependency-inventory.tsv" "$manifest_dir/npm-license-summary.tsv" <<'PY'
import json
import sys
from collections import Counter

lock_path, inventory_path, license_path = sys.argv[1:4]
lock = json.load(open(lock_path, encoding="utf-8"))
packages = lock.get("packages", {})
licenses = Counter()

with open(inventory_path, "w", encoding="utf-8") as inventory:
    inventory.write("name\tversion\tlicense\tresolved\tintegrity\n")
    for path, package in sorted(packages.items()):
        if not path:
            continue
        name = package.get("name") or path.removeprefix("node_modules/")
        version = package.get("version") or ""
        license_expr = package.get("license") or "UNKNOWN"
        resolved = package.get("resolved") or ""
        integrity = package.get("integrity") or ""
        licenses[license_expr] += 1
        inventory.write(f"{name}\t{version}\t{license_expr}\t{resolved}\t{integrity}\n")

with open(license_path, "w", encoding="utf-8") as summary:
    summary.write("license\tpackage_count\n")
    for license_expr, count in sorted(licenses.items()):
        summary.write(f"{license_expr}\t{count}\n")
PY

{
  echo "# Fedora RPM Review Candidate Evidence"
  echo
  echo "- Tag: \`$tag\`"
  echo "- Commit: \`$(git rev-parse "$tag^{commit}")\`"
  echo "- Version: \`$version\`"
  echo "- Generated: \`$(date -u +%Y-%m-%dT%H:%M:%SZ)\`"
  echo
  echo "## Artifacts"
  echo
  for artifact in "$artifact_dir"/*; do
    echo "- \`$(basename "$artifact")\`"
  done
  echo
  echo "## Manifests"
  echo
  for manifest in "$manifest_dir"/*; do
    echo "- \`$(basename "$manifest")\`"
  done
  echo
  echo "## Review Notes"
  echo
  echo "- The npm archive is an npm cache generated from \`web/package-lock.json\`, not committed \`node_modules\`."
  echo "- The Cargo archive is generated by \`cargo vendor --locked --versioned-dirs\`."
  echo "- Unknown or ambiguous license entries must be resolved before calling the RPM Fedora-review-ready."
  echo "- A later offline \`rpmbuild\` or \`mock\` validation must consume only these declared inputs."
} > "$release_root/EVIDENCE.md"

(
  cd "$release_root"
  sha256sum artifacts/* manifests/* EVIDENCE.md > SHA256SUMS
)

echo "Fedora RPM review-candidate evidence written to $release_root"
echo "Checksums: $release_root/SHA256SUMS"
