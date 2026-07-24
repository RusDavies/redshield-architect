# ArchiMate Mapping Matrix

RedShield remains metamodel-neutral internally. This matrix defines the first explicit compatibility posture for exporting or importing the initial portfolio object kinds to ArchiMate.

The target baseline is ArchiMate 4. The mappings are adapter guidance, not schema inheritance: RedShield package files keep their native object kinds, lifecycle fields, provenance, review state, and proposal history even when an ArchiMate exchange flattens them.

## Mapping Rules

Use this order when an adapter maps a portfolio object:

1. Preserve the RedShield object ID as an external identifier or property.
2. Map to the most specific ArchiMate element that preserves the object's architectural role.
3. Attach lifecycle, status, standard posture, source refs, and proposal/review metadata as properties unless the target exchange format has a better native field.
4. Emit a warning when a mapping depends on context not present in the RedShield object.
5. Prefer a lossy but explicit export over silently changing the RedShield object kind.

## Element Matrix

| RedShield kind | Native intent | Primary ArchiMate target | Alternate target when context proves it | Export posture | Import posture |
| --- | --- | --- | --- | --- | --- |
| `business_capability` | Business or operating ability supported by applications, services, technologies, or initiatives. | `Capability` in the strategy layer. | `Business Function` only when the source model clearly describes performed behavior rather than an ability. | Export as a capability with properties for criticality, lifecycle, owner refs, source refs, and tags. | Import ArchiMate capabilities as `business_capability`; do not turn functions into capabilities unless a rule says the source model uses functions as capability placeholders. |
| `portfolio_application` | Estate application, product, platform, or application-like system under architecture governance. | `Application Component`. | `Application Collaboration` for a logical grouping of application components; `Product` only when modeling a market-facing bundled offer, not an estate application. | Export as an application component by default. Preserve product/application distinction through tags or properties until a profile contract exists. | Import application components as `portfolio_application` unless they are clearly implementation-internal components better represented as UML/model elements. |
| `portfolio_service` | Provided or consumed service connecting capabilities, applications, components, and implementation work. | `Application Service`. | `Business Service` or `Technology Service` when provider and consumer context clearly place the service in those layers. | Export as an application service by default, with a warning if layer context is absent. | Import services to `portfolio_service` with a `source:` identity and layer tag when the ArchiMate service layer matters. |
| `technology_component` | Concrete runtime, framework, platform, database, tool, protocol, library family, or infrastructure component. | `Technology Node` for deployable/runtime infrastructure. | `System Software`, `Technology Service`, `Device`, `Equipment`, `Facility`, or `Artifact` when the source object is clearly software, exposed infrastructure behavior, hardware, physical equipment, location, or deployable file. | Export using the most specific target inferred from tags, standard state, external refs, or adapter policy; otherwise use `Technology Node` and warn. | Import concrete technology-layer structure or software elements as `technology_component`; preserve the source ArchiMate type as a tag/property. |
| `technology_standard` | Governance stance for a technology: approved, tolerated, discouraged, banned, or emerging. | `Constraint` or `Principle` in motivation. | `Requirement` when the standard is a mandatory compliance rule; `Technology Component` only when the source is the technology itself, not its governance posture. | Export the standard's policy posture as motivation metadata linked to affected technology components. | Import standards-like motivation elements as `technology_standard` only when they govern a technology choice. |
| `organization_unit` | Team, department, vendor group, or accountable organizational area. | `Business Actor`. | `Business Collaboration` for a group formed from multiple actors; `Business Role` for an abstract responsibility independent of a named unit. | Export as a business actor with ownership relationships to governed objects. | Import business actors/collaborations as `organization_unit` when they represent durable org structures rather than individual people or roles. |
| `owner` | Durable ownership role or accountable-party reference. | `Stakeholder`. | `Business Role` or `Business Actor` when the owner is a role/team inside the architecture model. | Export as a stakeholder by default and connect through association or assignment-style relationships. | Import stakeholders as `owner`; import business roles/actors as owners only when they are used specifically as accountability references. |
| `lifecycle_milestone` | Dated or named lifecycle event such as launch, deprecation, support end, migration complete, or retirement. | `Plateau`. | `Implementation Event` when the source is explicitly event-like; `Deliverable` when the milestone represents a produced artifact. | Export as plateau when it marks architecture state; include date/status properties. | Import plateaus as milestones when they represent state markers, not complete target-state packages. |
| `roadmap_item` | Planned change that moves architecture from current state toward target state. | `Work Package`. | `Course of Action` for strategic direction; `Deliverable` for concrete output; `Gap` for difference between baseline and target architecture. | Export as a work package unless the item is clearly strategy, artifact, or gap oriented. | Import work packages as `roadmap_item`; preserve deliverables/gaps as refs or properties until RedShield has a richer roadmap contract. |
| `risk` | Architecture, security, operational, delivery, obsolescence, compliance, or continuity risk. | `Assessment`. | `Driver` when the source expresses pressure rather than evaluated risk. | Export as assessment, preserving severity/status as properties. | Import assessments as `risk` only when they describe risk or evaluation; do not import every motivation assessment as a risk. |
| `control` | Required mitigation, governance rule, review gate, policy control, or compliance control. | `Requirement` or `Constraint`. | `Principle` when the control is directional guidance rather than testable obligation. | Export hard controls as requirements/constraints; export guidance controls as principles. | Import requirements/constraints as controls when they govern architecture choices rather than product functional requirements. |
| `governance_decision` | Durable approval, waiver, exception, standardization, or direction-setting decision. | No clean core element; usually a property-backed `Assessment`, `Constraint`, or `Principle`. | `Requirement` for mandatory decisions; `Course of Action` for strategic direction. | Export as motivation element plus properties for decision type, status, approval source, and source refs; always warn that decision lineage is lossy. | Import only when the source object is explicitly a decision, waiver, or approval. Otherwise keep it as source evidence, not a governance decision. |
| `data_source` | Source system or evidence feed for discovered/imported architecture facts. | `Artifact` for a concrete file/feed or `Data Object` for structured application-layer data. | `Resource` when the source is managed as a strategic asset; properties only when it is merely provenance metadata. | Prefer properties/source refs unless the source itself is architecture-relevant. | Import data objects/artifacts as `data_source` only when the source model is describing evidence feeds, inventories, or repositories. |

## Relationship Mapping

RedShield refs are deliberately simpler than ArchiMate relationships. Adapters should map them consistently and warn when the resulting relationship hides RedShield-specific meaning.

| RedShield relationship source | ArchiMate relationship posture | Notes |
| --- | --- | --- |
| `ownerRefs` | `Association` by default; `Assignment` only when the owner actively performs behavior or responsibility in the source context. | Accountability is not always behavior. Do not over-model it. |
| `capabilityRefs` | `Realization` from applications/services/roadmap items to capabilities when support is explicit; otherwise `Association`. | A dependency or tag is weaker than capability realization. |
| `technologyRefs` | `Serving`, `Realization`, or `Association` depending on whether the technology provides a service, implements an element, or is merely referenced. | Preserve unresolved `package:` and `source:` refs as properties if the target model cannot resolve them. |
| `riskRefs` | `Association` from assessment/risk to affected object. | Risk severity/status stays as properties. |
| `relatedElementRefs` | `Association` to the matching exported UML/model element or a property when no target element exists in the ArchiMate export. | RedShield UML elements are not automatically ArchiMate concepts. |
| `lifecycle.milestoneRefs` | `Association` to `Plateau`, `Implementation Event`, or `Deliverable` based on the milestone mapping. | Date fields stay on the RedShield object and/or target milestone element. |
| `sourceRefs` and `externalReferences` | Properties unless the source is modeled as `data_source`. | Provenance is first-class in RedShield; many ArchiMate tools treat it as annotation. |

## Lossiness Warnings

Adapters should produce warnings for at least these cases:

- a `portfolio_service` has no provider/consumer context, so the export defaults to `Application Service`
- a `technology_component` lacks enough detail to choose between node, system software, service, device, facility, equipment, and artifact
- a `technology_standard`, `control`, or `governance_decision` would export as generic motivation metadata
- lifecycle state, proposal review state, source refs, external refs, or Git/proposal lineage cannot round-trip through the target format
- `package:` or `source:` refs cannot be resolved by the adapter's import context

## Adapter Boundary

Do not add ArchiMate-specific fields to `model/portfolio.json` for this matrix alone. The current contract has enough structure for a first compatibility adapter:

- stable object IDs
- bounded object kinds
- lifecycle/status/criticality/standard state
- tags
- source and external references
- typed refs between portfolio objects and model elements
- qualified `package:` and `source:` identity

Add a bounded ArchiMate profile or mapping-hint schema only after an import/export implementation proves that tags, source refs, and adapter policy cannot preserve the required distinction.
