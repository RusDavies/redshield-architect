use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub schema_version: String,
    pub project_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RequirementFile {
    pub schema_version: String,
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub statement: String,
    pub status: String,
    pub priority: String,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ElementFile {
    pub schema_version: String,
    pub elements: Vec<ModelElement>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelElement {
    pub id: String,
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipFile {
    pub schema_version: String,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Relationship {
    pub id: String,
    pub relationship_kind: String,
    pub source_id: String,
    pub target_id: String,
    #[serde(default)]
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramFile {
    pub schema_version: String,
    pub diagrams: Vec<DiagramView>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramView {
    pub id: String,
    pub title: String,
    pub view_kind: String,
    pub model_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TraceFile {
    pub schema_version: String,
    pub links: Vec<TraceLink>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TraceLink {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub trace_kind: String,
    #[serde(default)]
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ModelPackage {
    pub root: PathBuf,
    pub manifest: Manifest,
    pub requirements: RequirementFile,
    pub elements: ElementFile,
    pub relationships: RelationshipFile,
    pub diagrams: DiagramFile,
    pub trace: TraceFile,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    pub proposal_id: String,
    pub schema_version: String,
    pub state: String,
    pub created_at: String,
    pub intent: String,
    #[serde(default)]
    pub operations: Vec<ProposalOperation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProposalOperation {
    pub op_id: String,
    pub op: String,
    pub args: Value,
    #[serde(default)]
    pub rationale: String,
    #[serde(default)]
    pub source_refs: Vec<String>,
}

pub fn load_package(root: impl AsRef<Path>) -> Result<ModelPackage> {
    let root = root.as_ref().to_path_buf();
    Ok(ModelPackage {
        manifest: read_json(root.join("manifest.json"))?,
        requirements: read_json(root.join("requirements/requirements.json"))?,
        elements: read_json(root.join("model/elements.json"))?,
        relationships: read_json(root.join("model/relationships.json"))?,
        diagrams: read_json(root.join("views/diagrams.json"))?,
        trace: read_json(root.join("trace/links.json"))?,
        root,
    })
}

pub fn validate_package(package: &ModelPackage) -> Result<Vec<String>> {
    let mut warnings = Vec::new();
    require_version("manifest", &package.manifest.schema_version)?;
    require_version("requirements", &package.requirements.schema_version)?;
    require_version("elements", &package.elements.schema_version)?;
    require_version("relationships", &package.relationships.schema_version)?;
    require_version("diagrams", &package.diagrams.schema_version)?;
    require_version("trace", &package.trace.schema_version)?;

    let mut ids = BTreeSet::new();
    for req in &package.requirements.requirements {
        ensure_unique(&mut ids, &req.id)?;
        ensure_non_empty(&req.title, &format!("{} title", req.id))?;
        ensure_non_empty(&req.statement, &format!("{} statement", req.id))?;
        if req.acceptance_criteria.is_empty() {
            warnings.push(format!("{} has no acceptance criteria", req.id));
        }
    }

    let mut element_kinds = BTreeMap::new();
    for element in &package.elements.elements {
        ensure_unique(&mut ids, &element.id)?;
        ensure_non_empty(&element.name, &format!("{} name", element.id))?;
        if !matches!(
            element.kind.as_str(),
            "actor" | "use_case" | "class" | "component" | "activity" | "sequence_participant"
        ) {
            bail!(
                "{} has unsupported element kind {}",
                element.id,
                element.kind
            );
        }
        element_kinds.insert(element.id.as_str(), element.kind.as_str());
    }

    for relationship in &package.relationships.relationships {
        ensure_unique(&mut ids, &relationship.id)?;
        if !ids.contains(relationship.source_id.as_str()) {
            bail!(
                "{} references missing source {}",
                relationship.id,
                relationship.source_id
            );
        }
        if !ids.contains(relationship.target_id.as_str()) {
            bail!(
                "{} references missing target {}",
                relationship.id,
                relationship.target_id
            );
        }
        if !matches!(
            relationship.relationship_kind.as_str(),
            "association" | "include" | "extend" | "trace" | "dependency"
        ) {
            bail!(
                "{} has unsupported relationship kind {}",
                relationship.id,
                relationship.relationship_kind
            );
        }
    }

    for diagram in &package.diagrams.diagrams {
        ensure_unique(&mut ids, &diagram.id)?;
        if diagram.view_kind != "use_case" {
            bail!(
                "{} has unsupported view kind {}",
                diagram.id,
                diagram.view_kind
            );
        }
        for model_ref in &diagram.model_refs {
            if !element_kinds.contains_key(model_ref.as_str()) {
                bail!(
                    "{} references missing model element {}",
                    diagram.id,
                    model_ref
                );
            }
        }
    }

    for link in &package.trace.links {
        ensure_unique(&mut ids, &link.id)?;
        if !ids.contains(link.source_id.as_str()) {
            bail!("{} references missing source {}", link.id, link.source_id);
        }
        if !ids.contains(link.target_id.as_str()) {
            bail!("{} references missing target {}", link.id, link.target_id);
        }
    }

    Ok(warnings)
}

pub fn validate_proposals(root: impl AsRef<Path>) -> Result<Vec<String>> {
    let proposal_dir = root.as_ref().join("proposals/open");
    if !proposal_dir.exists() {
        return Ok(vec!["no open proposal directory found".to_string()]);
    }

    let mut warnings = Vec::new();
    for entry in fs::read_dir(&proposal_dir)
        .with_context(|| format!("reading {}", proposal_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let proposal: Proposal = read_json(&path)?;
        validate_proposal(&proposal).with_context(|| format!("validating {}", path.display()))?;
        if proposal.operations.is_empty() {
            warnings.push(format!("{} contains no operations", proposal.proposal_id));
        }
    }
    Ok(warnings)
}

pub fn validate_proposal(proposal: &Proposal) -> Result<()> {
    require_version("proposal", &proposal.schema_version)?;
    ensure_non_empty(&proposal.proposal_id, "proposalId")?;
    ensure_non_empty(&proposal.intent, "intent")?;
    if !matches!(
        proposal.state.as_str(),
        "draft" | "validation_pending" | "review_ready" | "accepted" | "rejected" | "applied"
    ) {
        bail!(
            "{} has unsupported state {}",
            proposal.proposal_id,
            proposal.state
        );
    }

    let mut op_ids = BTreeSet::new();
    for operation in &proposal.operations {
        ensure_unique(&mut op_ids, &operation.op_id)?;
        if operation.rationale.trim().is_empty() {
            bail!("{} is missing rationale", operation.op_id);
        }
        match operation.op.as_str() {
            "create_requirement" => require_args(&operation.args, &["id", "title", "statement"])?,
            "create_model_element" => require_args(&operation.args, &["id", "kind", "name"])?,
            "create_relationship" => require_args(
                &operation.args,
                &["id", "relationshipKind", "sourceId", "targetId"],
            )?,
            "create_diagram_view" => {
                require_args(&operation.args, &["id", "title", "viewKind", "modelRefs"])?
            }
            "create_trace_link" => require_args(
                &operation.args,
                &["id", "sourceId", "targetId", "traceKind"],
            )?,
            other => bail!("{} uses unsupported operation {}", operation.op_id, other),
        }
    }
    Ok(())
}

pub fn render_use_case_svg(package: &ModelPackage, diagram_id: Option<&str>) -> Result<String> {
    let diagram = match diagram_id {
        Some(id) => package
            .diagrams
            .diagrams
            .iter()
            .find(|diagram| diagram.id == id),
        None => package
            .diagrams
            .diagrams
            .iter()
            .find(|diagram| diagram.view_kind == "use_case"),
    }
    .ok_or_else(|| anyhow!("no matching use-case diagram found"))?;

    let elements: BTreeMap<&str, &ModelElement> = package
        .elements
        .elements
        .iter()
        .map(|element| (element.id.as_str(), element))
        .collect();
    let actors: Vec<&ModelElement> = diagram
        .model_refs
        .iter()
        .filter_map(|id| elements.get(id.as_str()).copied())
        .filter(|element| element.kind == "actor")
        .collect();
    let use_cases: Vec<&ModelElement> = diagram
        .model_refs
        .iter()
        .filter_map(|id| elements.get(id.as_str()).copied())
        .filter(|element| element.kind == "use_case")
        .collect();

    let width = 900;
    let height = 160 + (actors.len().max(use_cases.len()) as i32 * 110).max(260);
    let mut svg = String::new();
    writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-labelledby="title desc">"#
    )?;
    writeln!(svg, "<title>{}</title>", escape_xml(&diagram.title))?;
    writeln!(
        svg,
        "<desc>Use-case diagram rendered from RedShield semantic model data.</desc>"
    )?;
    writeln!(
        svg,
        r##"<rect width="100%" height="100%" fill="#f8fafc"/>"##
    )?;
    writeln!(
        svg,
        r##"<rect x="250" y="72" width="570" height="{}" rx="8" fill="#ffffff" stroke="#334155" stroke-width="2"/>"##,
        height - 120
    )?;
    writeln!(
        svg,
        r##"<text x="270" y="106" font-family="Inter, Arial, sans-serif" font-size="18" font-weight="700" fill="#0f172a">{}</text>"##,
        escape_xml(&package.manifest.name)
    )?;

    for (index, actor) in actors.iter().enumerate() {
        let y = 170 + index as i32 * 110;
        writeln!(
            svg,
            r##"<circle cx="95" cy="{y}" r="18" fill="none" stroke="#0f766e" stroke-width="3"/>"##
        )?;
        writeln!(
            svg,
            r##"<line x1="95" y1="{}" x2="95" y2="{}" stroke="#0f766e" stroke-width="3"/>"##,
            y + 18,
            y + 58
        )?;
        writeln!(
            svg,
            r##"<line x1="62" y1="{}" x2="128" y2="{}" stroke="#0f766e" stroke-width="3"/>"##,
            y + 34,
            y + 34
        )?;
        writeln!(
            svg,
            r##"<line x1="95" y1="{}" x2="66" y2="{}" stroke="#0f766e" stroke-width="3"/>"##,
            y + 58,
            y + 92
        )?;
        writeln!(
            svg,
            r##"<line x1="95" y1="{}" x2="124" y2="{}" stroke="#0f766e" stroke-width="3"/>"##,
            y + 58,
            y + 92
        )?;
        writeln!(
            svg,
            r##"<text x="95" y="{}" text-anchor="middle" font-family="Inter, Arial, sans-serif" font-size="14" fill="#134e4a">{}</text>"##,
            y + 118,
            escape_xml(&actor.name)
        )?;
    }

    for (index, use_case) in use_cases.iter().enumerate() {
        let y = 160 + index as i32 * 110;
        writeln!(
            svg,
            r##"<ellipse cx="535" cy="{y}" rx="185" ry="42" fill="#ecfeff" stroke="#0369a1" stroke-width="2"/>"##
        )?;
        writeln!(
            svg,
            r##"<text x="535" y="{}" text-anchor="middle" font-family="Inter, Arial, sans-serif" font-size="15" font-weight="700" fill="#0c4a6e">{}</text>"##,
            y + 5,
            escape_xml(&use_case.name)
        )?;
    }

    for relationship in &package.relationships.relationships {
        let Some(source) = elements.get(relationship.source_id.as_str()) else {
            continue;
        };
        let Some(target) = elements.get(relationship.target_id.as_str()) else {
            continue;
        };
        if source.kind != "actor" || target.kind != "use_case" {
            continue;
        }
        let actor_index = actors.iter().position(|actor| actor.id == source.id);
        let use_case_index = use_cases
            .iter()
            .position(|use_case| use_case.id == target.id);
        if let (Some(actor_index), Some(use_case_index)) = (actor_index, use_case_index) {
            let y1 = 204 + actor_index as i32 * 110;
            let y2 = 160 + use_case_index as i32 * 110;
            writeln!(
                svg,
                r##"<line x1="132" y1="{y1}" x2="350" y2="{y2}" stroke="#475569" stroke-width="2"/>"##
            )?;
        }
    }

    writeln!(svg, "</svg>")?;
    Ok(svg)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let contents =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&contents).with_context(|| format!("parsing {}", path.display()))
}

fn ensure_unique<'a>(ids: &mut BTreeSet<&'a str>, id: &'a str) -> Result<()> {
    ensure_non_empty(id, "id")?;
    if !ids.insert(id) {
        bail!("duplicate id {id}");
    }
    Ok(())
}

fn ensure_non_empty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{field} must not be empty");
    }
    Ok(())
}

fn require_version(label: &str, version: &str) -> Result<()> {
    if version != "0.1.0" {
        bail!("{label} uses unsupported schema version {version}");
    }
    Ok(())
}

fn require_args(args: &Value, required: &[&str]) -> Result<()> {
    let object = args
        .as_object()
        .ok_or_else(|| anyhow!("operation args must be an object"))?;
    for field in required {
        if !object.contains_key(*field) {
            bail!("operation args missing {field}");
        }
    }
    Ok(())
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_validation_accepts_typed_operations() {
        let proposal: Proposal = serde_json::from_str(include_str!(
            "../examples/minimal/redshield/proposals/open/create-first-use-case.json"
        ))
        .unwrap();
        validate_proposal(&proposal).unwrap();
        let encoded = serde_json::to_string_pretty(&proposal).unwrap();
        let decoded: Proposal = serde_json::from_str(&encoded).unwrap();
        assert_eq!(proposal, decoded);
    }

    #[test]
    fn model_package_validates_and_renders_svg() {
        let package = load_package("examples/minimal/redshield").unwrap();
        let warnings = validate_package(&package).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        let proposal_warnings = validate_proposals("examples/minimal/redshield").unwrap();
        assert!(proposal_warnings.is_empty(), "{proposal_warnings:?}");
        let svg = render_use_case_svg(&package, Some("diagram.first-use-case")).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Review proposal"));
    }
}
