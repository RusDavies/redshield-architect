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
