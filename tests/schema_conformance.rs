use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

const SCHEMA_CASES: &[(&str, &str)] = &[
    (
        "schemas/manifest.schema.json",
        "examples/minimal/redshield/manifest.json",
    ),
    (
        "schemas/requirements.schema.json",
        "examples/minimal/redshield/requirements/requirements.json",
    ),
    (
        "schemas/elements.schema.json",
        "examples/minimal/redshield/model/elements.json",
    ),
    (
        "schemas/relationships.schema.json",
        "examples/minimal/redshield/model/relationships.json",
    ),
    (
        "schemas/diagrams.schema.json",
        "examples/minimal/redshield/views/diagrams.json",
    ),
    (
        "schemas/render-profile.schema.json",
        "examples/minimal/redshield/views/render-profile.json",
    ),
    (
        "schemas/trace.schema.json",
        "examples/minimal/redshield/trace/links.json",
    ),
    (
        "schemas/proposal.schema.json",
        "examples/minimal/redshield/proposals/open/create-first-use-case.json",
    ),
];

#[test]
fn schema_documents_are_valid_json_schema() {
    for (schema_path, _) in SCHEMA_CASES {
        let schema = read_json(schema_path);
        jsonschema::meta::validate(&schema)
            .unwrap_or_else(|error| panic!("{schema_path} is not a valid JSON Schema: {error}"));
    }
}

#[test]
fn minimal_example_conforms_to_published_schemas() {
    for (schema_path, instance_path) in SCHEMA_CASES {
        assert_conforms(schema_path, instance_path);
    }
}

#[test]
fn schema_validation_rejects_invalid_requirement_shape() {
    let schema = read_json("schemas/requirements.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("requirements schema should compile");
    let invalid = json!({
        "schemaVersion": "0.1.0",
        "requirements": [
            {
                "id": "req.missing-statement",
                "title": "Missing statement",
                "status": "accepted",
                "priority": "must"
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "requirements schema should reject missing statement"
    );
}

#[test]
fn schema_validation_rejects_unknown_proposal_operation() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let invalid = json!({
        "proposalId": "proposal.invalid-op",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-19T21:00:00Z",
        "intent": "Try an unsupported operation.",
        "operations": [
            {
                "opId": "op.rename-everything",
                "op": "rename_everything",
                "args": {},
                "rationale": "This operation is intentionally unsupported."
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "proposal schema should reject unknown operation names"
    );
}

#[test]
fn schema_validation_rejects_proposal_operation_missing_required_arg() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let invalid = json!({
        "proposalId": "proposal.invalid-move",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-20T03:10:00Z",
        "intent": "Move a diagram node with incomplete args.",
        "operations": [
            {
                "opId": "op.move-without-model-ref",
                "op": "move_diagram_node",
                "args": {
                    "diagramId": "diagram.first-use-case",
                    "x": 10,
                    "y": 20
                },
                "rationale": "This operation is intentionally malformed."
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "proposal schema should reject missing operation args"
    );
}

#[test]
fn schema_validation_rejects_proposal_operation_extra_arg() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let invalid = json!({
        "proposalId": "proposal.invalid-extra-arg",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-20T03:11:00Z",
        "intent": "Create an element with a stray arg.",
        "operations": [
            {
                "opId": "op.create-element-extra",
                "op": "create_model_element",
                "args": {
                    "id": "actor.architect",
                    "kind": "actor",
                    "name": "Architect",
                    "surprise": "nope"
                },
                "rationale": "This operation is intentionally malformed."
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "proposal schema should reject additional operation args"
    );
}

#[test]
fn schema_validation_accepts_model_element_common_metadata() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let valid = json!({
        "proposalId": "proposal.valid-element-metadata",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-20T15:30:00Z",
        "intent": "Create a model element with common metadata.",
        "operations": [
            {
                "opId": "op.create-component",
                "op": "create_model_element",
                "args": {
                    "id": "component.example",
                    "kind": "component",
                    "name": "Example Component",
                    "aliases": ["Example"],
                    "description": "Short summary.",
                    "documentation": "Longer documentation for the model element.",
                    "status": "accepted",
                    "stereotypes": ["Service"],
                    "tags": ["example"],
                    "provenance": {
                        "sourceRefs": ["source.example"],
                        "createdBy": "schema-test",
                        "createdAt": "2026-07-20T15:30:00Z",
                        "notes": "Seeded by a schema test."
                    },
                    "externalReferences": [
                        {
                            "id": "ref.example",
                            "label": "Example docs",
                            "uri": "docs/MODEL_PACKAGE.md",
                            "kind": "document"
                        }
                    ],
                    "architecture": {
                        "owners": [
                            {
                                "ref": "owner.platform",
                                "role": "accountable",
                                "name": "Platform Architecture"
                            }
                        ],
                        "lifecycle": {
                            "state": "active",
                            "phase": "prototype",
                            "milestoneRefs": ["milestone.local-first"],
                            "targetDate": "2026-07-31",
                            "notes": "Used by the first local workbench."
                        },
                        "criticality": "high",
                        "technologies": [
                            {
                                "ref": "technology.rust",
                                "role": "runtime",
                                "version": "1.89",
                                "standardState": "approved"
                            }
                        ],
                        "risks": [
                            {
                                "ref": "risk.proposal-integrity",
                                "severity": "medium",
                                "status": "mitigating",
                                "notes": "Accepted proposals are validated before application."
                            }
                        ],
                        "capabilities": [
                            {
                                "ref": "capability.model-review",
                                "fit": "primary",
                                "maturity": "developing"
                            }
                        ],
                        "services": [
                            {
                                "ref": "service.proposal-application",
                                "relationship": "provides",
                                "interfaceRef": "operation.apply-proposal"
                            }
                        ]
                    },
                    "classifier": {
                        "isAbstract": false,
                        "attributes": [
                            {
                                "name": "policyName",
                                "visibility": "private",
                                "typeRef": "String",
                                "multiplicity": {
                                    "lower": 1,
                                    "upper": 1
                                },
                                "defaultValue": "default",
                                "isReadOnly": true
                            }
                        ],
                        "operations": [
                            {
                                "name": "apply",
                                "visibility": "public",
                                "returnTypeRef": "Boolean",
                                "parameters": [
                                    {
                                        "name": "element",
                                        "typeRef": "ModelElement",
                                        "direction": "in"
                                    }
                                ]
                            }
                        ]
                    }
                },
                "rationale": "Common metadata should be accepted by create_model_element."
            }
        ]
    });

    if let Err(error) = validator.validate(&valid) {
        panic!("proposal schema should accept model element metadata: {error}");
    }
}

#[test]
fn schema_validation_rejects_invalid_architecture_metadata() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let invalid = json!({
        "proposalId": "proposal.invalid-architecture-metadata",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-20T22:00:00Z",
        "intent": "Create a component with invalid architecture metadata.",
        "operations": [
            {
                "opId": "op.create-component-invalid-architecture",
                "op": "create_model_element",
                "args": {
                    "id": "component.invalid-architecture",
                    "kind": "component",
                    "name": "Invalid Architecture",
                    "architecture": {
                        "criticality": "screaming",
                        "owners": [
                            {
                                "role": "accountable"
                            }
                        ]
                    }
                },
                "rationale": "Architecture metadata should stay bounded."
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "proposal schema should reject invalid architecture metadata"
    );
}

#[test]
fn schema_validation_rejects_classifier_details_on_non_classifier_element() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let invalid = json!({
        "proposalId": "proposal.invalid-classifier-target",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-20T19:40:00Z",
        "intent": "Create an actor with classifier details.",
        "operations": [
            {
                "opId": "op.create-actor",
                "op": "create_model_element",
                "args": {
                    "id": "actor.invalid",
                    "kind": "actor",
                    "name": "Invalid Actor",
                    "classifier": {
                        "operations": [
                            { "name": "act" }
                        ]
                    }
                },
                "rationale": "Actors should not carry classifier details."
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "proposal schema should reject classifier details on non-classifier elements"
    );
}

#[test]
fn schema_validation_accepts_specialized_element_details() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let valid = json!({
        "proposalId": "proposal.valid-specialized-details",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-20T20:40:00Z",
        "intent": "Create behavioral UML element details.",
        "operations": [
            {
                "opId": "op.actor",
                "op": "create_model_element",
                "args": {
                    "id": "actor.reviewer",
                    "kind": "actor",
                    "name": "Reviewer",
                    "actorDetails": {
                        "actorType": "role",
                        "responsibilities": ["Inspect proposals"],
                        "goals": ["Keep model changes reviewed"],
                        "constraints": ["Must not bypass approval"]
                    }
                },
                "rationale": "Actor details should be accepted on actors."
            },
            {
                "opId": "op.use-case",
                "op": "create_model_element",
                "args": {
                    "id": "usecase.review",
                    "kind": "use_case",
                    "name": "Review proposal",
                    "useCaseDetails": {
                        "primaryActorRef": "actor.reviewer",
                        "preconditions": ["Proposal exists"],
                        "postconditions": ["Decision recorded"],
                        "mainFlow": [
                            {
                                "step": 1,
                                "actorRef": "actor.reviewer",
                                "action": "Inspect proposal"
                            }
                        ],
                        "alternateFlows": [
                            {
                                "name": "Reject proposal",
                                "trigger": "Proposal is invalid",
                                "steps": [
                                    { "step": 1, "action": "Reject without applying changes" }
                                ]
                            }
                        ],
                        "extensionPoints": ["Ask agent for rationale"]
                    }
                },
                "rationale": "Use-case details should be accepted on use cases."
            },
            {
                "opId": "op.activity",
                "op": "create_model_element",
                "args": {
                    "id": "activity.review",
                    "kind": "activity",
                    "name": "Review activity",
                    "activityDetails": {
                        "parameters": [
                            { "name": "proposal", "typeRef": "ProposalTransaction", "direction": "in" }
                        ],
                        "nodes": [
                            { "id": "start", "name": "Start", "kind": "initial" },
                            { "id": "inspect", "name": "Inspect", "kind": "action" },
                            { "id": "done", "name": "Done", "kind": "final" }
                        ],
                        "flows": [
                            { "id": "flow.start-inspect", "sourceNodeId": "start", "targetNodeId": "inspect" },
                            { "id": "flow.inspect-done", "sourceNodeId": "inspect", "targetNodeId": "done" }
                        ]
                    }
                },
                "rationale": "Activity details should be accepted on activities."
            },
            {
                "opId": "op.participant",
                "op": "create_model_element",
                "args": {
                    "id": "participant.reviewer",
                    "kind": "sequence_participant",
                    "name": "Reviewer lifeline",
                    "sequenceParticipantDetails": {
                        "participantKind": "actor",
                        "representsRef": "actor.reviewer",
                        "lifelineName": "Reviewer",
                        "isExternal": false
                    }
                },
                "rationale": "Sequence participant details should be accepted on sequence participants."
            }
        ]
    });

    if let Err(error) = validator.validate(&valid) {
        panic!("proposal schema should accept specialized element details: {error}");
    }
}

#[test]
fn schema_validation_rejects_specialized_details_on_wrong_kind() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let invalid = json!({
        "proposalId": "proposal.invalid-specialized-details",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-20T20:45:00Z",
        "intent": "Create a component with use case details.",
        "operations": [
            {
                "opId": "op.component",
                "op": "create_model_element",
                "args": {
                    "id": "component.invalid",
                    "kind": "component",
                    "name": "Invalid Component",
                    "useCaseDetails": {
                        "mainFlow": [
                            { "step": 1, "action": "Should fail" }
                        ]
                    }
                },
                "rationale": "Use-case details should be kind-scoped."
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "proposal schema should reject specialized details on the wrong kind"
    );
}

#[test]
fn schema_validation_accepts_typed_layout_operation_args() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let valid = json!({
        "proposalId": "proposal.valid-layout-ops",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-20T03:12:00Z",
        "intent": "Apply typed layout operations.",
        "operations": [
            {
                "opId": "op.move-actor",
                "op": "move_diagram_node",
                "args": {
                    "diagramId": "diagram.first-use-case",
                    "modelRef": "actor.architect",
                    "x": 100,
                    "y": 120
                },
                "rationale": "Move a node."
            },
            {
                "opId": "op.route-connector",
                "op": "route_diagram_connector",
                "args": {
                    "diagramId": "diagram.first-use-case",
                    "relationshipRef": "rel.architect-render",
                    "routeHint": {
                        "kind": "orthogonal",
                        "points": [
                            { "x": 200, "y": 120 },
                            { "x": 320, "y": 180 }
                        ]
                    }
                },
                "rationale": "Route a connector."
            }
        ]
    });

    if let Err(error) = validator.validate(&valid) {
        panic!("proposal schema should accept typed layout args: {error}");
    }
}

#[test]
fn schema_validation_accepts_typed_render_profile_operation_args() {
    let schema = read_json("schemas/proposal.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("proposal schema should compile");
    let valid = json!({
        "proposalId": "proposal.valid-render-profile-ops",
        "schemaVersion": "0.1.0",
        "state": "accepted",
        "createdAt": "2026-07-20T14:25:00Z",
        "intent": "Apply render profile operations.",
        "operations": [
            {
                "opId": "op.upsert-render-rule",
                "op": "upsert_render_rule",
                "args": {
                    "profileId": "render-profile.default",
                    "rule": {
                        "id": "render.ui.kind.class",
                        "selector": {
                            "elementKind": "class"
                        },
                        "renderAs": {
                            "rendererId": "uml.class",
                            "style": {
                                "fillColor": "#ffffff",
                                "strokeColor": "#334155",
                                "textColor": "#0f172a"
                            }
                        },
                        "precedence": 150,
                        "enabled": true
                    }
                },
                "rationale": "Persist a render rule."
            },
            {
                "opId": "op.remove-render-rule",
                "op": "remove_render_rule",
                "args": {
                    "profileId": "render-profile.default",
                    "ruleId": "render.ui.kind.class"
                },
                "rationale": "Remove a render rule."
            }
        ]
    });

    if let Err(error) = validator.validate(&valid) {
        panic!("proposal schema should accept typed render profile args: {error}");
    }
}

#[test]
fn schema_validation_rejects_invalid_diagram_layout_shape() {
    let schema = read_json("schemas/diagrams.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("diagrams schema should compile");
    let invalid = json!({
        "schemaVersion": "0.1.0",
        "diagrams": [
            {
                "id": "diagram.invalid-layout",
                "title": "Invalid layout",
                "viewKind": "use_case",
                "modelRefs": ["actor.architect"],
                "layout": {
                    "coordinateSystem": "canvas",
                    "layoutState": "mixed",
                    "nodes": [
                        {
                            "modelRef": "actor.architect",
                            "bounds": {
                                "x": 0,
                                "y": 0,
                                "width": 0,
                                "height": 86
                            },
                            "layoutState": "manual"
                        }
                    ],
                    "connectors": []
                }
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "diagrams schema should reject non-positive node bounds"
    );
}

#[test]
fn schema_validation_rejects_empty_render_profile_selector() {
    let schema = read_json("schemas/render-profile.schema.json");
    let validator =
        jsonschema::validator_for(&schema).expect("render profile schema should compile");
    let invalid = json!({
        "schemaVersion": "0.1.0",
        "profiles": [
            {
                "id": "render-profile.invalid",
                "title": "Invalid Render Profile",
                "rules": [
                    {
                        "id": "render.empty-selector",
                        "selector": {},
                        "renderAs": {
                            "rendererId": "uml.class"
                        },
                        "precedence": 100
                    }
                ],
                "fallback": {
                    "rendererId": "uml.class"
                }
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "render profile schema should reject empty selectors"
    );
}

#[test]
fn schema_validation_rejects_image_renderer_without_asset_ref() {
    let schema = read_json("schemas/render-profile.schema.json");
    let validator =
        jsonschema::validator_for(&schema).expect("render profile schema should compile");
    let invalid = json!({
        "schemaVersion": "0.1.0",
        "profiles": [
            {
                "id": "render-profile.invalid-image",
                "title": "Invalid Image Render Profile",
                "rules": [
                    {
                        "id": "render.image-without-asset",
                        "selector": {
                            "elementKind": "class"
                        },
                        "renderAs": {
                            "rendererId": "image.element"
                        },
                        "precedence": 100
                    }
                ],
                "fallback": {
                    "rendererId": "uml.class"
                }
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "render profile schema should reject image renderers without asset refs"
    );
}

#[test]
fn schema_validation_rejects_render_asset_outside_package_asset_path() {
    let schema = read_json("schemas/render-profile.schema.json");
    let validator =
        jsonschema::validator_for(&schema).expect("render profile schema should compile");
    let invalid = json!({
        "schemaVersion": "0.1.0",
        "profiles": [
            {
                "id": "render-profile.invalid-asset-uri",
                "title": "Invalid Asset URI Render Profile",
                "rules": [],
                "fallback": {
                    "rendererId": "uml.class"
                },
                "assets": [
                    {
                        "id": "asset.remote-duck",
                        "uri": "https://example.com/duck.png",
                        "kind": "image/png",
                        "status": "referenced",
                        "provenance": {
                            "sourceType": "imported",
                            "source": "https://example.com/duck.png",
                            "license": "unknown"
                        }
                    }
                ]
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "render profile schema should reject assets outside assets/render/"
    );
}

#[test]
fn schema_validation_rejects_available_render_asset_without_hash() {
    let schema = read_json("schemas/render-profile.schema.json");
    let validator =
        jsonschema::validator_for(&schema).expect("render profile schema should compile");
    let invalid = json!({
        "schemaVersion": "0.1.0",
        "profiles": [
            {
                "id": "render-profile.invalid-available-asset",
                "title": "Invalid Available Asset Render Profile",
                "rules": [],
                "fallback": {
                    "rendererId": "uml.class"
                },
                "assets": [
                    {
                        "id": "asset.available-duck",
                        "uri": "assets/render/duck.png",
                        "kind": "image/png",
                        "status": "available",
                        "provenance": {
                            "sourceType": "user_provided",
                            "source": "duck.png",
                            "license": "user-provided"
                        }
                    }
                ]
            }
        ]
    });

    assert!(
        validator.validate(&invalid).is_err(),
        "render profile schema should require a hash for available assets"
    );
}

fn assert_conforms(schema_path: &str, instance_path: &str) {
    let schema = read_json(schema_path);
    let instance = read_json(instance_path);
    let validator = jsonschema::validator_for(&schema)
        .unwrap_or_else(|error| panic!("{schema_path} failed to compile: {error}"));

    if let Err(error) = validator.validate(&instance) {
        panic!("{instance_path} does not conform to {schema_path}: {error}");
    }
}

fn read_json(path: impl AsRef<Path>) -> Value {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", display(path)));
    serde_json::from_str(&contents)
        .unwrap_or_else(|error| panic!("parsing {}: {error}", display(path)))
}

fn display(path: &Path) -> String {
    PathBuf::from(path).display().to_string()
}
