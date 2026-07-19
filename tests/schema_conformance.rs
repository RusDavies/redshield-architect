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
