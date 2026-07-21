# RedShield Architect

RedShield Architect is an early-stage, Linux-first workbench for requirements, architecture modeling, UML views, traceability, and AI-assisted design review.

The core idea is simple: requirements and model elements should be semantic, versioned objects, while diagrams are views over that model. AI tools should propose reviewable model changes rather than silently editing diagrams, documents, or project files.

## Direction

RedShield Architect is not intended to be a broad enterprise ALM clone. The first useful slice is a local-first architecture workbench that can:

- capture requirements and architecture notes
- model practical UML concepts such as use cases, classes, components, activities, and sequences
- maintain traceability between requirements, model elements, tests, decisions, and implementation tasks
- store canonical model data in deterministic, Git-friendly text files
- render diagrams from model data
- let AI providers propose typed, reviewable model operations
- preserve provenance, validation results, and human approval decisions

## Early Non-Goals

- full feature parity with established enterprise modeling suites
- Windows-first product design
- networked collaboration before the local model is credible
- opaque AI mutation of canonical model files
- proprietary-format lock-in as the primary storage model

## Repository Map

- `src/` - first Rust model/validation/rendering core and CLI
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
- `docs/RENDER_EXPORT_BEHAVIOR.md` - export contract for built-in, image-backed, SVG, and custom HTML renderers

## Status

This repository now contains the first thin prototype core: a Rust CLI that loads a text-backed `redshield/` model package, validates canonical objects, validates diagram, saved portfolio view, and roadmap presentation metadata, applies accepted proposal transactions including typed view/layout, saved portfolio view, and roadmap presentation operations, and renders one semantic use-case diagram to SVG. The example package also defines an initial render profile schema for matching model elements to built-in or image-backed renderers. The `web/` spike reads the same example package, uses persisted view metadata as the starting point for direct manipulation, resolves render profile rules for actor/class/component/image-backed nodes, provides read-only saved portfolio and roadmap presentation summaries, and can save, accept, and download proposal-shaped operation drafts from canvas actions.

## License

RedShield Architect is licensed under the MIT License. See `LICENSE`.
