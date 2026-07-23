#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/smoke_fedora_rpm_lifecycle.sh --rpm-dir <dir> [--previous-rpm-dir <dir>] [--install-root <dir>]

Runs Fedora RPM lifecycle smoke checks in a clean DNF installroot:
  - install the selected redshield-architect-workbench RPM
  - verify package metadata and installed desktop/metainfo/icon files
  - optionally install a previous RPM and upgrade to the selected RPM
  - remove the package and verify it is gone

The helper uses sudo when it is not already running as root. It does not install
the package into the host root.
USAGE
}

rpm_dir=""
previous_rpm_dir=""
install_root=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --rpm-dir)
      rpm_dir="${2:-}"
      shift 2
      ;;
    --previous-rpm-dir)
      previous_rpm_dir="${2:-}"
      shift 2
      ;;
    --install-root)
      install_root="${2:-}"
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

if [[ -z "$rpm_dir" ]]; then
  echo "Missing required --rpm-dir <dir>" >&2
  usage >&2
  exit 2
fi

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

dnf_command=""
if command -v dnf5 >/dev/null 2>&1; then
  dnf_command="dnf5"
elif command -v dnf >/dev/null 2>&1; then
  dnf_command="dnf"
else
  echo "Missing required command: dnf5 or dnf" >&2
  exit 1
fi

for command_name in git rpm sha256sum; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 1
  fi
done

sudo_prefix=()
if [[ "$(id -u)" != "0" ]]; then
  if ! sudo -n true >/dev/null 2>&1; then
    echo "This smoke check needs root for a DNF installroot; sudo -n is unavailable." >&2
    exit 1
  fi
  sudo_prefix=(sudo -n)
fi

rpm_dir="$(cd "$rpm_dir" && pwd)"
current_rpm="$(find "$rpm_dir" -maxdepth 1 -type f \
  -name 'redshield-architect-workbench-[0-9]*.x86_64.rpm' \
  ! -name '*debuginfo*' \
  ! -name '*debugsource*' \
  | sort -V | tail -n 1)"

if [[ -z "$current_rpm" ]]; then
  echo "No redshield-architect-workbench binary RPM found in $rpm_dir" >&2
  exit 1
fi

previous_rpm=""
if [[ -n "$previous_rpm_dir" ]]; then
  previous_rpm_dir="$(cd "$previous_rpm_dir" && pwd)"
  previous_rpm="$(find "$previous_rpm_dir" -maxdepth 1 -type f \
    -name 'redshield-architect-workbench-[0-9]*.x86_64.rpm' \
    ! -name '*debuginfo*' \
    ! -name '*debugsource*' \
    | sort -V | tail -n 1)"
  if [[ -z "$previous_rpm" ]]; then
    echo "No previous redshield-architect-workbench binary RPM found in $previous_rpm_dir" >&2
    exit 1
  fi
fi

install_root="${install_root:-$repo_root/target/fedora-rpm-smoke/$(basename "$rpm_dir")/root}"
evidence_dir="$(dirname "$install_root")"
cache_dir="$evidence_dir/dnf-cache"
report="$evidence_dir/SMOKE.md"
releasever="$(rpm -E %fedora)"
package_name="redshield-architect-workbench"

"${sudo_prefix[@]}" rm -rf "$install_root" "$cache_dir"
mkdir -p "$evidence_dir"
"${sudo_prefix[@]}" mkdir -p "$install_root" "$cache_dir"

dnf_base=(
  "$dnf_command"
  --use-host-config
  --installroot "$install_root"
  --releasever "$releasever"
  --setopt "install_weak_deps=False"
  --setopt "cachedir=$cache_dir"
  --setopt "keepcache=False"
  -y
)

if [[ -n "$previous_rpm" ]]; then
  "${sudo_prefix[@]}" "${dnf_base[@]}" install "$previous_rpm"
  "${sudo_prefix[@]}" rpm --root "$install_root" -q "$package_name" >/dev/null
  "${sudo_prefix[@]}" "${dnf_base[@]}" upgrade "$current_rpm"
else
  "${sudo_prefix[@]}" "${dnf_base[@]}" install "$current_rpm"
fi

"${sudo_prefix[@]}" rpm --root "$install_root" -q "$package_name" >/dev/null

for installed_path in \
  /usr/bin/redshield-architect-workbench \
  /usr/share/applications/com.redshield.architect.desktop \
  /usr/share/metainfo/com.redshield.architect.metainfo.xml \
  /usr/share/icons/hicolor/32x32/apps/redshield-architect-workbench.png \
  /usr/share/icons/hicolor/128x128/apps/redshield-architect-workbench.png \
  /usr/share/icons/hicolor/256x256/apps/redshield-architect-workbench.png; do
  if [[ ! -e "$install_root$installed_path" ]]; then
    echo "Expected installed path missing: $installed_path" >&2
    exit 1
  fi
done

"${sudo_prefix[@]}" rpm --root "$install_root" -e "$package_name"
if "${sudo_prefix[@]}" rpm --root "$install_root" -q "$package_name" >/dev/null 2>&1; then
  echo "Package still installed after remove: $package_name" >&2
  exit 1
fi

{
  echo "# Fedora RPM Lifecycle Smoke"
  echo
  echo "- RPM directory: \`$rpm_dir\`"
  echo "- Current RPM: \`$(basename "$current_rpm")\`"
  if [[ -n "$previous_rpm" ]]; then
    echo "- Previous RPM: \`$(basename "$previous_rpm")\`"
  fi
  echo "- Install root: \`$install_root\`"
  echo "- DNF: \`$dnf_command\`"
  echo "- Fedora releasever: \`$releasever\`"
  echo "- Generated: \`$(date -u +%Y-%m-%dT%H:%M:%SZ)\`"
  echo
  echo "## Checks"
  echo
  echo "- install transaction passed"
  if [[ -n "$previous_rpm" ]]; then
    echo "- upgrade transaction passed"
  else
    echo "- upgrade transaction skipped; no previous RPM directory was supplied"
  fi
  echo "- package query passed after install"
  echo "- expected executable, desktop file, AppStream metadata, and icons existed"
  echo "- remove transaction passed"
  echo "- package query confirmed removal"
} > "$report"

echo "Fedora RPM lifecycle smoke checks passed."
echo "Evidence: $report"
