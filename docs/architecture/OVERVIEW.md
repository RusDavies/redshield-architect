# Architecture Overview

## Shape

RedShield Architect is expected to start as a local-first desktop workbench with a typed model core and a web-based UI shell.

The intended stack is:

- Tauri 2 desktop shell
- Rust core for model storage, validation, proposal application, imports, exports, and CLI reuse
- TypeScript UI for the interactive workbench and diagram views
- deterministic JSON model package as the canonical project format

## Core Boundary

The core model engine owns canonical state. UI features, CLI commands, importers, exporters, and AI adapters should all express changes through typed operations.

Examples:

- `create_requirement`
- `update_requirement`
- `create_model_element`
- `create_trace_link`
- `create_diagram_view`
- `apply_proposal_operation`

The same operation path should run validation and produce consistent diffs regardless of whether the change came from a human UI action, CLI command, importer, or AI proposal.

## Model Package

The canonical model package should be a directory of normalized JSON files, provisionally under `redshield/`.

Markdown may be generated for summaries and review packets, but Markdown should not become a competing source of truth for requirements, UML elements, trace links, diagrams, or proposals.

## Diagram Views

Diagrams are views over semantic model objects. A diagram may store layout and presentation hints, but it should not be the only place where architectural facts live.

The first diagram target is a use-case view because it is easy to connect to actors, scenarios, requirements, and traceability.

## AI Proposal Boundary

AI providers should submit proposal packages containing:

- typed operations
- rationale
- source references
- validation results
- provider/provenance metadata
- review state

The product should treat these proposals like reviewable patches against the model. Providers should not directly mutate canonical model files.

## Import And Export

Import/export should be practical and incremental. Possible targets include:

- XMI
- PlantUML
- Mermaid
- Structurizr DSL
- SVG
- PDF
- JSON/YAML summaries

These formats are integration surfaces, not the internal model authority.
