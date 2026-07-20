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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub documentation: String,
    #[serde(
        default = "default_element_status",
        skip_serializing_if = "is_default_element_status"
    )]
    pub status: String,
    #[serde(default)]
    pub stereotypes: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "ElementProvenance::is_empty")]
    pub provenance: ElementProvenance,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_references: Vec<ExternalReference>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ElementProvenance {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl ElementProvenance {
    fn is_empty(&self) -> bool {
        self.source_refs.is_empty()
            && self.created_by.is_none()
            && self.created_at.is_none()
            && self.notes.is_none()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExternalReference {
    pub id: String,
    pub label: String,
    pub uri: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kind: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<DiagramStyle>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<DiagramStyle>,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DiagramStyle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_style: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderProfileFile {
    pub schema_version: String,
    pub profiles: Vec<RenderProfile>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderProfile {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default)]
    pub rules: Vec<RenderRule>,
    pub fallback: RenderTarget,
    #[serde(default)]
    pub assets: Vec<RenderAsset>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderRule {
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub selector: RenderSelector,
    pub render_as: RenderTarget,
    pub precedence: u32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RenderSelector {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stereotype: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderTarget {
    pub renderer_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<DiagramStyle>,
    #[serde(default)]
    pub ports: Vec<RenderPort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<RenderLabel>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderPort {
    pub id: String,
    pub side: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderLabel {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderAsset {
    pub id: String,
    pub uri: String,
    pub kind: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byte_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<RenderAssetDimensions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
    pub provenance: RenderAssetProvenance,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderAssetDimensions {
    pub width: u64,
    pub height: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderAssetProvenance {
    pub source_type: String,
    pub source: String,
    pub license: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
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
    pub render_profiles: RenderProfileFile,
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
    pub diagram_layout_operations_applied: usize,
    pub render_profile_operations_applied: usize,
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
    aliases: Vec<String>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    documentation: String,
    #[serde(default = "default_element_status")]
    status: String,
    #[serde(default)]
    stereotypes: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    provenance: ElementProvenance,
    #[serde(default)]
    external_references: Vec<ExternalReference>,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MoveDiagramNodeArgs {
    diagram_id: String,
    model_ref: String,
    x: f64,
    y: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResizeDiagramNodeArgs {
    diagram_id: String,
    model_ref: String,
    width: f64,
    height: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AlignDiagramNodesArgs {
    diagram_id: String,
    model_refs: Vec<String>,
    alignment: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DistributeDiagramNodesArgs {
    diagram_id: String,
    model_refs: Vec<String>,
    axis: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConnectDiagramRelationshipArgs {
    diagram_id: String,
    relationship_ref: String,
    #[serde(default)]
    route_hint: Option<DiagramRouteHint>,
    #[serde(default)]
    label_position: Option<DiagramPoint>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RouteDiagramConnectorArgs {
    diagram_id: String,
    relationship_ref: String,
    route_hint: DiagramRouteHint,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleDiagramObjectArgs {
    diagram_id: String,
    object_kind: String,
    object_ref: String,
    style: DiagramStyle,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplyDiagramAutoLayoutArgs {
    diagram_id: String,
    layout_engine: String,
    nodes: Vec<DiagramNodeLayout>,
    #[serde(default)]
    connectors: Vec<DiagramConnectorLayout>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpsertRenderRuleArgs {
    profile_id: String,
    rule: RenderRule,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveRenderRuleArgs {
    profile_id: String,
    rule_id: String,
}

pub fn load_package(root: impl AsRef<Path>) -> Result<ModelPackage> {
    let root = root.as_ref().to_path_buf();
    Ok(ModelPackage {
        manifest: read_json(root.join("manifest.json"))?,
        requirements: read_json(root.join("requirements/requirements.json"))?,
        elements: read_json(root.join("model/elements.json"))?,
        relationships: read_json(root.join("model/relationships.json"))?,
        diagrams: read_json(root.join("views/diagrams.json"))?,
        render_profiles: read_json(root.join("views/render-profile.json"))?,
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
        diagram_layout_operations_applied: 0,
        render_profile_operations_applied: 0,
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
                    aliases: args.aliases,
                    description: args.description,
                    documentation: args.documentation,
                    status: args.status,
                    stereotypes: args.stereotypes,
                    tags: args.tags,
                    provenance: args.provenance,
                    external_references: args.external_references,
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
            "move_diagram_node" => {
                let args: MoveDiagramNodeArgs = parse_args(operation)?;
                move_diagram_node(package, args)?;
                summary.diagram_layout_operations_applied += 1;
            }
            "resize_diagram_node" => {
                let args: ResizeDiagramNodeArgs = parse_args(operation)?;
                resize_diagram_node(package, args)?;
                summary.diagram_layout_operations_applied += 1;
            }
            "align_diagram_nodes" => {
                let args: AlignDiagramNodesArgs = parse_args(operation)?;
                align_diagram_nodes(package, args)?;
                summary.diagram_layout_operations_applied += 1;
            }
            "distribute_diagram_nodes" => {
                let args: DistributeDiagramNodesArgs = parse_args(operation)?;
                distribute_diagram_nodes(package, args)?;
                summary.diagram_layout_operations_applied += 1;
            }
            "connect_diagram_relationship" => {
                let args: ConnectDiagramRelationshipArgs = parse_args(operation)?;
                connect_diagram_relationship(package, args)?;
                summary.diagram_layout_operations_applied += 1;
            }
            "route_diagram_connector" => {
                let args: RouteDiagramConnectorArgs = parse_args(operation)?;
                route_diagram_connector(package, args)?;
                summary.diagram_layout_operations_applied += 1;
            }
            "style_diagram_object" => {
                let args: StyleDiagramObjectArgs = parse_args(operation)?;
                style_diagram_object(package, args)?;
                summary.diagram_layout_operations_applied += 1;
            }
            "apply_diagram_auto_layout" => {
                let args: ApplyDiagramAutoLayoutArgs = parse_args(operation)?;
                apply_diagram_auto_layout(package, args)?;
                summary.diagram_layout_operations_applied += 1;
            }
            "upsert_render_rule" => {
                let args: UpsertRenderRuleArgs = parse_args(operation)?;
                upsert_render_rule(package, args)?;
                summary.render_profile_operations_applied += 1;
            }
            "remove_render_rule" => {
                let args: RemoveRenderRuleArgs = parse_args(operation)?;
                remove_render_rule(package, args)?;
                summary.render_profile_operations_applied += 1;
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
    require_version("render profiles", &package.render_profiles.schema_version)?;
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
        for alias in &element.aliases {
            ensure_non_empty(alias, &format!("{} alias", element.id))?;
        }
        if !matches!(
            element.status.as_str(),
            "draft" | "proposed" | "accepted" | "deprecated" | "retired"
        ) {
            bail!(
                "{} has unsupported element status {}",
                element.id,
                element.status
            );
        }
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
        for stereotype in &element.stereotypes {
            ensure_non_empty(stereotype, &format!("{} stereotype", element.id))?;
        }
        for tag in &element.tags {
            ensure_non_empty(tag, &format!("{} tag", element.id))?;
        }
        validate_element_provenance(element)?;
        validate_external_references(element)?;
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
    let relationships_by_id: BTreeMap<&str, &Relationship> = package
        .relationships
        .relationships
        .iter()
        .map(|relationship| (relationship.id.as_str(), relationship))
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
            validate_diagram_layout(diagram, layout, &element_kinds, &relationships_by_id)?;
        }
    }

    validate_render_profiles(&package.render_profiles, &element_kinds)?;

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

fn validate_element_provenance(element: &ModelElement) -> Result<()> {
    for source_ref in &element.provenance.source_refs {
        ensure_non_empty(source_ref, &format!("{} provenance sourceRef", element.id))?;
    }
    if let Some(created_by) = &element.provenance.created_by {
        ensure_non_empty(created_by, &format!("{} provenance createdBy", element.id))?;
    }
    if let Some(created_at) = &element.provenance.created_at {
        ensure_non_empty(created_at, &format!("{} provenance createdAt", element.id))?;
    }
    Ok(())
}

fn validate_external_references(element: &ModelElement) -> Result<()> {
    let mut refs = BTreeSet::new();
    for reference in &element.external_references {
        ensure_unique(&mut refs, &reference.id)?;
        ensure_non_empty(
            &reference.label,
            &format!("{} external reference {} label", element.id, reference.id),
        )?;
        ensure_non_empty(
            &reference.uri,
            &format!("{} external reference {} uri", element.id, reference.id),
        )?;
    }
    Ok(())
}

fn validate_diagram_layout(
    diagram: &DiagramView,
    layout: &DiagramLayout,
    element_kinds: &BTreeMap<&str, &str>,
    relationships_by_id: &BTreeMap<&str, &Relationship>,
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
        if let Some(style) = &node.style {
            validate_diagram_style(&diagram.id, &node.model_ref, style)?;
        }
    }

    let mut connector_refs = BTreeSet::new();
    for connector in &layout.connectors {
        ensure_unique(&mut connector_refs, &connector.relationship_ref)?;
        let Some(relationship) = relationships_by_id.get(connector.relationship_ref.as_str())
        else {
            bail!(
                "{} layout references missing relationship {}",
                diagram.id,
                connector.relationship_ref
            );
        };
        if !diagram_refs.contains(relationship.source_id.as_str())
            || !diagram_refs.contains(relationship.target_id.as_str())
        {
            bail!(
                "{} connector {} references relationship endpoints outside the diagram view",
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
        if let Some(style) = &connector.style {
            validate_diagram_style(&diagram.id, &connector.relationship_ref, style)?;
        }
    }

    Ok(())
}

fn validate_diagram_style(diagram_id: &str, object_ref: &str, style: &DiagramStyle) -> Result<()> {
    for color in [
        style.fill_color.as_deref(),
        style.stroke_color.as_deref(),
        style.text_color.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if !is_hex_color(color) {
            bail!("{diagram_id} layout object {object_ref} has invalid color {color}");
        }
    }
    if let Some(line_style) = &style.line_style {
        if !matches!(line_style.as_str(), "solid" | "dashed" | "dotted") {
            bail!(
                "{diagram_id} layout object {object_ref} has unsupported line style {line_style}"
            );
        }
    }
    Ok(())
}

fn validate_render_profiles(
    render_profiles: &RenderProfileFile,
    element_kinds: &BTreeMap<&str, &str>,
) -> Result<()> {
    let mut profile_ids = BTreeSet::new();
    for profile in &render_profiles.profiles {
        ensure_unique(&mut profile_ids, &profile.id)?;
        ensure_non_empty(&profile.title, &format!("{} title", profile.id))?;
        let asset_ids: BTreeSet<&str> = profile
            .assets
            .iter()
            .map(|asset| asset.id.as_str())
            .collect();
        validate_render_target(&profile.id, "fallback", &profile.fallback, &asset_ids)?;

        let mut rule_ids = BTreeSet::new();
        for rule in &profile.rules {
            ensure_unique(&mut rule_ids, &rule.id)?;
            validate_render_selector(&profile.id, &rule.id, &rule.selector, element_kinds)?;
            validate_render_target(&profile.id, &rule.id, &rule.render_as, &asset_ids)?;
        }

        let mut checked_asset_ids = BTreeSet::new();
        for asset in &profile.assets {
            ensure_unique(&mut checked_asset_ids, &asset.id)?;
            validate_render_asset(&profile.id, asset)?;
        }
    }
    Ok(())
}

fn validate_render_selector(
    profile_id: &str,
    rule_id: &str,
    selector: &RenderSelector,
    element_kinds: &BTreeMap<&str, &str>,
) -> Result<()> {
    let populated = [
        selector.element_id.as_deref(),
        selector.element_kind.as_deref(),
        selector.stereotype.as_deref(),
        selector.tag.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter(|value| !value.trim().is_empty())
    .count();
    if populated == 0 {
        bail!("{profile_id} rule {rule_id} selector must match at least one field");
    }
    if let Some(element_id) = &selector.element_id {
        if !element_kinds.contains_key(element_id.as_str()) {
            bail!("{profile_id} rule {rule_id} references missing element {element_id}");
        }
    }
    if let Some(element_kind) = &selector.element_kind {
        validate_element_kind(profile_id, rule_id, element_kind)?;
    }
    Ok(())
}

fn validate_render_target(
    profile_id: &str,
    rule_id: &str,
    render_target: &RenderTarget,
    asset_ids: &BTreeSet<&str>,
) -> Result<()> {
    if !matches!(
        render_target.renderer_id.as_str(),
        "uml.actor"
            | "uml.use_case"
            | "uml.class"
            | "uml.component"
            | "uml.activity"
            | "uml.sequence_participant"
            | "image.element"
            | "html.custom"
    ) {
        bail!(
            "{profile_id} rule {rule_id} has unsupported renderer {}",
            render_target.renderer_id
        );
    }
    if render_target.renderer_id == "image.element" {
        let Some(asset_ref) = &render_target.asset_ref else {
            bail!("{profile_id} rule {rule_id} image renderer requires assetRef");
        };
        if !asset_ids.contains(asset_ref.as_str()) {
            bail!("{profile_id} rule {rule_id} references missing asset {asset_ref}");
        }
    }
    if let Some(style) = &render_target.style {
        validate_diagram_style(profile_id, rule_id, style)?;
    }
    for port in &render_target.ports {
        ensure_non_empty(&port.id, "render port id")?;
        if !matches!(port.side.as_str(), "top" | "right" | "bottom" | "left") {
            bail!(
                "{profile_id} rule {rule_id} has unsupported port side {}",
                port.side
            );
        }
        if let Some(offset) = port.offset {
            if !(0.0..=1.0).contains(&offset) {
                bail!(
                    "{profile_id} rule {rule_id} port {} has invalid offset {offset}",
                    port.id
                );
            }
        }
    }
    if let Some(label) = &render_target.label {
        if let Some(position) = &label.position {
            if !matches!(
                position.as_str(),
                "inside" | "top" | "right" | "bottom" | "left"
            ) {
                bail!("{profile_id} rule {rule_id} has unsupported label position {position}");
            }
        }
    }
    Ok(())
}

fn validate_render_asset(profile_id: &str, asset: &RenderAsset) -> Result<()> {
    ensure_non_empty(
        &asset.uri,
        &format!("{} asset {} uri", profile_id, asset.id),
    )?;
    if !asset.uri.starts_with("assets/render/")
        || !(asset.uri.ends_with(".png")
            || asset.uri.ends_with(".jpg")
            || asset.uri.ends_with(".jpeg")
            || asset.uri.ends_with(".svg"))
        || asset.uri.contains("..")
    {
        bail!(
            "{profile_id} asset {} uses unsupported uri {}",
            asset.id,
            asset.uri
        );
    }
    if !matches!(
        asset.kind.as_str(),
        "image/png" | "image/jpeg" | "image/svg+xml"
    ) {
        bail!(
            "{profile_id} asset {} has unsupported kind {}",
            asset.id,
            asset.kind
        );
    }
    if !matches!(
        asset.status.as_str(),
        "referenced" | "available" | "missing" | "blocked"
    ) {
        bail!(
            "{profile_id} asset {} has unsupported status {}",
            asset.id,
            asset.status
        );
    }
    if asset.status == "available" && asset.content_sha256.is_none() {
        bail!(
            "{profile_id} asset {} is available without contentSha256",
            asset.id
        );
    }
    ensure_non_empty(
        &asset.provenance.source_type,
        &format!("{} asset {} sourceType", profile_id, asset.id),
    )?;
    ensure_non_empty(
        &asset.provenance.source,
        &format!("{} asset {} source", profile_id, asset.id),
    )?;
    ensure_non_empty(
        &asset.provenance.license,
        &format!("{} asset {} license", profile_id, asset.id),
    )?;
    Ok(())
}

fn validate_element_kind(profile_id: &str, rule_id: &str, element_kind: &str) -> Result<()> {
    if !matches!(
        element_kind,
        "actor" | "use_case" | "class" | "component" | "activity" | "sequence_participant"
    ) {
        bail!("{profile_id} rule {rule_id} has unsupported selector kind {element_kind}");
    }
    Ok(())
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value
            .chars()
            .skip(1)
            .all(|character| character.is_ascii_hexdigit())
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

fn move_diagram_node(package: &mut ModelPackage, args: MoveDiagramNodeArgs) -> Result<()> {
    let diagram = find_diagram_mut(package, &args.diagram_id)?;
    let node = upsert_node_layout(diagram, &args.model_ref)?;
    node.bounds.x = args.x;
    node.bounds.y = args.y;
    node.layout_state = "manual".to_string();
    mark_layout_manual(diagram)?;
    Ok(())
}

fn resize_diagram_node(package: &mut ModelPackage, args: ResizeDiagramNodeArgs) -> Result<()> {
    if args.width <= 0.0 || args.height <= 0.0 {
        bail!(
            "{} layout node {} must have positive bounds",
            args.diagram_id,
            args.model_ref
        );
    }
    let diagram = find_diagram_mut(package, &args.diagram_id)?;
    let node = upsert_node_layout(diagram, &args.model_ref)?;
    node.bounds.width = args.width;
    node.bounds.height = args.height;
    node.layout_state = "manual".to_string();
    mark_layout_manual(diagram)?;
    Ok(())
}

fn align_diagram_nodes(package: &mut ModelPackage, args: AlignDiagramNodesArgs) -> Result<()> {
    if args.model_refs.len() < 2 {
        bail!("align_diagram_nodes requires at least two modelRefs");
    }
    let diagram = find_diagram_mut(package, &args.diagram_id)?;
    ensure_node_layouts(diagram, &args.model_refs)?;
    let bounds = node_bounds(diagram, &args.model_refs)?;
    let target = match args.alignment.as_str() {
        "left" => bounds
            .iter()
            .map(|bounds| bounds.x)
            .fold(f64::INFINITY, f64::min),
        "right" => bounds
            .iter()
            .map(|bounds| bounds.x + bounds.width)
            .fold(f64::NEG_INFINITY, f64::max),
        "top" => bounds
            .iter()
            .map(|bounds| bounds.y)
            .fold(f64::INFINITY, f64::min),
        "bottom" => bounds
            .iter()
            .map(|bounds| bounds.y + bounds.height)
            .fold(f64::NEG_INFINITY, f64::max),
        "hcenter" => {
            let min = bounds
                .iter()
                .map(|bounds| bounds.x)
                .fold(f64::INFINITY, f64::min);
            let max = bounds
                .iter()
                .map(|bounds| bounds.x + bounds.width)
                .fold(f64::NEG_INFINITY, f64::max);
            (min + max) / 2.0
        }
        "vcenter" => {
            let min = bounds
                .iter()
                .map(|bounds| bounds.y)
                .fold(f64::INFINITY, f64::min);
            let max = bounds
                .iter()
                .map(|bounds| bounds.y + bounds.height)
                .fold(f64::NEG_INFINITY, f64::max);
            (min + max) / 2.0
        }
        other => bail!("unsupported diagram node alignment {other}"),
    };

    for model_ref in &args.model_refs {
        let node = find_node_layout_mut(diagram, model_ref)?;
        match args.alignment.as_str() {
            "left" => node.bounds.x = target,
            "right" => node.bounds.x = target - node.bounds.width,
            "top" => node.bounds.y = target,
            "bottom" => node.bounds.y = target - node.bounds.height,
            "hcenter" => node.bounds.x = target - node.bounds.width / 2.0,
            "vcenter" => node.bounds.y = target - node.bounds.height / 2.0,
            _ => unreachable!(),
        }
        node.layout_state = "manual".to_string();
    }
    mark_layout_manual(diagram)?;
    Ok(())
}

fn distribute_diagram_nodes(
    package: &mut ModelPackage,
    args: DistributeDiagramNodesArgs,
) -> Result<()> {
    if args.model_refs.len() < 3 {
        bail!("distribute_diagram_nodes requires at least three modelRefs");
    }
    if !matches!(args.axis.as_str(), "x" | "y") {
        bail!("unsupported diagram distribution axis {}", args.axis);
    }
    let diagram = find_diagram_mut(package, &args.diagram_id)?;
    ensure_node_layouts(diagram, &args.model_refs)?;
    let mut ordered: Vec<(String, f64)> = args
        .model_refs
        .iter()
        .map(|model_ref| {
            let node = find_node_layout(diagram, model_ref)?;
            let value = if args.axis == "x" {
                node.bounds.x
            } else {
                node.bounds.y
            };
            Ok((model_ref.clone(), value))
        })
        .collect::<Result<_>>()?;
    ordered.sort_by(|left, right| left.1.total_cmp(&right.1));
    let first = ordered.first().map(|(_, value)| *value).unwrap_or(0.0);
    let last = ordered.last().map(|(_, value)| *value).unwrap_or(first);
    let step = (last - first) / (ordered.len() - 1) as f64;

    for (index, (model_ref, _)) in ordered.iter().enumerate() {
        let node = find_node_layout_mut(diagram, model_ref)?;
        if args.axis == "x" {
            node.bounds.x = first + index as f64 * step;
        } else {
            node.bounds.y = first + index as f64 * step;
        }
        node.layout_state = "manual".to_string();
    }
    mark_layout_manual(diagram)?;
    Ok(())
}

fn connect_diagram_relationship(
    package: &mut ModelPackage,
    args: ConnectDiagramRelationshipArgs,
) -> Result<()> {
    ensure_relationship_exists(package, &args.relationship_ref)?;
    let diagram = find_diagram_mut(package, &args.diagram_id)?;
    let connector = upsert_connector_layout(diagram, &args.relationship_ref)?;
    connector.route_hint = args.route_hint;
    connector.label_position = args.label_position;
    connector.layout_state = "manual".to_string();
    mark_layout_manual(diagram)?;
    Ok(())
}

fn route_diagram_connector(
    package: &mut ModelPackage,
    args: RouteDiagramConnectorArgs,
) -> Result<()> {
    ensure_relationship_exists(package, &args.relationship_ref)?;
    let diagram = find_diagram_mut(package, &args.diagram_id)?;
    let connector = upsert_connector_layout(diagram, &args.relationship_ref)?;
    connector.route_hint = Some(args.route_hint);
    connector.layout_state = "manual".to_string();
    mark_layout_manual(diagram)?;
    Ok(())
}

fn style_diagram_object(package: &mut ModelPackage, args: StyleDiagramObjectArgs) -> Result<()> {
    match args.object_kind.as_str() {
        "node" => {
            let diagram = find_diagram_mut(package, &args.diagram_id)?;
            let node = upsert_node_layout(diagram, &args.object_ref)?;
            node.style = Some(args.style);
            node.layout_state = "manual".to_string();
        }
        "connector" => {
            ensure_relationship_exists(package, &args.object_ref)?;
            let diagram = find_diagram_mut(package, &args.diagram_id)?;
            let connector = upsert_connector_layout(diagram, &args.object_ref)?;
            connector.style = Some(args.style);
            connector.layout_state = "manual".to_string();
        }
        other => bail!("unsupported diagram style object kind {other}"),
    }
    let diagram = find_diagram_mut(package, &args.diagram_id)?;
    mark_layout_manual(diagram)?;
    Ok(())
}

fn apply_diagram_auto_layout(
    package: &mut ModelPackage,
    args: ApplyDiagramAutoLayoutArgs,
) -> Result<()> {
    let diagram = find_diagram_mut(package, &args.diagram_id)?;
    let layout = ensure_layout_mut(diagram);
    layout.coordinate_system = "canvas".to_string();
    layout.layout_engine = args.layout_engine;
    layout.layout_state = "generated".to_string();
    layout.nodes = args.nodes;
    if !args.connectors.is_empty() {
        layout.connectors = args.connectors;
    }
    for node in &mut layout.nodes {
        node.layout_state = "generated".to_string();
    }
    for connector in &mut layout.connectors {
        connector.layout_state = "generated".to_string();
    }
    Ok(())
}

fn upsert_render_rule(package: &mut ModelPackage, args: UpsertRenderRuleArgs) -> Result<()> {
    let profile = find_render_profile_mut(package, &args.profile_id)?;
    if let Some(index) = profile
        .rules
        .iter()
        .position(|rule| rule.id == args.rule.id)
    {
        profile.rules[index] = args.rule;
    } else {
        profile.rules.push(args.rule);
    }
    Ok(())
}

fn remove_render_rule(package: &mut ModelPackage, args: RemoveRenderRuleArgs) -> Result<()> {
    let profile = find_render_profile_mut(package, &args.profile_id)?;
    let before = profile.rules.len();
    profile.rules.retain(|rule| rule.id != args.rule_id);
    if profile.rules.len() == before {
        bail!(
            "{} is missing render rule {}",
            args.profile_id,
            args.rule_id
        );
    }
    Ok(())
}

fn find_render_profile_mut<'a>(
    package: &'a mut ModelPackage,
    profile_id: &str,
) -> Result<&'a mut RenderProfile> {
    package
        .render_profiles
        .profiles
        .iter_mut()
        .find(|profile| profile.id == profile_id)
        .ok_or_else(|| anyhow!("missing render profile {profile_id}"))
}

fn find_diagram_mut<'a>(
    package: &'a mut ModelPackage,
    diagram_id: &str,
) -> Result<&'a mut DiagramView> {
    package
        .diagrams
        .diagrams
        .iter_mut()
        .find(|diagram| diagram.id == diagram_id)
        .ok_or_else(|| anyhow!("missing diagram {diagram_id}"))
}

fn ensure_layout_mut(diagram: &mut DiagramView) -> &mut DiagramLayout {
    diagram.layout.get_or_insert_with(|| DiagramLayout {
        coordinate_system: "canvas".to_string(),
        layout_engine: String::new(),
        layout_state: "manual".to_string(),
        nodes: Vec::new(),
        connectors: Vec::new(),
    })
}

fn upsert_node_layout<'a>(
    diagram: &'a mut DiagramView,
    model_ref: &str,
) -> Result<&'a mut DiagramNodeLayout> {
    if !diagram
        .model_refs
        .iter()
        .any(|reference| reference == model_ref)
    {
        bail!("{} does not include modelRef {model_ref}", diagram.id);
    }
    let layout = ensure_layout_mut(diagram);
    if let Some(index) = layout
        .nodes
        .iter()
        .position(|node| node.model_ref == model_ref)
    {
        return Ok(&mut layout.nodes[index]);
    }
    layout.nodes.push(DiagramNodeLayout {
        model_ref: model_ref.to_string(),
        bounds: default_node_bounds(),
        layout_state: "manual".to_string(),
        label_position: None,
        style: None,
    });
    Ok(layout
        .nodes
        .last_mut()
        .expect("node layout was just inserted"))
}

fn upsert_connector_layout<'a>(
    diagram: &'a mut DiagramView,
    relationship_ref: &str,
) -> Result<&'a mut DiagramConnectorLayout> {
    let layout = ensure_layout_mut(diagram);
    if let Some(index) = layout
        .connectors
        .iter()
        .position(|connector| connector.relationship_ref == relationship_ref)
    {
        return Ok(&mut layout.connectors[index]);
    }
    layout.connectors.push(DiagramConnectorLayout {
        relationship_ref: relationship_ref.to_string(),
        layout_state: "manual".to_string(),
        route_hint: None,
        label_position: None,
        style: None,
    });
    Ok(layout
        .connectors
        .last_mut()
        .expect("connector layout was just inserted"))
}

fn ensure_node_layouts(diagram: &mut DiagramView, model_refs: &[String]) -> Result<()> {
    for model_ref in model_refs {
        upsert_node_layout(diagram, model_ref)?;
    }
    Ok(())
}

fn find_node_layout<'a>(
    diagram: &'a DiagramView,
    model_ref: &str,
) -> Result<&'a DiagramNodeLayout> {
    diagram
        .layout
        .as_ref()
        .and_then(|layout| layout.nodes.iter().find(|node| node.model_ref == model_ref))
        .ok_or_else(|| anyhow!("{} is missing layout node {model_ref}", diagram.id))
}

fn find_node_layout_mut<'a>(
    diagram: &'a mut DiagramView,
    model_ref: &str,
) -> Result<&'a mut DiagramNodeLayout> {
    let diagram_id = diagram.id.clone();
    diagram
        .layout
        .as_mut()
        .and_then(|layout| {
            layout
                .nodes
                .iter_mut()
                .find(|node| node.model_ref == model_ref)
        })
        .ok_or_else(|| anyhow!("{diagram_id} is missing layout node {model_ref}"))
}

fn node_bounds(diagram: &DiagramView, model_refs: &[String]) -> Result<Vec<DiagramBounds>> {
    model_refs
        .iter()
        .map(|model_ref| Ok(find_node_layout(diagram, model_ref)?.bounds.clone()))
        .collect()
}

fn mark_layout_manual(diagram: &mut DiagramView) -> Result<()> {
    let layout = diagram
        .layout
        .as_mut()
        .ok_or_else(|| anyhow!("{} is missing layout", diagram.id))?;
    layout.coordinate_system = "canvas".to_string();
    layout.layout_state = "mixed".to_string();
    Ok(())
}

fn ensure_relationship_exists(package: &ModelPackage, relationship_ref: &str) -> Result<()> {
    if package
        .relationships
        .relationships
        .iter()
        .any(|relationship| relationship.id == relationship_ref)
    {
        Ok(())
    } else {
        bail!("missing relationship {relationship_ref}")
    }
}

fn default_node_bounds() -> DiagramBounds {
    DiagramBounds {
        x: 0.0,
        y: 0.0,
        width: 210.0,
        height: 86.0,
    }
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
            "move_diagram_node" => {
                require_args(&operation.args, &["diagramId", "modelRef", "x", "y"])?
            }
            "resize_diagram_node" => require_args(
                &operation.args,
                &["diagramId", "modelRef", "width", "height"],
            )?,
            "align_diagram_nodes" => {
                require_args(&operation.args, &["diagramId", "modelRefs", "alignment"])?
            }
            "distribute_diagram_nodes" => {
                require_args(&operation.args, &["diagramId", "modelRefs", "axis"])?
            }
            "connect_diagram_relationship" => {
                require_args(&operation.args, &["diagramId", "relationshipRef"])?
            }
            "route_diagram_connector" => require_args(
                &operation.args,
                &["diagramId", "relationshipRef", "routeHint"],
            )?,
            "style_diagram_object" => require_args(
                &operation.args,
                &["diagramId", "objectKind", "objectRef", "style"],
            )?,
            "apply_diagram_auto_layout" => {
                require_args(&operation.args, &["diagramId", "layoutEngine", "nodes"])?
            }
            "upsert_render_rule" => require_args(&operation.args, &["profileId", "rule"])?,
            "remove_render_rule" => require_args(&operation.args, &["profileId", "ruleId"])?,
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
    write_json(
        package.root.join("views/render-profile.json"),
        &package.render_profiles,
    )?;
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
    for profile in &mut package.render_profiles.profiles {
        profile.rules.sort_by(|left, right| left.id.cmp(&right.id));
        profile.assets.sort_by(|left, right| left.id.cmp(&right.id));
    }
    package
        .render_profiles
        .profiles
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

fn default_element_status() -> String {
    "accepted".to_string()
}

fn is_default_element_status(status: &str) -> bool {
    status == "accepted"
}

fn default_enabled() -> bool {
    true
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
        assert_eq!(layout.nodes.len(), 5);
        assert_eq!(layout.connectors.len(), 3);
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
        "name": "Export SVG",
        "aliases": ["SVG export"],
        "description": "Export an accepted diagram view as SVG.",
        "documentation": "The exported SVG remains a delivery artifact; canonical model truth stays in the model package.",
        "status": "proposed",
        "provenance": {
          "sourceRefs": ["source.roadmap"],
          "createdBy": "test",
          "createdAt": "2026-07-20T15:30:00Z"
        },
        "externalReferences": [
          {
            "id": "ref.svg",
            "label": "SVG export note",
            "uri": "docs/MODEL_PACKAGE.md#thin-cli",
            "kind": "document"
          }
        ]
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
        let exported = package
            .elements
            .elements
            .iter()
            .find(|element| element.id == "usecase.export-svg")
            .unwrap();
        assert_eq!(exported.status, "proposed");
        assert_eq!(exported.aliases, vec!["SVG export"]);
        assert_eq!(exported.provenance.source_refs, vec!["source.roadmap"]);
        assert_eq!(exported.external_references[0].id, "ref.svg");
        let applied = fs::read_to_string(summary.applied_proposal_path).unwrap();
        assert!(applied.contains(r#""state": "applied""#));
    }

    #[test]
    fn accepted_proposal_applies_diagram_layout_operations() {
        let root = copy_example_to_temp();
        let proposal_path = root.join("proposals/open/accepted-layout-ops.json");
        fs::write(
            &proposal_path,
            r##"{
  "proposalId": "proposal.layout-ops",
  "schemaVersion": "0.1.0",
  "state": "accepted",
  "createdAt": "2026-07-19T23:40:00Z",
  "intent": "Apply direct manipulation layout changes.",
  "operations": [
    {
      "opId": "op.move-actor",
      "op": "move_diagram_node",
      "args": {
        "diagramId": "diagram.first-use-case",
        "modelRef": "actor.architect",
        "x": 120,
        "y": 144
      },
      "rationale": "The actor was manually positioned on the canvas."
    },
    {
      "opId": "op.resize-actor",
      "op": "resize_diagram_node",
      "args": {
        "diagramId": "diagram.first-use-case",
        "modelRef": "actor.architect",
        "width": 240,
        "height": 100
      },
      "rationale": "The actor node was resized to fit the label."
    },
    {
      "opId": "op.align-top",
      "op": "align_diagram_nodes",
      "args": {
        "diagramId": "diagram.first-use-case",
        "modelRefs": ["actor.architect", "usecase.review-proposal"],
        "alignment": "top"
      },
      "rationale": "The actor and proposal use case were aligned."
    },
    {
      "opId": "op.distribute-y",
      "op": "distribute_diagram_nodes",
      "args": {
        "diagramId": "diagram.first-use-case",
        "modelRefs": ["actor.architect", "usecase.review-proposal", "usecase.render-diagram"],
        "axis": "y"
      },
      "rationale": "The visible use-case objects were distributed vertically."
    },
    {
      "opId": "op.connect-review",
      "op": "connect_diagram_relationship",
      "args": {
        "diagramId": "diagram.first-use-case",
        "relationshipRef": "rel.architect-review",
        "labelPosition": { "x": 360, "y": 128 }
      },
      "rationale": "The existing semantic relationship was made visible in the diagram view."
    },
    {
      "opId": "op.route-render",
      "op": "route_diagram_connector",
      "args": {
        "diagramId": "diagram.first-use-case",
        "relationshipRef": "rel.architect-render",
        "routeHint": {
          "kind": "orthogonal",
          "points": [
            { "x": 300, "y": 194 },
            { "x": 420, "y": 244 }
          ]
        }
      },
      "rationale": "The render connector was manually routed."
    },
    {
      "opId": "op.style-actor",
      "op": "style_diagram_object",
      "args": {
        "diagramId": "diagram.first-use-case",
        "objectKind": "node",
        "objectRef": "actor.architect",
        "style": {
          "fillColor": "#ffffff",
          "strokeColor": "#0f766e",
          "textColor": "#134e4a"
        }
      },
      "rationale": "The actor node was styled as a manual view concern."
    },
    {
      "opId": "op.style-render-connector",
      "op": "style_diagram_object",
      "args": {
        "diagramId": "diagram.first-use-case",
        "objectKind": "connector",
        "objectRef": "rel.architect-render",
        "style": {
          "strokeColor": "#475569",
          "lineStyle": "dashed"
        }
      },
      "rationale": "The render connector was styled as a manual view concern."
    }
  ]
}
"##,
        )
        .unwrap();

        let summary = apply_accepted_proposal_file(&root, &proposal_path).unwrap();
        assert_eq!(summary.diagram_layout_operations_applied, 8);

        let package = load_package(&root).unwrap();
        validate_package(&package).unwrap();
        let layout = package.diagrams.diagrams[0].layout.as_ref().unwrap();
        assert_eq!(layout.layout_state, "mixed");
        let actor = layout
            .nodes
            .iter()
            .find(|node| node.model_ref == "actor.architect")
            .unwrap();
        assert_eq!(actor.bounds.width, 240.0);
        assert_eq!(actor.layout_state, "manual");
        assert_eq!(
            actor.style.as_ref().unwrap().fill_color.as_deref(),
            Some("#ffffff")
        );
        let connector = layout
            .connectors
            .iter()
            .find(|connector| connector.relationship_ref == "rel.architect-render")
            .unwrap();
        assert_eq!(connector.route_hint.as_ref().unwrap().kind, "orthogonal");
        assert_eq!(
            connector.style.as_ref().unwrap().line_style.as_deref(),
            Some("dashed")
        );
    }

    #[test]
    fn accepted_proposal_applies_generated_auto_layout() {
        let root = copy_example_to_temp();
        let proposal_path = root.join("proposals/open/accepted-auto-layout.json");
        fs::write(
            &proposal_path,
            r#"{
  "proposalId": "proposal.auto-layout",
  "schemaVersion": "0.1.0",
  "state": "accepted",
  "createdAt": "2026-07-19T23:42:00Z",
  "intent": "Replace view metadata with a generated layout.",
  "operations": [
    {
      "opId": "op.apply-elk",
      "op": "apply_diagram_auto_layout",
      "args": {
        "diagramId": "diagram.first-use-case",
        "layoutEngine": "elk.layered",
        "nodes": [
          {
            "modelRef": "actor.architect",
            "bounds": { "x": 10, "y": 20, "width": 210, "height": 86 },
            "layoutState": "manual"
          },
          {
            "modelRef": "usecase.review-proposal",
            "bounds": { "x": 320, "y": 20, "width": 210, "height": 86 },
            "layoutState": "manual"
          },
          {
            "modelRef": "usecase.render-diagram",
            "bounds": { "x": 320, "y": 150, "width": 210, "height": 86 },
            "layoutState": "manual"
          }
        ]
      },
      "rationale": "The layout engine generated fresh canvas bounds."
    }
  ]
}
"#,
        )
        .unwrap();

        let summary = apply_accepted_proposal_file(&root, &proposal_path).unwrap();
        assert_eq!(summary.diagram_layout_operations_applied, 1);
        let package = load_package(&root).unwrap();
        let layout = package.diagrams.diagrams[0].layout.as_ref().unwrap();
        assert_eq!(layout.layout_engine, "elk.layered");
        assert_eq!(layout.layout_state, "generated");
        assert!(
            layout
                .nodes
                .iter()
                .all(|node| node.layout_state == "generated")
        );
    }

    #[test]
    fn accepted_proposal_applies_render_profile_operations() {
        let root = copy_example_to_temp();
        let proposal_path = root.join("proposals/open/accepted-render-profile-ops.json");
        fs::write(
            &proposal_path,
            r##"{
  "proposalId": "proposal.render-profile-ops",
  "schemaVersion": "0.1.0",
  "state": "accepted",
  "createdAt": "2026-07-20T14:25:00Z",
  "intent": "Persist workbench render profile edits.",
  "operations": [
    {
      "opId": "op.upsert-review-rule",
      "op": "upsert_render_rule",
      "args": {
        "profileId": "render-profile.default",
        "rule": {
          "id": "render.ui.elementId.usecase.review-proposal",
          "description": "Render the review use case with a workbench-authored style.",
          "selector": {
            "elementId": "usecase.review-proposal"
          },
          "renderAs": {
            "rendererId": "uml.use_case",
            "style": {
              "fillColor": "#e0f2fe",
              "strokeColor": "#0369a1",
              "textColor": "#0c4a6e"
            },
            "ports": [
              {
                "id": "in",
                "side": "left",
                "offset": 0.5
              }
            ],
            "label": {
              "visible": true,
              "position": "inside"
            }
          },
          "precedence": 250,
          "enabled": true
        }
      },
      "rationale": "The workbench render-rule editor changed renderer metadata."
    },
    {
      "opId": "op.remove-duck-rule",
      "op": "remove_render_rule",
      "args": {
        "profileId": "render-profile.default",
        "ruleId": "render.stereotype-duck"
      },
      "rationale": "The workbench render-rule editor removed an example rule."
    }
  ]
}
"##,
        )
        .unwrap();

        let summary = apply_accepted_proposal_file(&root, &proposal_path).unwrap();
        assert_eq!(summary.render_profile_operations_applied, 2);

        let package = load_package(&root).unwrap();
        validate_package(&package).unwrap();
        let profile = &package.render_profiles.profiles[0];
        assert!(
            profile
                .rules
                .iter()
                .any(|rule| rule.id == "render.ui.elementId.usecase.review-proposal")
        );
        assert!(
            !profile
                .rules
                .iter()
                .any(|rule| rule.id == "render.stereotype-duck")
        );
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
