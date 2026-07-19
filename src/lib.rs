use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramFile {
    pub schema_version: String,
    pub diagrams: Vec<DiagramView>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramView {
    pub id: String,
    pub title: String,
    pub view_kind: String,
    pub model_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<DiagramLayout>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramLayout {
    pub coordinate_system: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub layout_engine: String,
    pub layout_state: String,
    #[serde(default)]
    pub nodes: Vec<DiagramNodeLayout>,
    #[serde(default)]
    pub connectors: Vec<DiagramConnectorLayout>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramNodeLayout {
    pub model_ref: String,
    pub bounds: DiagramBounds,
    pub layout_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_position: Option<DiagramPoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramConnectorLayout {
    pub relationship_ref: String,
    pub layout_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_hint: Option<DiagramRouteHint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_position: Option<DiagramPoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramRouteHint {
    pub kind: String,
    #[serde(default)]
    pub points: Vec<DiagramPoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramPoint {
    pub x: f64,
    pub y: f64,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplySummary {
    pub requirements_created: usize,
    pub elements_created: usize,
    pub relationships_created: usize,
    pub diagrams_created: usize,
    pub trace_links_created: usize,
    pub applied_proposal_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRequirementArgs {
    id: String,
    title: String,
    statement: String,
    #[serde(default = "default_status")]
    status: String,
    #[serde(default = "default_priority")]
    priority: String,
    #[serde(default)]
    acceptance_criteria: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateModelElementArgs {
    id: String,
    kind: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRelationshipArgs {
    id: String,
    relationship_kind: String,
    source_id: String,
    target_id: String,
    #[serde(default)]
    label: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateDiagramViewArgs {
    id: String,
    title: String,
    view_kind: String,
    model_refs: Vec<String>,
    #[serde(default)]
    layout: Option<DiagramLayout>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTraceLinkArgs {
    id: String,
    source_id: String,
    target_id: String,
    trace_kind: String,
    #[serde(default)]
    confidence: Option<f32>,
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

pub fn apply_accepted_proposal_file(
    root: impl AsRef<Path>,
    proposal_path: impl AsRef<Path>,
) -> Result<ApplySummary> {
    let root = root.as_ref();
    let proposal_path = proposal_path.as_ref();
    let mut package = load_package(root)?;
    let mut proposal: Proposal = read_json(proposal_path)?;
    validate_proposal(&proposal)
        .with_context(|| format!("validating {}", proposal_path.display()))?;

    if proposal.state != "accepted" {
        bail!(
            "{} must be in accepted state before application",
            proposal.proposal_id
        );
    }

    let mut summary = apply_proposal_operations(&mut package, &proposal)?;
    validate_package(&package)?;
    write_package(&package)?;

    proposal.state = "applied".to_string();
    let applied_dir = root.join("proposals/applied");
    fs::create_dir_all(&applied_dir)
        .with_context(|| format!("creating {}", applied_dir.display()))?;
    let applied_path = applied_dir.join(
        proposal_path
            .file_name()
            .ok_or_else(|| anyhow!("proposal path has no file name"))?,
    );
    write_json(&applied_path, &proposal)?;
    summary.applied_proposal_path = applied_path;
    Ok(summary)
}

pub fn apply_proposal_operations(
    package: &mut ModelPackage,
    proposal: &Proposal,
) -> Result<ApplySummary> {
    let mut summary = ApplySummary {
        requirements_created: 0,
        elements_created: 0,
        relationships_created: 0,
        diagrams_created: 0,
        trace_links_created: 0,
        applied_proposal_path: PathBuf::new(),
    };

    for operation in &proposal.operations {
        match operation.op.as_str() {
            "create_requirement" => {
                let args: CreateRequirementArgs = parse_args(operation)?;
                ensure_available_id(package, &args.id)?;
                package.requirements.requirements.push(Requirement {
                    id: args.id,
                    title: args.title,
                    statement: args.statement,
                    status: args.status,
                    priority: args.priority,
                    acceptance_criteria: args.acceptance_criteria,
                    tags: args.tags,
                });
                summary.requirements_created += 1;
            }
            "create_model_element" => {
                let args: CreateModelElementArgs = parse_args(operation)?;
                ensure_available_id(package, &args.id)?;
                package.elements.elements.push(ModelElement {
                    id: args.id,
                    kind: args.kind,
                    name: args.name,
                    description: args.description,
                    tags: args.tags,
                });
                summary.elements_created += 1;
            }
            "create_relationship" => {
                let args: CreateRelationshipArgs = parse_args(operation)?;
                ensure_available_id(package, &args.id)?;
                package.relationships.relationships.push(Relationship {
                    id: args.id,
                    relationship_kind: args.relationship_kind,
                    source_id: args.source_id,
                    target_id: args.target_id,
                    label: args.label,
                });
                summary.relationships_created += 1;
            }
            "create_diagram_view" => {
                let args: CreateDiagramViewArgs = parse_args(operation)?;
                ensure_available_id(package, &args.id)?;
                package.diagrams.diagrams.push(DiagramView {
                    id: args.id,
                    title: args.title,
                    view_kind: args.view_kind,
                    model_refs: args.model_refs,
                    layout: args.layout,
                });
                summary.diagrams_created += 1;
            }
            "create_trace_link" => {
                let args: CreateTraceLinkArgs = parse_args(operation)?;
                ensure_available_id(package, &args.id)?;
                package.trace.links.push(TraceLink {
                    id: args.id,
                    source_id: args.source_id,
                    target_id: args.target_id,
                    trace_kind: args.trace_kind,
                    confidence: args.confidence,
                });
                summary.trace_links_created += 1;
            }
            other => bail!("{} uses unsupported operation {}", operation.op_id, other),
        }
    }

    sort_package(package);
    Ok(summary)
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
    let relationship_ids: BTreeSet<&str> = package
        .relationships
        .relationships
        .iter()
        .map(|relationship| relationship.id.as_str())
        .collect();

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
        if let Some(layout) = &diagram.layout {
            validate_diagram_layout(diagram, layout, &element_kinds, &relationship_ids)?;
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

fn validate_diagram_layout(
    diagram: &DiagramView,
    layout: &DiagramLayout,
    element_kinds: &BTreeMap<&str, &str>,
    relationship_ids: &BTreeSet<&str>,
) -> Result<()> {
    if layout.coordinate_system != "canvas" {
        bail!(
            "{} has unsupported layout coordinate system {}",
            diagram.id,
            layout.coordinate_system
        );
    }
    if !matches!(
        layout.layout_state.as_str(),
        "generated" | "manual" | "mixed"
    ) {
        bail!(
            "{} has unsupported layout state {}",
            diagram.id,
            layout.layout_state
        );
    }

    let diagram_refs: BTreeSet<&str> = diagram.model_refs.iter().map(String::as_str).collect();
    let mut node_refs = BTreeSet::new();
    for node in &layout.nodes {
        ensure_unique(&mut node_refs, &node.model_ref)?;
        if !element_kinds.contains_key(node.model_ref.as_str()) {
            bail!(
                "{} layout references missing model element {}",
                diagram.id,
                node.model_ref
            );
        }
        if !diagram_refs.contains(node.model_ref.as_str()) {
            bail!(
                "{} layout node {} is not in modelRefs",
                diagram.id,
                node.model_ref
            );
        }
        validate_bounds(&diagram.id, &node.model_ref, &node.bounds)?;
        validate_layout_state(&diagram.id, &node.model_ref, &node.layout_state)?;
    }

    let mut connector_refs = BTreeSet::new();
    for connector in &layout.connectors {
        ensure_unique(&mut connector_refs, &connector.relationship_ref)?;
        if !relationship_ids.contains(connector.relationship_ref.as_str()) {
            bail!(
                "{} layout references missing relationship {}",
                diagram.id,
                connector.relationship_ref
            );
        }
        validate_layout_state(
            &diagram.id,
            &connector.relationship_ref,
            &connector.layout_state,
        )?;
        if let Some(route_hint) = &connector.route_hint {
            if !matches!(
                route_hint.kind.as_str(),
                "straight" | "step" | "smoothstep" | "bezier" | "orthogonal"
            ) {
                bail!(
                    "{} connector {} has unsupported route hint {}",
                    diagram.id,
                    connector.relationship_ref,
                    route_hint.kind
                );
            }
        }
    }

    Ok(())
}

fn validate_bounds(diagram_id: &str, model_ref: &str, bounds: &DiagramBounds) -> Result<()> {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        bail!("{diagram_id} layout node {model_ref} must have positive bounds");
    }
    Ok(())
}

fn validate_layout_state(diagram_id: &str, object_ref: &str, layout_state: &str) -> Result<()> {
    if !matches!(layout_state, "generated" | "manual") {
        bail!(
            "{diagram_id} layout object {object_ref} has unsupported layout state {layout_state}"
        );
    }
    Ok(())
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
    render_dot_to_svg(&render_use_case_dot(package, diagram_id)?)
}

pub fn render_use_case_dot(package: &ModelPackage, diagram_id: Option<&str>) -> Result<String> {
    let diagram = find_use_case_diagram(package, diagram_id)?;
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

    let mut dot = String::new();
    writeln!(
        dot,
        "digraph {} {{",
        dot_id(diagram.id.strip_prefix("diagram.").unwrap_or(&diagram.id))
    )?;
    writeln!(
        dot,
        "  graph [rankdir=LR, bgcolor=\"{}\", pad=\"0.35\", nodesep=\"0.8\", ranksep=\"1.2\", label=\"{}\", labelloc=t, fontsize=20, fontname=\"Inter, Arial, sans-serif\", fontcolor=\"{}\"]",
        "#f8fafc",
        escape_dot_label(&diagram.title),
        "#0f172a"
    )?;
    writeln!(
        dot,
        "  node [fontname=\"Inter, Arial, sans-serif\", fontsize=12, style=\"filled\", color=\"{}\", fontcolor=\"{}\"]",
        "#334155", "#0f172a"
    )?;
    writeln!(
        dot,
        "  edge [fontname=\"Inter, Arial, sans-serif\", fontsize=10, color=\"{}\", fontcolor=\"{}\"]",
        "#475569", "#475569"
    )?;

    for actor in &actors {
        writeln!(
            dot,
            "  {} [id=\"{}\", label=\"{}\", shape=box, fillcolor=\"{}\", color=\"{}\", fontcolor=\"{}\", tooltip=\"{}\"]",
            dot_id(&actor.id),
            escape_dot_label(&actor.id),
            escape_dot_label(&actor.name),
            "#ccfbf1",
            "#0f766e",
            "#134e4a",
            escape_dot_label(&actor.id)
        )?;
    }

    writeln!(
        dot,
        "  subgraph cluster_system {{\n    id=\"{}\"\n    label=\"{}\"\n    color=\"{}\"\n    fillcolor=\"{}\"\n    style=\"rounded,filled\"\n    fontname=\"Inter, Arial, sans-serif\"\n    fontsize=16\n    fontcolor=\"{}\"",
        escape_dot_label(&format!("cluster.{}", package.manifest.project_id)),
        escape_dot_label(&package.manifest.name),
        "#334155",
        "#ffffff",
        "#0f172a"
    )?;

    for use_case in &use_cases {
        writeln!(
            dot,
            "    {} [id=\"{}\", label=\"{}\", shape=ellipse, fillcolor=\"{}\", color=\"{}\", fontcolor=\"{}\", tooltip=\"{}\"]",
            dot_id(&use_case.id),
            escape_dot_label(&use_case.id),
            escape_dot_label(&use_case.name),
            "#ecfeff",
            "#0369a1",
            "#0c4a6e",
            escape_dot_label(&use_case.id)
        )?;
    }
    writeln!(dot, "  }}")?;

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
        if actors.iter().any(|actor| actor.id == source.id)
            && use_cases.iter().any(|use_case| use_case.id == target.id)
        {
            writeln!(
                dot,
                "  {} -> {} [id=\"{}\", label=\"{}\", dir=none, tooltip=\"{}\"]",
                dot_id(&source.id),
                dot_id(&target.id),
                escape_dot_label(&relationship.id),
                escape_dot_label(&relationship.label),
                escape_dot_label(&relationship.id)
            )?;
        }
    }

    writeln!(dot, "}}")?;
    Ok(dot)
}

pub fn render_dot_to_svg(dot: &str) -> Result<String> {
    let mut child = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("starting Graphviz dot")?;

    child
        .stdin
        .as_mut()
        .ok_or_else(|| anyhow!("opening dot stdin"))?
        .write_all(dot.as_bytes())
        .context("writing DOT to Graphviz")?;

    let output = child
        .wait_with_output()
        .context("waiting for Graphviz dot")?;
    if !output.status.success() {
        bail!(
            "Graphviz dot failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let svg = String::from_utf8(output.stdout).context("Graphviz produced non-UTF8 SVG")?;
    if !svg.contains("<svg") {
        bail!("Graphviz output did not contain an SVG document");
    }
    Ok(svg)
}

fn find_use_case_diagram<'a>(
    package: &'a ModelPackage,
    diagram_id: Option<&str>,
) -> Result<&'a DiagramView> {
    match diagram_id {
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
    .ok_or_else(|| anyhow!("no matching use-case diagram found"))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let contents =
        fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&contents).with_context(|| format!("parsing {}", path.display()))
}

fn write_package(package: &ModelPackage) -> Result<()> {
    write_json(package.root.join("manifest.json"), &package.manifest)?;
    write_json(
        package.root.join("requirements/requirements.json"),
        &package.requirements,
    )?;
    write_json(package.root.join("model/elements.json"), &package.elements)?;
    write_json(
        package.root.join("model/relationships.json"),
        &package.relationships,
    )?;
    write_json(package.root.join("views/diagrams.json"), &package.diagrams)?;
    write_json(package.root.join("trace/links.json"), &package.trace)?;
    Ok(())
}

fn write_json(path: impl AsRef<Path>, value: &impl Serialize) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut contents = serde_json::to_string_pretty(value)
        .with_context(|| format!("serializing {}", path.display()))?;
    contents.push('\n');
    fs::write(path, contents).with_context(|| format!("writing {}", path.display()))
}

fn parse_args<T: for<'de> Deserialize<'de>>(operation: &ProposalOperation) -> Result<T> {
    serde_json::from_value(operation.args.clone())
        .with_context(|| format!("parsing args for {}", operation.op_id))
}

fn ensure_available_id(package: &ModelPackage, id: &str) -> Result<()> {
    let mut ids = BTreeSet::new();
    for existing in &package.requirements.requirements {
        ids.insert(existing.id.as_str());
    }
    for existing in &package.elements.elements {
        ids.insert(existing.id.as_str());
    }
    for existing in &package.relationships.relationships {
        ids.insert(existing.id.as_str());
    }
    for existing in &package.diagrams.diagrams {
        ids.insert(existing.id.as_str());
    }
    for existing in &package.trace.links {
        ids.insert(existing.id.as_str());
    }
    if ids.contains(id) {
        bail!("cannot create duplicate id {id}");
    }
    Ok(())
}

fn sort_package(package: &mut ModelPackage) {
    package
        .requirements
        .requirements
        .sort_by(|left, right| left.id.cmp(&right.id));
    package
        .elements
        .elements
        .sort_by(|left, right| left.id.cmp(&right.id));
    package
        .relationships
        .relationships
        .sort_by(|left, right| left.id.cmp(&right.id));
    package
        .diagrams
        .diagrams
        .sort_by(|left, right| left.id.cmp(&right.id));
    package
        .trace
        .links
        .sort_by(|left, right| left.id.cmp(&right.id));
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

fn default_status() -> String {
    "proposed".to_string()
}

fn default_priority() -> String {
    "must".to_string()
}

fn dot_id(input: &str) -> String {
    let mut id = String::from("n_");
    for character in input.chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            id.push(character);
        } else {
            id.push('_');
        }
    }
    id
}

fn escape_dot_label(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
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
        let layout = package.diagrams.diagrams[0].layout.as_ref().unwrap();
        assert_eq!(layout.coordinate_system, "canvas");
        assert_eq!(layout.layout_state, "mixed");
        assert_eq!(layout.nodes.len(), 3);
        assert_eq!(layout.connectors.len(), 2);
        let proposal_warnings = validate_proposals("examples/minimal/redshield").unwrap();
        assert!(proposal_warnings.is_empty(), "{proposal_warnings:?}");
        let dot = render_use_case_dot(&package, Some("diagram.first-use-case")).unwrap();
        assert!(dot.contains("digraph"));
        assert!(dot.contains("actor.architect"));
        let svg = render_use_case_svg(&package, Some("diagram.first-use-case")).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Review proposal"));
    }

    #[test]
    fn accepted_proposal_applies_to_canonical_files() {
        let root = copy_example_to_temp();
        let proposal_path = root.join("proposals/open/accepted-add-export-use-case.json");
        fs::write(
            &proposal_path,
            r#"{
  "proposalId": "proposal.add-export-use-case",
  "schemaVersion": "0.1.0",
  "state": "accepted",
  "createdAt": "2026-07-19T20:00:00Z",
  "intent": "Add an export use case to the accepted model.",
  "operations": [
    {
      "opId": "op.create-export-use-case",
      "op": "create_model_element",
      "args": {
        "id": "usecase.export-svg",
        "kind": "use_case",
        "name": "Export SVG"
      },
      "rationale": "SVG export is part of the thin prototype acceptance path.",
      "sourceRefs": ["source.roadmap"]
    },
    {
      "opId": "op.link-architect-export",
      "op": "create_relationship",
      "args": {
        "id": "rel.architect-export",
        "relationshipKind": "association",
        "sourceId": "actor.architect",
        "targetId": "usecase.export-svg",
        "label": "exports"
      },
      "rationale": "The existing architect actor initiates SVG export.",
      "sourceRefs": ["source.roadmap"]
    },
    {
      "opId": "op.trace-export",
      "op": "create_trace_link",
      "args": {
        "id": "trace.render-export",
        "sourceId": "req.review-ai-proposals",
        "targetId": "usecase.export-svg",
        "traceKind": "satisfies",
        "confidence": 0.8
      },
      "rationale": "The export use case supports the rendered diagram acceptance criteria.",
      "sourceRefs": ["source.requirements"]
    }
  ]
}
"#,
        )
        .unwrap();

        let summary = apply_accepted_proposal_file(&root, &proposal_path).unwrap();
        assert_eq!(summary.elements_created, 1);
        assert_eq!(summary.relationships_created, 1);
        assert_eq!(summary.trace_links_created, 1);

        let package = load_package(&root).unwrap();
        validate_package(&package).unwrap();
        assert!(
            package
                .elements
                .elements
                .iter()
                .any(|element| element.id == "usecase.export-svg")
        );
        let applied = fs::read_to_string(summary.applied_proposal_path).unwrap();
        assert!(applied.contains(r#""state": "applied""#));
    }

    #[test]
    fn validation_rejects_broken_diagram_layout_references() {
        let mut package = load_package("examples/minimal/redshield").unwrap();
        let layout = package.diagrams.diagrams[0].layout.as_mut().unwrap();
        layout.connectors[0].relationship_ref = "rel.missing".to_string();

        let error = validate_package(&package).unwrap_err().to_string();
        assert!(
            error.contains("rel.missing"),
            "expected missing relationship error, got {error}"
        );
    }

    fn copy_example_to_temp() -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "redshield-apply-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        copy_dir(Path::new("examples/minimal/redshield"), &root).unwrap();
        root
    }

    fn copy_dir(source: &Path, target: &Path) -> Result<()> {
        fs::create_dir_all(target)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let target_path = target.join(entry.file_name());
            if source_path.is_dir() {
                copy_dir(&source_path, &target_path)?;
            } else {
                fs::copy(&source_path, &target_path)?;
            }
        }
        Ok(())
    }
}
