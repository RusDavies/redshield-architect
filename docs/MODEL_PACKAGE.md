# Model Package

The first RedShield model package is a directory of deterministic JSON files under `redshield/`.

```text
redshield/
  manifest.json
  requirements/requirements.json
  model/portfolio.json
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

Inspect portfolio facts without opening the workbench:

```sh
cargo run -- portfolio-summary examples/minimal/redshield
cargo run -- portfolio-summary examples/minimal/redshield tauri
```

Render the first use-case diagram:

```sh
cargo run -- render-use-case examples/minimal/redshield target/redshield/first-use-case.svg
```

Render the first read-only portfolio lifecycle roadmap:

```sh
cargo run -- render-lifecycle-roadmap examples/minimal/redshield target/redshield/portfolio-lifecycle-roadmap.svg
```

The renderer converts semantic model element and relationship IDs into Graphviz DOT, then renders SVG through `dot -Tsvg`. DOT and SVG are generated artifacts; the canonical source remains the JSON model package.

The `web/` spike loads the same example model into an interactive React Flow canvas and uses ELK for auto-layout. It is the first GUI interaction candidate for direct manipulation: move, align, distribute, connect, inspect, and persist view metadata.

The workbench emits proposal-shaped operation drafts for direct manipulation actions. Dragging nodes emits `move_diagram_node`, align/distribute controls emit their matching layout operations, ELK emits `apply_diagram_auto_layout`, creating a connector emits both a draft `create_relationship` and `connect_diagram_relationship`, and the semantic inspector emits `update_model_element_details` for reviewed element detail edits.

The current spike can save/load the draft transaction in browser local storage, mark it accepted, and download proposal JSON that the CLI can apply. Direct filesystem writes from the workbench remain a later Tauri/backend adapter concern.

## Portfolio Objects

`model/portfolio.json` is the first RedShield-native enterprise architecture slice. It keeps portfolio facts close to requirements, UML elements, trace links, diagrams, and proposal review instead of copying a portfolio-tool data model.

The initial object kinds are deliberately bounded:

- `business_capability`
- `portfolio_application`
- `portfolio_service`
- `technology_component`
- `technology_standard`
- `organization_unit`
- `owner`
- `lifecycle_milestone`
- `roadmap_item`
- `risk`
- `control`
- `governance_decision`
- `data_source`

Each portfolio object has a stable `id`, `kind`, `name`, optional description, status, lifecycle state, structured lifecycle details, criticality, standard state, tags, source references, external references, and typed reference lists for owners, capabilities, technologies, risks, and related model elements. `relatedElementRefs` are validated against `model/elements.json`, so portfolio facts can point at the same UML-backed components and use cases already used by diagrams and traceability.

Structured lifecycle details are documented in [Portfolio Lifecycle Semantics](PORTFOLIO_LIFECYCLE.md). They support current state, local phase, target state/date, support/retirement dates, and milestone references for portfolio applications, products represented as applications, portfolio services, technology components, and technology standards.

This is the product wedge: business capabilities, services, technologies, risks, and ownership can be reviewed as deterministic package data while still linking directly to solution-architecture model elements. It is not a broad EA repository yet, and the MVP UI does not need to edit every object kind on day one.

The schema/UI split for these concepts is defined in [Enterprise Architecture Schema/UI Boundary](EA_SCHEMA_UI_BOUNDARY.md).

Portfolio views are named package views over those facts, documented in [Portfolio View Semantics](PORTFOLIO_VIEWS.md). The first view kinds are `capability_map`, `application_landscape`, `lifecycle_roadmap`, `risk_heatmap`, and `dependency_map`. They may reference portfolio objects through `portfolioRefs` and model elements through `modelRefs`.

Portfolio changes use typed proposal operations:

- `create_portfolio_object`
- `update_portfolio_object`

The schema rejects unknown object kinds, unsupported lifecycle/criticality/standard states, stray operation args, and no-op portfolio updates. Accepted proposal application writes `model/portfolio.json` through the same validation path as model elements, relationships, diagrams, trace links, and render profiles.

The CLI `portfolio-summary` command and the workbench sidebar provide read-only portfolio summaries: object totals, kind/lifecycle/criticality counts, standard-state hints, related model-link counts, and a compact object list. The CLI accepts an optional search string. The workbench sidebar supports text search plus kind and lifecycle filters. These filters are temporary local narrowing controls: they do not edit portfolio data, create package state, or define shareable views.

The CLI `render-lifecycle-roadmap` command renders the first portfolio view kind as generated SVG. It colors referenced portfolio objects by lifecycle state, derives a visible timeline scale from lifecycle dates, uses portfolio-kind swimlanes, renders target-state callouts, renders lifecycle milestones distinctly, and draws milestone links from structured lifecycle metadata when both objects are in the view.

## Model Elements

Model elements are semantic objects under `model/elements.json`. The common element envelope now supports:

- `id`, `kind`, and `name`
- `aliases` for alternate names or role labels
- `description` for a short summary
- `documentation` for longer notes owned by the model element
- `status`: `draft`, `proposed`, `accepted`, `deprecated`, or `retired`
- `stereotypes` and `tags` for classification and render/profile selectors
- `provenance` with source references, creator, creation timestamp, and notes
- `externalReferences` for links to documents, standards, tickets, repositories, or imported/source artifacts
- `architecture` metadata for owner, lifecycle, criticality, technology, risk, capability, and service mappings
- `classifier` details for UML class/component elements
- `actorDetails`, `useCaseDetails`, `activityDetails`, and `sequenceParticipantDetails` for supported UML behavior/interaction elements

Architecture metadata is optional on any model element. It is intended for solution/enterprise architecture facts that attach to the element without changing its UML kind or diagram appearance:

- `owners`: references to accountable, responsible, technical, business, or support owners, with optional display names
- `lifecycle`: state, phase, milestone references, target date, and notes
- `criticality`: `low`, `medium`, `high`, or `critical`
- `technologies`: technology references with role, version, and standard state
- `risks`: risk references with severity, status, and notes
- `capabilities`: capability references with fit and maturity
- `services`: service mappings for provided, consumed, exposed, dependent, or supporting services

The first schema keeps these as bounded mappings rather than a full enterprise architecture repository. Follow-up object types can later make owners, capabilities, services, risks, milestones, and technology standards canonical objects of their own.

Classifier details are optional and currently valid only on `class` and `component` model elements. They support:

- element-level `isAbstract` and `isStatic` flags
- `attributes` with name, visibility, `typeRef`, multiplicity, default value, static/read-only flags, and documentation
- `operations` with name, visibility, `returnTypeRef`, parameters, abstract/static flags, and documentation
- operation parameters with name, `typeRef`, direction, multiplicity, and default value
- multiplicity bounds using integer `lower`, integer `upper`, or unbounded `upper: "*"`

This keeps semantic UML classifier detail in `model/elements.json`, while view layout, renderer selection, label placement, and custom visual treatment remain view/render metadata.

The other supported UML element detail envelopes are also kind-scoped:

- `actorDetails` belongs to `actor` elements and records actor type, responsibilities, goals, and constraints.
- `useCaseDetails` belongs to `use_case` elements and records actor references, preconditions, postconditions, main flow steps, alternate flows, and extension points.
- `activityDetails` belongs to `activity` elements and records parameters, activity nodes, and flows between activity nodes.
- `sequenceParticipantDetails` belongs to `sequence_participant` elements and records participant kind, represented model reference, lifeline name, and whether the lifeline is external.

## Render Profiles

Render profiles define how matching model elements are drawn without changing semantic model truth. They live under `views/render-profile.json` in the prototype package and are validated by `schemas/render-profile.schema.json`.

A profile contains ordered rules plus a fallback renderer. Rules match elements by `elementId`, `elementKind`, `stereotype`, or `tag`; higher `precedence` wins when multiple enabled rules match. A render target chooses a renderer ID such as `uml.actor`, `uml.class`, `uml.component`, `image.element`, or `html.custom`, and may supply style, label placement, connector ports, and asset references.

Image-backed renderers use explicit assets with source and license provenance. The schema allows package-relative references such as `assets/render/duck.png`, but the example deliberately records only the reference and provenance placeholder rather than committing a binary asset. The detailed asset rules live in [Asset References](ASSET_REFERENCES.md).

The current workbench spike resolves the first profile, applies rule precedence, and renders built-in actor, class, component, and image-backed element nodes. Referenced or missing image assets render as a visible asset-status placeholder until a real package asset is available.

The workbench sidebar also includes a draft render-rule editor. It can assign renderer/style rules by element ID, element kind, stereotype, or tag; toggle existing rules; reset to the packaged default profile; and download the current in-memory render profile JSON. Semantic element edits remain in the inspector and emit semantic proposal operations rather than render-profile operations.

Render-rule edits now emit typed proposal operations:

- `upsert_render_rule`
- `remove_render_rule`

Accepted render-profile operations are applied by the same `apply-proposal` command as model/view operations. They update `views/render-profile.json`, revalidate the model package, and store the applied proposal copy under `redshield/proposals/applied/`. This gives the future Tauri/backend adapter a durable operation path instead of letting the browser mutate package files directly.

Export behavior for built-in, image-backed, SVG, and custom HTML renderers is defined in [Render Export Behavior](RENDER_EXPORT_BEHAVIOR.md).

Diagram views may now include canonical view metadata under `layout`. This metadata records canvas coordinates separately from semantic model truth. UML views use `modelRefs`; portfolio views may also use `portfolioRefs` to include portfolio objects:

- `coordinateSystem`: currently `canvas`
- `layoutEngine`: optional generator such as `elk.layered`
- `layoutState`: `generated`, `manual`, or `mixed`
- `nodes`: model element references with persisted bounds, per-node layout state, and optional label positions
- `connectors`: relationship references with per-connector layout state, optional route hints, and optional label positions
- `style`: optional node or connector presentation metadata such as fill, stroke, text color, and line style

The Rust validator checks that layout nodes reference elements in the diagram, connector layout references point to real relationships, bounds are positive, and route/layout states are supported. Canvas edits should still become typed operations before they mutate these files durably.

## View/Layout Operations

Accepted proposal transactions can now apply typed semantic element, view/layout, and render-profile operations:

- `create_portfolio_object`
- `update_portfolio_object`
- `update_model_element_details`
- `move_diagram_node`
- `resize_diagram_node`
- `align_diagram_nodes`
- `distribute_diagram_nodes`
- `connect_diagram_relationship`
- `route_diagram_connector`
- `style_diagram_object`
- `apply_diagram_auto_layout`
- `upsert_render_rule`
- `remove_render_rule`

`update_model_element_details` updates existing semantic model elements in `model/elements.json`. It keeps `id` and `kind` stable, can update common semantic fields such as name, documentation, status, tags, provenance, and external references, and can replace or clear supported detail envelopes such as `architecture`, `classifier`, `actorDetails`, `useCaseDetails`, `activityDetails`, and `sequenceParticipantDetails`. Package validation still enforces kind boundaries, supported states, bounded architecture metadata, and malformed UML details before the accepted proposal is written.

View/layout operations update only `views/diagrams.json`. Render-profile operations update only `views/render-profile.json`. They do not create requirements, model elements, trace links, or semantic relationships. Semantic relationship creation still uses `create_relationship`; `connect_diagram_relationship` only makes an existing relationship visible/configured in a diagram view.

The proposal JSON Schema validates operation arguments per operation type. For example, `move_diagram_node` requires `diagramId`, `modelRef`, `x`, and `y`; `align_diagram_nodes` requires at least two model refs and a supported alignment; `upsert_render_rule` requires `profileId` plus a render rule matching the render profile schema; and unknown or stray args are rejected before the Rust application layer runs.

Apply an accepted proposal transaction:

```sh
cargo run -- apply-proposal path/to/redshield path/to/proposal.json
```

Application requires proposal state `accepted`. The core applies typed create operations to canonical files, revalidates the package, writes deterministic JSON, and stores an applied proposal copy under `redshield/proposals/applied/`.
