# RedShield Architect

RedShield Architect is an early-stage, Linux-strong and web-ready workbench for requirements, architecture modeling, UML views, traceability, and AI-assisted design review.

The core idea is simple: requirements and model elements should be semantic, versioned objects, while diagrams are views over that model. AI tools should propose reviewable model changes rather than silently editing diagrams, documents, or project files.

## Build Notes

The workbench can build a Linux AppImage from `web/`:

```sh
npm run build:appimage
```

For debug packaging:

```sh
npm run build:appimage:debug
```

The wrapper first uses Tauri's normal AppImage bundler. On newer Fedora-style hosts where Tauri's cached linuxdeploy AppImage uses an older embedded `strip` that cannot read `.relr.dyn` sections, it retries with extracted linuxdeploy and the system `strip`.

Linux desktop metadata lives in `web/src-tauri/metainfo/`. Tauri copies the
AppStream metainfo into AppImage and RPM bundles so desktop software centers
and distro-native package checks can identify the workbench consistently.

The first Fedora-oriented development RPM workflow lives in `packaging/fedora/`
and can be run from the repository root:

```sh
./scripts/build_fedora_rpm.sh
```

This produces local RPM artifacts under `target/rpm/`. It is a development
workflow, not a Fedora review submission; dependency vendoring/offline build
policy still needs to be tightened before public repository packaging.

Fedora RPM review-candidate evidence and offline validation helpers are also in
`scripts/`. Generate evidence from a clean tagged tree with
`prepare_fedora_rpm_review_candidate.sh`, then validate that candidate with
`validate_fedora_rpm_review_candidate.sh --evidence-dir <dir>`.

## Direction

RedShield Architect is not intended to be a broad enterprise ALM clone. The first useful slice is a local-first architecture workbench that keeps a deliberate path to browser-hosted, self-hosted, and SaaS surfaces. It can:

- capture requirements and architecture notes
- model practical UML concepts such as use cases, classes, components, activities, and sequences
- maintain traceability between requirements, model elements, tests, decisions, and implementation tasks
- store canonical model data in deterministic, Git-friendly text files
- render diagrams from model data
- let AI providers propose typed, reviewable model operations
- preserve provenance, validation results, and human approval decisions
- reuse the same TypeScript workbench and model-operation boundary across Linux desktop and future web deployments

## Early Non-Goals

- full feature parity with established enterprise modeling suites
- Windows-first product design
- networked collaboration before the local model is credible
- opaque AI mutation of canonical model files
- proprietary-format lock-in as the primary storage model

## Repository Map

- `src/` - first Rust model/validation/rendering core and CLI
- `scripts/` - project build helpers, including the Linux AppImage wrapper
- `packaging/fedora/` - first Fedora-oriented development RPM spec and notes
- `web/` - React Flow + ELK workbench interaction spike
- `schemas/` - prototype JSON Schemas for the `redshield/` model package and proposal transactions
- `examples/minimal/redshield/` - smallest model package used by tests and the CLI
- `docs/PRODUCT_BRIEF.md` - product framing and audience
- `docs/REQUIREMENTS.md` - first prototype requirements
- `docs/ROADMAP.md` - public milestone outline
- `docs/architecture/OVERVIEW.md` - architecture direction
- `docs/security/THREAT_MODEL.md` - public safety and privacy model
- `docs/research/COMPARABLE_TOOLS.md` - high-level market and tooling context
- `docs/MODEL_PACKAGE.md` - current text-backed model package shape and CLI usage
- `docs/PORTFOLIO_SAVED_VIEWS.md` - saved portfolio query contract
- `docs/PORTFOLIO_ROADMAP_PRESENTATION.md` - lifecycle-roadmap presentation contract
- `docs/PORTFOLIO_SUBTYPE_PROFILES.md` - decision to defer product/application/service subtype profiles until import/export evidence proves the need
- `docs/RENDER_EXPORT_BEHAVIOR.md` - export contract for built-in, image-backed, SVG, and custom HTML renderers
- `docs/AI_AGENT_INTERACTION_SURFACE.md` - provider-agnostic workbench contract for conversation, context, provenance, proposal review, and apply/reject controls
- `docs/ARCHIMATE_MAPPING_MATRIX.md` - explicit first mapping posture from RedShield portfolio object kinds and refs to ArchiMate adapter targets
- `docs/IMPORT_EXPORT_PRIORITY.md` - priority order for native packages, SVG/PDF, PlantUML, Mermaid, Structurizr DSL, ArchiMate exchange, and XMI

## Status

This repository now contains the first thin prototype core: a Rust CLI that loads a text-backed `redshield/` model package, validates canonical objects, validates diagram, saved portfolio view, and roadmap presentation metadata, applies accepted proposal transactions including typed view/layout, saved portfolio view, and roadmap presentation operations, and renders one semantic use-case diagram to SVG. The example package also defines an initial render profile schema for matching model elements to built-in or image-backed renderers. The `web/` spike reads the same example package, uses persisted view metadata as the starting point for direct manipulation, resolves render profile rules for actor/class/component/image-backed nodes, provides read-only saved portfolio and roadmap presentation summaries, and can save, accept, and download proposal-shaped operation drafts from canvas actions.

## License

RedShield Architect is licensed under the MIT License. See `LICENSE`.
