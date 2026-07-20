# Model Package

The first RedShield model package is a directory of deterministic JSON files under `redshield/`.

```text
redshield/
  manifest.json
  requirements/requirements.json
  model/elements.json
  model/relationships.json
  views/diagrams.json
  views/render-profile.json
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

The workbench emits proposal-shaped operation drafts for direct manipulation actions. Dragging nodes emits `move_diagram_node`, align/distribute controls emit their matching layout operations, ELK emits `apply_diagram_auto_layout`, and creating a connector emits both a draft `create_relationship` and `connect_diagram_relationship`.

The current spike can save/load the draft transaction in browser local storage, mark it accepted, and download proposal JSON that the CLI can apply. Direct filesystem writes from the workbench remain a later Tauri/backend adapter concern.

## Render Profiles

Render profiles define how matching model elements are drawn without changing semantic model truth. They live under `views/render-profile.json` in the prototype package and are validated by `schemas/render-profile.schema.json`.

A profile contains ordered rules plus a fallback renderer. Rules match elements by `elementId`, `elementKind`, `stereotype`, or `tag`; higher `precedence` wins when multiple enabled rules match. A render target chooses a renderer ID such as `uml.actor`, `uml.class`, `uml.component`, `image.element`, or `html.custom`, and may supply style, label placement, connector ports, and asset references.

Image-backed renderers use explicit assets with source and license provenance. The schema allows references such as `assets/render/duck.png`, but the example deliberately records only the reference and provenance placeholder rather than committing a binary asset.

Diagram views may now include canonical view metadata under `layout`. This metadata records canvas coordinates separately from semantic model truth:

- `coordinateSystem`: currently `canvas`
- `layoutEngine`: optional generator such as `elk.layered`
- `layoutState`: `generated`, `manual`, or `mixed`
- `nodes`: model element references with persisted bounds, per-node layout state, and optional label positions
- `connectors`: relationship references with per-connector layout state, optional route hints, and optional label positions
- `style`: optional node or connector presentation metadata such as fill, stroke, text color, and line style

The Rust validator checks that layout nodes reference elements in the diagram, connector layout references point to real relationships, bounds are positive, and route/layout states are supported. Canvas edits should still become typed operations before they mutate these files durably.

## View/Layout Operations

Accepted proposal transactions can now apply typed view/layout operations to canonical diagram metadata:

- `move_diagram_node`
- `resize_diagram_node`
- `align_diagram_nodes`
- `distribute_diagram_nodes`
- `connect_diagram_relationship`
- `route_diagram_connector`
- `style_diagram_object`
- `apply_diagram_auto_layout`

These operations update only `views/diagrams.json`. They do not create requirements, model elements, trace links, or semantic relationships. Semantic relationship creation still uses `create_relationship`; `connect_diagram_relationship` only makes an existing relationship visible/configured in a diagram view.

The proposal JSON Schema validates operation arguments per operation type. For example, `move_diagram_node` requires `diagramId`, `modelRef`, `x`, and `y`; `align_diagram_nodes` requires at least two model refs and a supported alignment; and unknown or stray args are rejected before the Rust application layer runs.

Apply an accepted proposal transaction:

```sh
cargo run -- apply-proposal path/to/redshield path/to/proposal.json
```

Application requires proposal state `accepted`. The core applies typed create operations to canonical files, revalidates the package, writes deterministic JSON, and stores an applied proposal copy under `redshield/proposals/applied/`.
