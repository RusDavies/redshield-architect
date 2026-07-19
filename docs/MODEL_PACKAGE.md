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

The JSON Schemas in `schemas/` describe the prototype file shapes. `cargo test` validates the example package against those published schemas and meta-validates the schema documents themselves. The Rust core validates the cross-file semantic slice that JSON Schema cannot see alone: schema version, duplicate IDs, supported element and relationship kinds, broken references, use-case diagram references, trace links, and proposal operation envelopes.

## Thin CLI

Validate the example model:

```sh
cargo run -- validate examples/minimal/redshield
```

Render the first use-case diagram:

```sh
cargo run -- render-use-case examples/minimal/redshield target/redshield/first-use-case.svg
```

The renderer converts semantic model element and relationship IDs into Graphviz DOT, then renders SVG through `dot -Tsvg`. DOT and SVG are generated artifacts; the canonical source remains the JSON model package.

The `web/` spike loads the same example model into an interactive React Flow canvas and uses ELK for auto-layout. It is the first GUI interaction candidate for direct manipulation: move, align, distribute, connect, inspect, and persist view metadata.

Diagram views may now include canonical view metadata under `layout`. This metadata records canvas coordinates separately from semantic model truth:

- `coordinateSystem`: currently `canvas`
- `layoutEngine`: optional generator such as `elk.layered`
- `layoutState`: `generated`, `manual`, or `mixed`
- `nodes`: model element references with persisted bounds, per-node layout state, and optional label positions
- `connectors`: relationship references with per-connector layout state, optional route hints, and optional label positions

The Rust validator checks that layout nodes reference elements in the diagram, connector layout references point to real relationships, bounds are positive, and route/layout states are supported. Canvas edits should still become typed operations before they mutate these files durably.

Apply an accepted proposal transaction:

```sh
cargo run -- apply-proposal path/to/redshield path/to/proposal.json
```

Application requires proposal state `accepted`. The core applies typed create operations to canonical files, revalidates the package, writes deterministic JSON, and stores an applied proposal copy under `redshield/proposals/applied/`.
