# Fedora RPM Packaging

This directory contains the first Fedora-oriented development RPM spec for the
RedShield Architect Tauri workbench.

It is intentionally a local build workflow, not a Fedora review submission yet.
Before it becomes review-ready, the Rust crate dependencies and npm frontend
dependencies need an offline/vendor strategy that satisfies Fedora packaging
policy.

Build locally from the repository root:

```sh
./scripts/build_fedora_rpm.sh
```

The script creates a source tarball from the current working tree, excludes
generated build outputs, and runs `rpmbuild -ba` with a temporary RPM topdir.
Built RPMs are copied to `target/rpm/`.

Prepare review-candidate source evidence from a clean tagged tree:

```sh
./scripts/prepare_fedora_rpm_review_candidate.sh --tag v0.1.0
```

The release-prep helper writes a tagged source archive, Cargo vendor archive,
npm dependency-cache archive, dependency inventories, license summaries, and
SHA-256 checksums under `target/fedora-rpm-review/<tag>/`. These are declared
source inputs for a future offline `rpmbuild` or `mock` gate; generating them
does not by itself make the package Fedora-review-ready.

Expected local tools on Fedora:

- `rpmbuild`
- `cargo` and `rust`
- `nodejs` and `nodejs-npm`
- `python3`
- `pkgconf-pkg-config`
- `webkit2gtk4.1-devel`
- `gtk3-devel`
- `libsoup3-devel`
- `javascriptcoregtk4.1-devel`
- `openssl-devel`
- `desktop-file-utils`
- `appstream`
