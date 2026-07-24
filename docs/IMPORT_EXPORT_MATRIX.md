# Import And Export Matrix

This matrix turns the import/export priority decision into implementation guidance. It describes what each external format should carry, what it must not pretend to carry, and how RedShield should warn when a round trip loses native package meaning.

RedShield's canonical format remains the validated `redshield/` JSON package. Every importer that creates or changes model state should emit proposal operations first; no external file should directly mutate canonical package files.

## Format Matrix

| Format | Direction | First useful scope | Good fit | Poor fit | Review posture |
| --- | --- | --- | --- | --- | --- |
| RedShield JSON | import and export | Full package validation, examples, proposal application, package imports. | Canonical model truth, deterministic diffs, typed operations, provenance, lifecycle, traceability. | External tool interoperability without an adapter. | Direct package load is allowed only through validation; changes still flow through proposal operations or trusted CLI commands. |
| RedShield YAML | export/import convenience only | Optional human-authored mirror for selected package files after JSON is stable. | Hand-edited seed files, examples, configuration-like authoring. | A second canonical package representation. YAML anchors, merge keys, implicit typing, and comments can make stable round trips messy. | Treat as a generated or convenience surface until a real authoring workflow proves it needs first-class support. |
| SVG | export only | Diagram and roadmap exports from a validated export scene. | Vector diagram delivery, documentation embedding, reviewable generated artifacts. | Semantic model import. Coordinates and shapes are not canonical architecture facts. | Export warnings may be non-blocking for preview output; release-quality export should block unsafe assets or unsupported renderers. |
| PDF | export only | Human-facing review/download generated from the SVG/export-scene path. | Stable presentation, sharing, print/archive use. | Semantic model import, editable source, proposal interchange. | Treat as a delivery artifact with metadata; never import as model truth. |
| PlantUML | export first, limited import later | Use-case, class, component, activity, and sequence views from supported UML model elements. | Text documentation, pull-request review, developer workflows, basic UML diagram exchange. | Full RedShield package semantics, portfolio lifecycle, render profiles, proposal history. | Export deterministic text with RedShield IDs in comments/metadata where possible. Import later as proposal previews from a bounded syntax subset. |
| Mermaid | export first, limited import later | Markdown-friendly flow, sequence, class, and lightweight architecture views. | Docs-as-code and browser-rendered lightweight diagrams. | Precise UML interchange, rich renderer/style rules, portfolio semantics. | Export readable diagrams with explicit lossiness warnings. Import only after a supported subset is documented. |
| Structurizr DSL | export first, import later | C4-style software architecture model and views once RedShield-to-C4 mapping is defined. | Text architecture-as-code, software systems, containers, components, relationships, views. | UML behavioral detail, arbitrary portfolio governance facts, renderer profiles. | Import must preserve source identity and create proposal previews; export should avoid inventing C4 hierarchy where RedShield has no evidence. |
| ArchiMate exchange | export preview before import | Portfolio objects and selected relationships using the ArchiMate mapping matrix. | Enterprise architecture interoperability and tool migration/export for EA facts. | Canonical RedShield package shape, proposal lineage, Git history, source provenance, detailed UML. | Start with portfolio export preview and explicit lossiness warnings. Import later as proposal previews with `source:` refs. |
| XMI | defer | RedShield-supported UML subset export after UML operations stabilize. | Interchange with UML/MOF-based tools when a bounded profile is selected. | Early prototype work, diagram layout round trips, vendor-neutral full-fidelity interchange. | Never lead with XMI import. Start with export fixtures, then importer previews for one declared UML/XMI profile. |

## Package Coverage Matrix

| RedShield package concept | JSON | YAML | SVG/PDF | PlantUML | Mermaid | Structurizr DSL | ArchiMate exchange | XMI |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Requirements | canonical | possible mirror | summary/labels only | notes/comments only | notes/comments only | properties or documentation only | properties only | out of initial scope |
| UML model elements | canonical | possible mirror | rendered shapes only | good subset for use case/class/component/activity/sequence | limited subset for class/sequence/flow/architecture | components/containers only when C4 mapping exists | only when mapped to EA elements or properties | eventual supported UML subset |
| Relationships | canonical | possible mirror | rendered connectors only | good subset for supported UML relationships | limited subset; relationship semantics often flatten | good C4-style relationship fit | mapped EA relationships with warnings | eventual supported UML subset |
| Diagram layout | canonical view metadata | possible mirror | primary rendered output | partial; text order and layout hints only | partial; renderer-owned layout | view definitions where supported | views where supported by exchange format | defer; vendor layout support is inconsistent |
| Render profiles/assets | canonical view/profile metadata | possible mirror | primary export consumer | unsupported except comments/stereotypes | unsupported except labels/classes | unsupported or style properties only | properties only | defer |
| Portfolio objects | canonical | possible mirror | labels/summaries only | mostly unsupported | mostly unsupported | partial through software-system/C4 model mapping | primary semantic target | out of initial scope |
| Portfolio views | canonical | possible mirror | lifecycle roadmap and future generated views | limited generated diagrams only | limited generated diagrams only | views once C4 mapping exists | viewpoints/views where supported | out of initial scope |
| Roadmap presentations | canonical | possible mirror | generated timeline/roadmap output | limited Gantt-like export only if useful later | limited timeline/flow export only if useful later | mostly unsupported | plateaus/work packages with warnings | out of initial scope |
| Trace links | canonical | possible mirror | metadata/annotations only | comments only | comments only | properties only | properties only | out of initial scope |
| Proposal transactions | canonical package workflow | possible mirror for drafts only | export evidence only | unsupported except comments | unsupported except comments | unsupported except properties | unsupported except properties | unsupported |
| Provenance/source refs | canonical | possible mirror | metadata and visible warnings | comments/metadata where possible | comments/metadata where possible | properties | properties and source IDs | properties/profile-specific metadata |

## Warning Categories

Adapters should share warning categories so CLI, workbench, and tests can agree on outcomes:

- `unsupported_concept`: the target format has no useful representation for a RedShield concept
- `lossy_semantics`: the target can represent something similar but drops native meaning
- `lossy_layout`: diagram geometry, labels, ports, or renderer state cannot round-trip
- `lossy_provenance`: source refs, external refs, proposal IDs, reviewer state, or Git commit references cannot round-trip
- `ambiguous_mapping`: multiple target concepts are plausible and adapter policy chose a default
- `unresolved_external_ref`: `package:` or `source:` refs cannot be resolved in the adapter context
- `unsafe_asset`: an asset or renderer cannot be safely embedded, executed, fetched, or converted
- `unsupported_import_syntax`: an importer saw valid target syntax outside RedShield's supported subset

Warnings may allow diagnostic or preview exports. Importers should attach warnings to generated proposal previews so reviewers can accept, reject, or revise the conversion.

## First Implementation Slices

1. Define a shared export-scene structure for generated SVG/PDF output and text diagram exporters.
2. Add SVG export fixture tests that assert RedShield IDs, labels, and unsafe asset warnings.
3. Add PlantUML export for the supported use-case, class, component, activity, and sequence subset.
4. Add Mermaid export for the overlapping lightweight subset.
5. Add golden fixtures for every supported exporter, including warning snapshots.
6. Define Structurizr DSL mapping for software systems, containers, components, people, relationships, and views before writing the adapter.
7. Add ArchiMate exchange export preview for portfolio objects using [ArchiMate Mapping Matrix](ARCHIMATE_MAPPING_MATRIX.md).
8. Defer XMI until RedShield has stable UML update/delete operations and one explicit XMI profile target.

## Source Notes

Format references checked while defining the matrix:

- OMG XMI specification: <https://www.omg.org/spec/XMI/>
- PlantUML: <https://plantuml.com/>
- Mermaid syntax reference: <https://mermaid.ai/open-source/intro/syntax-reference.html>
- Structurizr DSL language reference: <https://docs.structurizr.com/dsl/language>
- The Open Group ArchiMate Model Exchange File Format: <https://www.opengroup.org/open-group-archimate-model-exchange-file-format>
- W3C SVG 2: <https://www.w3.org/TR/SVG2/>
- PDF Association ISO 32000-2 resource: <https://pdfa.org/resource/iso-32000-2/>
- IETF JSON RFC 8259: <https://datatracker.ietf.org/doc/html/rfc8259>
- YAML 1.2.2: <https://yaml.org/spec/1.2.2/>
