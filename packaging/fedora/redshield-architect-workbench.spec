Name:           redshield-architect-workbench
Version:        0.1.0
Release:        0.1%{?dist}
Summary:        Requirements and architecture modeling workbench
%bcond_with     offline_sources

License:        MIT
URL:            https://github.com/RusDavies/redshield-architect
Source0:        redshield-architect-%{version}.tar.gz
%if %{with offline_sources}
Source1:        redshield-architect-%{version}-cargo-vendor.tar.gz
Source2:        redshield-architect-%{version}-npm-cache.tar.gz
%endif

BuildRequires:  appstream
BuildRequires:  atk-devel
BuildRequires:  cairo-devel
BuildRequires:  cargo
BuildRequires:  dbus-devel
BuildRequires:  desktop-file-utils
BuildRequires:  gdk-pixbuf2-devel
BuildRequires:  glib2-devel
BuildRequires:  gtk3-devel
BuildRequires:  javascriptcoregtk4.1-devel
BuildRequires:  libsoup3-devel
BuildRequires:  nodejs
BuildRequires:  nodejs-npm
BuildRequires:  openssl-devel
BuildRequires:  pango-devel
BuildRequires:  pkgconf-pkg-config
BuildRequires:  rust
BuildRequires:  webkit2gtk4.1-devel

Requires:       gtk3
Requires:       libsoup3
Requires:       webkit2gtk4.1

%description
RedShield Architect is a Linux-first workbench for requirements,
architecture modeling, UML views, traceability, and AI-assisted design
review. This is the initial Fedora-oriented development RPM spec for the
Tauri workbench.

This spec is intended for local development builds. Fedora-review-ready
packaging still needs full Fedora policy review, but it can be validated
against generated offline Cargo and npm source inputs.

%prep
%autosetup -n redshield-architect-%{version}
%if %{with offline_sources}
tar -C %{_builddir} -xzf %{SOURCE1}
tar -C %{_builddir} -xzf %{SOURCE2}
mkdir -p .cargo
cat > .cargo/config.toml <<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "%{_builddir}/cargo-vendor"

[net]
offline = true
EOF
%endif

%build
export PKG_CONFIG=/usr/bin/pkg-config
export CARGO_HOME="${RPM_CARGO_HOME:-%{_topdir}/../rpm-cache/cargo-home}"
%if %{with offline_sources}
export CARGO_NET_OFFLINE=true
export npm_config_cache="%{_builddir}/npm-cache"
npm ci --prefix web --offline --ignore-scripts --no-audit --fund=false
cargo_flags="--offline"
%else
export npm_config_cache="${RPM_NPM_CACHE:-%{_topdir}/../rpm-cache/npm-cache}"
npm ci --prefix web
cargo_flags=""
%endif

npm run --prefix web build
cargo build --manifest-path web/src-tauri/Cargo.toml --release --locked $cargo_flags

%install
install -Dm0755 web/src-tauri/target/release/redshield-architect-workbench \
  %{buildroot}%{_bindir}/redshield-architect-workbench

install -Dm0644 packaging/fedora/com.redshield.architect.desktop \
  %{buildroot}%{_datadir}/applications/com.redshield.architect.desktop

install -Dm0644 web/src-tauri/metainfo/com.redshield.architect.metainfo.xml \
  %{buildroot}%{_metainfodir}/com.redshield.architect.metainfo.xml

install -Dm0644 web/src-tauri/icons/32x32.png \
  %{buildroot}%{_datadir}/icons/hicolor/32x32/apps/redshield-architect-workbench.png
install -Dm0644 web/src-tauri/icons/128x128.png \
  %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/redshield-architect-workbench.png
install -Dm0644 web/src-tauri/icons/128x128@2x.png \
  %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/redshield-architect-workbench.png

%check
export PKG_CONFIG=/usr/bin/pkg-config
export CARGO_HOME="${RPM_CARGO_HOME:-%{_topdir}/../rpm-cache/cargo-home}"
%if %{with offline_sources}
export CARGO_NET_OFFLINE=true
cargo_flags="--offline"
%else
cargo_flags=""
%endif
cargo test --locked $cargo_flags
desktop-file-validate %{buildroot}%{_datadir}/applications/com.redshield.architect.desktop
appstreamcli validate --pedantic %{buildroot}%{_metainfodir}/com.redshield.architect.metainfo.xml

%files
%license LICENSE
%doc README.md
%{_bindir}/redshield-architect-workbench
%{_datadir}/applications/com.redshield.architect.desktop
%{_metainfodir}/com.redshield.architect.metainfo.xml
%{_datadir}/icons/hicolor/32x32/apps/redshield-architect-workbench.png
%{_datadir}/icons/hicolor/128x128/apps/redshield-architect-workbench.png
%{_datadir}/icons/hicolor/256x256/apps/redshield-architect-workbench.png

%changelog
* Wed Jul 22 2026 RedShield Architect contributors <noreply@redshield.dev> - 0.1.0-0.1
- Add initial Fedora-oriented development RPM spec.
