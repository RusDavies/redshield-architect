Name:           redshield-architect-workbench
Version:        0.1.0
Release:        0.1%{?dist}
Summary:        Requirements and architecture modeling workbench

License:        MIT
URL:            https://github.com/RusDavies/redshield-architect
Source0:        redshield-architect-%{version}.tar.gz

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
packaging still needs vendored/offline Rust and npm dependency handling.

%prep
%autosetup -n redshield-architect-%{version}

%build
export PKG_CONFIG=/usr/bin/pkg-config
export CARGO_HOME="${RPM_CARGO_HOME:-%{_topdir}/../rpm-cache/cargo-home}"
export npm_config_cache="${RPM_NPM_CACHE:-%{_topdir}/../rpm-cache/npm-cache}"

npm ci --prefix web
npm run --prefix web build
cargo build --manifest-path web/src-tauri/Cargo.toml --release --locked

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
cargo test --locked
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
