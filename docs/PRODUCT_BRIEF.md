# Product Brief

## Product

RedShield Architect is a Linux-first requirements and architecture modeling workbench for technical teams that want text-backed models, useful UML views, traceability, and AI-assisted review without turning the model into opaque generated sludge.

## Audience

Initial users:

- software architects and senior engineers designing new systems
- small technical teams that already use Git for project artifacts
- consultants who need reviewable requirements, model, and decision evidence
- engineers who want AI assistance without losing provenance or approval control

Later users may include enterprise architecture, regulated engineering, security review, and modernization teams.

## Problem

Architecture tools often split into two unsatisfying camps:

- diagram-first tools that are pleasant for drawing but weak as durable semantic models
- heavyweight modeling suites that are powerful but cumbersome, proprietary, and often awkward on Linux

AI adds another problem: generated suggestions can be useful, but they are risky when they mutate canonical project artifacts without typed operations, validation, provenance, or human review.

## Thesis

RedShield Architect should treat architecture work as a model lifecycle:

- requirements are structured objects
- UML elements are structured objects
- diagrams are views over those objects
- traceability is first-class
- AI output is a proposal, not project truth
- Git history can carry model evolution, review, and accountability

## Prototype Scope

The first prototype should support one end-to-end path:

1. Create or import a small requirements set.
2. Propose model elements from approved source notes.
3. Review and accept or reject proposed operations.
4. Render at least one diagram view from accepted model data.
5. Show traceability from requirements to model elements and implementation tasks.
6. Export deterministic text-backed project files.

## Differentiators

- Linux-first desktop workflow
- deterministic model package suitable for Git
- practical UML rather than ceremony for its own sake
- AI proposal review as a core feature
- provenance and validation attached to model changes
- export-oriented design rather than storage lock-in

## Edition Strategy

RedShield Architect is planned as an MIT-licensed open-core product. The open core should provide real local architecture value: requirements, practical UML, traceability, deterministic model storage, validation, imports/exports, and local AI proposal review.

Paid/commercial scope is reserved for hosted collaboration, enterprise governance, organization-wide integrations, managed policy, compliance evidence, reporting at scale, and commercial support.

## Risks

- scope creep toward a full ALM suite
- weak diagram ergonomics despite strong model semantics
- overfitting to one AI provider or agent runtime
- insufficient import/export compatibility
- unclear licensing or packaging choices
