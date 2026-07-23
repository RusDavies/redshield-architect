#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
spec_file="$repo_root/packaging/fedora/redshield-architect-workbench.spec"
version="$(awk '/^Version:/ { print $2; exit }' "$spec_file")"
source_name="redshield-architect-$version"
topdir="$repo_root/target/rpmbuild"
rpm_out="$repo_root/target/rpm"
cache_dir="$repo_root/target/rpm-cache"

required_commands=(rpmbuild cargo npm appstreamcli desktop-file-validate)
for command_name in "${required_commands[@]}"; do
  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Missing required command: $command_name" >&2
    exit 1
  fi
done

if [[ -x /usr/bin/pkg-config ]]; then
  export PKG_CONFIG="${PKG_CONFIG:-/usr/bin/pkg-config}"
fi

rm -rf "$topdir" "$rpm_out"
mkdir -p "$topdir"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS} "$rpm_out" "$cache_dir"

tar -C "$repo_root" \
  --exclude .git \
  --exclude target \
  --exclude web/dist \
  --exclude web/node_modules \
  --exclude web/src-tauri/target \
  --transform "s,^,$source_name/," \
  -czf "$topdir/SOURCES/$source_name.tar.gz" \
  .

cp "$spec_file" "$topdir/SPECS/"

rpmbuild \
  --define "_topdir $topdir" \
  --define "_smp_mflags -j$(nproc)" \
  -ba "$topdir/SPECS/$(basename "$spec_file")"

find "$topdir/RPMS" "$topdir/SRPMS" -type f \( -name '*.rpm' -o -name '*.src.rpm' \) \
  -exec cp {} "$rpm_out/" \;

echo "RPM artifacts copied to $rpm_out"
