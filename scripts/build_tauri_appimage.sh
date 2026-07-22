#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
web_dir="$repo_root/web"
tauri_dir="$web_dir/src-tauri"

profile="release"
for arg in "$@"; do
  case "$arg" in
    --debug|-d)
      profile="debug"
      ;;
  esac
done

if [[ -x /usr/bin/pkg-config ]]; then
  export PKG_CONFIG="${PKG_CONFIG:-/usr/bin/pkg-config}"
fi

echo "Building Tauri AppImage with normal Tauri bundler..."
if (cd "$web_dir" && npm run tauri -- build --bundles appimage "$@"); then
  exit 0
fi

bundle_dir="$tauri_dir/target/$profile/bundle/appimage"
appdir="$bundle_dir/RedShield Architect.AppDir"
linuxdeploy_appimage="${TAURI_LINUXDEPLOY_APPIMAGE:-$HOME/.cache/tauri/linuxdeploy-x86_64.AppImage}"
tauri_cache="${TAURI_CACHE_DIR:-$HOME/.cache/tauri}"
extract_dir="$tauri_dir/target/appimage-linuxdeploy"

if [[ ! -d "$appdir" ]]; then
  echo "Tauri AppImage bundling failed and no AppDir was generated at $appdir" >&2
  exit 1
fi

if [[ ! -x "$linuxdeploy_appimage" ]]; then
  echo "Cannot find executable linuxdeploy AppImage at $linuxdeploy_appimage" >&2
  exit 1
fi

if [[ ! -x "$tauri_cache/linuxdeploy-plugin-gtk.sh" ]]; then
  echo "Cannot find Tauri linuxdeploy GTK plugin at $tauri_cache/linuxdeploy-plugin-gtk.sh" >&2
  exit 1
fi

strip_bin="${STRIP:-/usr/bin/strip}"
if [[ ! -x "$strip_bin" ]]; then
  echo "Cannot find executable strip at $strip_bin" >&2
  exit 1
fi

echo "Normal Tauri AppImage bundling failed."
echo "Retrying with extracted linuxdeploy and system strip: $strip_bin"

rm -rf "$extract_dir" "$extract_dir.tmp"
mkdir -p "$extract_dir.tmp"
(
  cd "$extract_dir.tmp"
  "$linuxdeploy_appimage" --appimage-extract >/dev/null
)
mv "$extract_dir.tmp/squashfs-root" "$extract_dir"
rm -rf "$extract_dir.tmp"

cp "$strip_bin" "$extract_dir/usr/bin/strip"

(
  cd "$bundle_dir"
  PATH="$tauri_cache:/usr/bin:/bin:$PATH" \
    PKG_CONFIG="${PKG_CONFIG:-pkg-config}" \
    "$extract_dir/usr/bin/linuxdeploy" \
      --verbosity 1 \
      --appdir "$appdir" \
      --plugin gtk \
      --output appimage
)

product_name="$(cd "$web_dir" && node -p "require('./src-tauri/tauri.conf.json').productName")"
version="$(cd "$web_dir" && node -p "require('./src-tauri/tauri.conf.json').version")"
machine="$(uname -m)"
case "$machine" in
  x86_64)
    tauri_arch="amd64"
    appimage_arch="x86_64"
    ;;
  aarch64)
    tauri_arch="arm64"
    appimage_arch="aarch64"
    ;;
  *)
    tauri_arch="$machine"
    appimage_arch="$machine"
    ;;
esac

direct_name="${product_name// /_}-${appimage_arch}.AppImage"
tauri_name="${product_name}_${version}_${tauri_arch}.AppImage"

if [[ -f "$bundle_dir/$direct_name" && "$direct_name" != "$tauri_name" ]]; then
  mv -f "$bundle_dir/$direct_name" "$bundle_dir/$tauri_name"
fi

echo "AppImage bundle ready: $bundle_dir/$tauri_name"
