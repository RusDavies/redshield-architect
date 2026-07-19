# Model Package

The first RedShield model package is a directory of deterministic JSON files under `redshield/`.

```text
redshield/
  manifest.json
  requirements/requirements.json
  model/elements.json
  model/relationships.json
  views/diagrams.json
  trace/links.json
  proposals/open/*.json
```

The JSON Schemas in `schemas/` describe the prototype file shapes. The Rust core currently validates the same first slice: schema version, duplicate IDs, supported element and relationship kinds, broken references, use-case diagram references, trace links, and proposal operation envelopes.

## Thin CLI

Validate the example model:

```sh
cargo run -- validate examples/minimal/redshield
```

Render the first use-case diagram:

```sh
cargo run -- render-use-case examples/minimal/redshield target/redshield/first-use-case.svg
```

The rendered SVG is generated from semantic model element and relationship IDs. It is not canonical project state.

Apply an accepted proposal transaction:

```sh
cargo run -- apply-proposal path/to/redshield path/to/proposal.json
```

Application requires proposal state `accepted`. The core applies typed create operations to canonical files, revalidates the package, writes deterministic JSON, and stores an applied proposal copy under `redshield/proposals/applied/`.
