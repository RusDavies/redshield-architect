# ArchiMate Alignment Decision

RedShield should stay metamodel-neutral internally and treat ArchiMate as a mapping/export target, not as the canonical model language.

This decision is not anti-standard. It is anti-accidental-coupling. Standards are useful when they create interoperability; they are expensive when they become the product's spine before the product has proven its own workflow.

## Current Standard Context

The Open Group describes ArchiMate as an open and independent enterprise architecture modeling language for describing, analyzing, and visualizing relationships across business domains. The Open Group licensed-downloads page currently lists ArchiMate Specification version 4 as the latest version, released April 2026.

Sources:

- The Open Group ArchiMate overview: <https://www.opengroup.org/archimate-forum/archimate-overview>
- The Open Group ArchiMate licensed downloads: <https://www.opengroup.org/archimate-licensed-downloads>

## Recommendation

Use a RedShield-native portfolio model as the internal source of truth. Add ArchiMate alignment as an explicit compatibility layer later:

- import/export mapping
- optional profile/notation hints
- validation warnings when an object cannot map cleanly
- documentation showing approximate mapping, lossiness, and non-goals

Do not make ArchiMate element types, relationship taxonomy, layers, notation, or exchange format the canonical package model.

## Why Not Native ArchiMate Internally

RedShield's core workflow is not portfolio-first EA modeling. It is requirements, semantic software/solution architecture, diagrams, traceability, proposal review, validation, and Git-friendly package change.

Using ArchiMate as the internal model would pull the product toward:

- a broad enterprise modeling vocabulary before the MVP needs it
- diagram/notation semantics that are not central to RedShield's first workflow
- standard-version coupling, now made more obvious by the ArchiMate 4 release
- user confusion between UML solution models and EA notation
- a larger import/export conformance burden before the package contract is stable

That is exactly the kind of "small compatibility decision" that quietly turns into a second product hiding inside the first one.

## What RedShield Should Keep Native

The native model should keep concepts that support RedShield jobs-to-be-done:

- requirements and acceptance criteria
- model elements and relationships
- diagram views over model truth
- trace links
- proposal operations
- portfolio facts with provenance and review state
- lifecycle, ownership, technology, risk, control, and governance facts
- source references and imported/discovered fact provenance

These map partially to ArchiMate concepts, but RedShield should preserve its own semantics first: typed operations, proposal review, package validation, Git diffs, and links from solution architecture to portfolio facts.

## Mapping Strategy

Treat ArchiMate mapping as a separate adapter matrix:

| RedShield concept | ArchiMate alignment posture |
| --- | --- |
| `business_capability` | Usually maps to an ArchiMate capability-like strategy concept. |
| `portfolio_application` | Usually maps to an application-layer element, depending on product/application distinction. |
| `portfolio_service` | May map to application/business/technology service depending on provider and consumer context. |
| `technology_component` | May map to technology-layer node/device/system-software/artifact-style concepts. |
| `technology_standard` | Better treated as RedShield governance metadata, not a direct ArchiMate element. |
| `owner` / `organization_unit` | May map to business actor/role or remain governance metadata. |
| `lifecycle_milestone` | Better treated as RedShield lifecycle metadata or roadmap overlay. |
| `roadmap_item` | May map to implementation/migration concepts, but needs explicit scope. |
| `risk` / `control` | Better treated as governance/risk metadata unless a profile is defined. |
| `governance_decision` | RedShield decision/audit metadata, not a native ArchiMate core object. |
| `data_source` | RedShield provenance metadata. |

The adapter should be honest about lossy mappings. If an ArchiMate export would flatten review state, source provenance, proposal lineage, or Git application metadata, the export should say so.

## Compatibility Boundary

RedShield can support ArchiMate without becoming ArchiMate-shaped:

- allow package metadata to record an optional `archimate` mapping profile later
- keep mapping tables versioned by ArchiMate specification version
- prefer export/import adapters over internal inheritance from ArchiMate types
- do not make ArchiMate notation required for RedShield diagrams
- do not block native RedShield objects just because they lack perfect ArchiMate equivalents

The correct posture is "interoperate where useful, remain native where RedShield's review and package model has stronger semantics."

## Follow-Up Work

- Build an explicit ArchiMate mapping matrix for the first portfolio object kinds.
- Decide whether to support ArchiMate exchange format import/export before or after PlantUML/Mermaid/Structurizr priorities.
- Add schema fields for optional mapping hints only after a real adapter needs them.
- Add tests for lossy mapping warnings when export/import exists.
