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

Validate a generated review-candidate evidence directory with an offline RPM
build that consumes only those declared inputs:

```sh
./scripts/validate_fedora_rpm_review_candidate.sh \
  --evidence-dir target/fedora-rpm-review/v0.1.0
```

The validation helper checks `SHA256SUMS`, copies only the tagged source
archive, Cargo vendor archive, and npm cache archive into a fresh RPM topdir,
then rebuilds the RPM with `--with offline_sources`. Cargo is forced offline and
npm uses the generated cache with `npm ci --offline`. To run the same source RPM
through `mock` when available:

```sh
./scripts/validate_fedora_rpm_review_candidate.sh \
  --evidence-dir target/fedora-rpm-review/v0.1.0 \
  --builder mock \
  --mock-config fedora-rawhide-x86_64
```

The `mock` path is the better clean-build-root evidence. Plain `rpmbuild` is a
useful local gate, but it is still host-shaped.

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
