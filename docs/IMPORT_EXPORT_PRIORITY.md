# Import And Export Priority

RedShield import/export work should serve the native package workflow first. External formats are compatibility surfaces, not alternate sources of truth.

The first priority decision is to separate three different jobs:

1. durable RedShield package state
2. readable/generated delivery artifacts
3. semantic interoperability with other modeling tools

## Priority Order

| Priority | Format | Direction | Why |
| --- | --- | --- | --- |
| 0 | RedShield JSON package | import and export | Canonical model state. Already the source of truth for validation, Git diffs, proposal application, and examples. |
| 1 | SVG | export | First useful diagram artifact. It is deterministic, reviewable, embeddable in docs, and already aligned with the renderer/export-scene contract. |
| 2 | PDF | export | Delivery format for human review. It should be generated from the same validated export scene as SVG, not implemented as a separate semantic path. |
| 3 | PlantUML | export first, limited import later | Good documentation and code-review target for UML-ish diagrams. Cheap to inspect, easy to diff, and useful before full model interchange exists. |
| 4 | Mermaid | export first, limited import later | Useful for Markdown-native docs and lightweight architecture diagrams, but less precise for RedShield's richer model semantics. |
| 5 | Structurizr DSL | export first, import later | Strong text-based architecture-as-code fit, especially for C4-style views, but needs a deliberate mapping from RedShield model/portfolio/view concepts. |
| 6 | ArchiMate Model Exchange File Format | export preview before import | Valuable for EA interoperability once the portfolio mapping matrix has implementation evidence. Import must be preview/review-first because ArchiMate semantics can flatten RedShield provenance, lifecycle, and proposal history. |
| 7 | XMI | defer | Broad and tool-fragmented enough that supporting it too early would distort the prototype around UML interchange instead of RedShield's package/review workflow. |

## Format Posture

### RedShield JSON Package

The `redshield/` package remains the canonical model. Adapters may read or write package files only through validation and typed proposal operations.

YAML can be considered later as a human-authored convenience format, but it should not become a second canonical package representation unless a real workflow justifies the extra round-trip burden.

### SVG And PDF

SVG is the first durable diagram export target. PDF follows SVG as a presentation artifact.

Neither format should be imported as semantic model truth. An SVG or PDF importer may eventually support trace/provenance extraction, but it should not try to reconstruct canonical UML, portfolio, or proposal state from rendered graphics. That way lies sadness with coordinates.

### PlantUML And Mermaid

PlantUML and Mermaid should be early text export targets because they are easy to review in pull requests and useful in developer documentation.

Initial support should be export-only:

- use-case, component, class, activity, and sequence views where RedShield already has enough semantic data
- deterministic comments or metadata for RedShield IDs where the target syntax can carry them
- warnings when stereotypes, render profiles, lifecycle fields, provenance, or proposal state cannot be represented

Import should wait until export examples reveal the subset worth parsing. Early import should produce proposal previews, not direct canonical edits.

### Structurizr DSL

Structurizr DSL should come after PlantUML/Mermaid exports but before XMI. It is text-first and useful for architecture-as-code workflows, but it needs a stronger mapping decision:

- which RedShield model elements correspond to software systems, containers, components, people, deployment nodes, and views
- how portfolio applications/services/capabilities relate to Structurizr workspace boundaries
- whether imported Structurizr identifiers become local IDs, `source:` refs, or proposal-created objects

Structurizr import should be preview/review-first and should preserve source identity.

### ArchiMate Exchange

ArchiMate exchange belongs after the mapping matrix and detailed import/export matrix have enough evidence to keep the adapter honest.

Recommended order:

1. export a small portfolio-only ArchiMate exchange preview from RedShield package data
2. emit explicit lossiness warnings using [ArchiMate Mapping Matrix](ARCHIMATE_MAPPING_MATRIX.md)
3. import ArchiMate exchange files into reviewable proposals, preserving source IDs as `source:` refs
4. add bounded mapping hints only if adapter evidence proves tags, source refs, and policy are insufficient

ArchiMate exchange should not become the internal package shape.

### XMI

XMI should be deferred until the supported UML subset, proposal operations, and basic text exports are stable. It is useful for tool interchange, but it is expensive to do credibly and easy to overfit to one vendor's dialect.

When it arrives, start with export of the RedShield-supported UML subset, then importer previews. Do not treat XMI as the first migration path for users.

## Implementation Sequence

1. Keep native JSON package validation and proposal application as the mandatory gate.
2. Finish the shared export-scene boundary for SVG, with PDF downstream.
3. Add deterministic PlantUML and Mermaid exports for the supported UML/view subset.
4. Apply the detailed format coverage and warning posture in [Import And Export Matrix](IMPORT_EXPORT_MATRIX.md).
5. Add Structurizr DSL export once view mapping is clear.
6. Add an ArchiMate exchange export preview for portfolio objects using the mapping matrix.
7. Add importer previews only after exported examples and lossiness warnings are practical.
8. Defer XMI until RedShield's own UML subset and proposal workflow are stable enough to resist being bent into one vendor-shaped dialect.

## Non-Goals For The Prototype

- round-trip equivalence with every external modeling tool
- direct semantic import from SVG or PDF
- making ArchiMate or XMI the canonical package format
- hidden importer mutations without proposal review
- supporting vendor extensions before baseline open formats work
