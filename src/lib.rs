use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const PORTFOLIO_KINDS: &[&str] = &[
    "business_capability",
    "portfolio_application",
    "portfolio_service",
    "technology_component",
    "technology_standard",
    "organization_unit",
    "owner",
    "lifecycle_milestone",
    "roadmap_item",
    "risk",
    "control",
    "governance_decision",
    "data_source",
];
const PORTFOLIO_STATUSES: &[&str] = &["draft", "proposed", "accepted", "deprecated", "retired"];
const LIFECYCLE_STATES: &[&str] = &[
    "idea",
    "planned",
    "active",
    "deprecated",
    "retiring",
    "retired",
];
const LIFECYCLE_TARGET_STATES: &[&str] =
    &["planned", "active", "deprecated", "retiring", "retired"];
const CRITICALITIES: &[&str] = &["low", "medium", "high", "critical"];
const STANDARD_STATES: &[&str] = &["approved", "tolerated", "discouraged", "banned", "emerging"];
const SAVED_VIEW_SORT_FIELDS: &[&str] = &[
    "name",
    "kind",
    "status",
    "lifecycleState",
    "criticality",
    "standardState",
];
const SAVED_VIEW_COLUMNS: &[&str] = &[
    "id",
    "kind",
    "name",
    "status",
    "lifecycleState",
    "criticality",
    "standardState",
    "ownerRefs",
    "capabilityRefs",
    "technologyRefs",
    "riskRefs",
    "relatedElementRefs",
    "tags",
];
const ROADMAP_BUCKET_SOURCES: &[&str] = &[
    "targetDate",
    "retirementDate",
    "endOfSupportDate",
    "currentFrom",
    "auto",
];
const ROADMAP_BUCKET_GRANULARITIES: &[&str] = &["month", "quarter", "half_year", "year"];
const ROADMAP_DATE_LABEL_FORMATS: &[&str] = &["date", "month", "quarter", "year"];
const ROADMAP_SWIMLANE_GROUPS: &[&str] = &[
    "portfolioKind",
    "lifecycleState",
    "criticality",
    "owner",
    "capability",
    "technology",
    "none",
];
const ROADMAP_MILESTONE_LINK_STYLES: &[&str] = &["solid", "dashed", "dotted"];
const ROADMAP_DENSITIES: &[&str] = &["compact", "comfortable", "detailed"];
const ROADMAP_COLOR_FIELDS: &[&str] = &[
    "lifecycleState",
    "criticality",
    "standardState",
    "portfolioKind",
    "none",
];

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
pub struct PortfolioFile {
    pub schema_version: String,
    pub objects: Vec<PortfolioObject>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioObject {
    pub id: String,
    pub kind: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(
        default = "default_element_status",
        skip_serializing_if = "is_default_element_status"
    )]
    pub status: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub lifecycle_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<PortfolioLifecycle>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub criticality: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub standard_state: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owner_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capability_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub technology_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_element_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_references: Vec<ExternalReference>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioLifecycle {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub state: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub phase: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub current_from: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_state: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_date: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub end_of_support_date: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub retirement_date: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub milestone_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSavedViewFile {
    pub schema_version: String,
    pub views: Vec<PortfolioSavedView>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSavedView {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub scope: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub result_kinds: Vec<String>,
    #[serde(default, skip_serializing_if = "PortfolioSavedViewQuery::is_empty")]
    pub query: PortfolioSavedViewQuery,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sort: Vec<PortfolioSavedViewSort>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "PortfolioSavedViewPresentation::is_empty"
    )]
    pub presentation: PortfolioSavedViewPresentation,
    #[serde(default, skip_serializing_if = "ElementProvenance::is_empty")]
    pub provenance: ElementProvenance,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSavedViewQuery {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kinds: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lifecycle_states: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub criticalities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub standard_states: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owner_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capability_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub technology_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risk_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_element_refs: Vec<String>,
}

impl PortfolioSavedViewQuery {
    fn is_empty(&self) -> bool {
        self.text.is_empty()
            && self.kinds.is_empty()
            && self.statuses.is_empty()
            && self.lifecycle_states.is_empty()
            && self.criticalities.is_empty()
            && self.standard_states.is_empty()
            && self.tags.is_empty()
            && self.owner_refs.is_empty()
            && self.capability_refs.is_empty()
            && self.technology_refs.is_empty()
            && self.risk_refs.is_empty()
            && self.related_element_refs.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSavedViewSort {
    pub field: String,
    #[serde(default = "default_sort_direction")]
    pub direction: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSavedViewPresentation {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub density: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub group_by: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_counts: Option<bool>,
}

impl PortfolioSavedViewPresentation {
    fn is_empty(&self) -> bool {
        self.density.is_empty() && self.group_by.is_empty() && self.show_counts.is_none()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapPresentationFile {
    pub schema_version: String,
    pub presentations: Vec<RoadmapPresentation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapPresentation {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies_to_view_kinds: Vec<String>,
    #[serde(default, skip_serializing_if = "RoadmapTimeline::is_empty")]
    pub timeline: RoadmapTimeline,
    #[serde(default, skip_serializing_if = "RoadmapSwimlanes::is_empty")]
    pub swimlanes: RoadmapSwimlanes,
    #[serde(default, skip_serializing_if = "RoadmapTargetStates::is_empty")]
    pub target_states: RoadmapTargetStates,
    #[serde(default, skip_serializing_if = "RoadmapMilestones::is_empty")]
    pub milestones: RoadmapMilestones,
    #[serde(default, skip_serializing_if = "RoadmapStyling::is_empty")]
    pub styling: RoadmapStyling,
    #[serde(default, skip_serializing_if = "ElementProvenance::is_empty")]
    pub provenance: ElementProvenance,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapTimeline {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bucket_source: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bucket_granularity: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub range_start: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub range_end: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_undated_bucket: Option<bool>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub date_label_format: String,
}

impl RoadmapTimeline {
    fn is_empty(&self) -> bool {
        self.bucket_source.is_empty()
            && self.bucket_granularity.is_empty()
            && self.range_start.is_empty()
            && self.range_end.is_empty()
            && self.include_undated_bucket.is_none()
            && self.date_label_format.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapSwimlanes {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub group_by: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub order: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_empty_lanes: Option<bool>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fallback_lane_title: String,
}

impl RoadmapSwimlanes {
    fn is_empty(&self) -> bool {
        self.group_by.is_empty()
            && self.order.is_empty()
            && self.include_empty_lanes.is_none()
            && self.fallback_lane_title.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapTargetStates {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_callouts: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_target_dates: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_no_change_targets: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states: Vec<String>,
}

impl RoadmapTargetStates {
    fn is_empty(&self) -> bool {
        self.show_callouts.is_none()
            && self.show_target_dates.is_none()
            && self.show_no_change_targets.is_none()
            && self.states.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapMilestones {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_milestone_nodes: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_milestone_links: Option<bool>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub link_style: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub include_unreferenced_milestones: Option<bool>,
}

impl RoadmapMilestones {
    fn is_empty(&self) -> bool {
        self.show_milestone_nodes.is_none()
            && self.show_milestone_links.is_none()
            && self.link_style.is_empty()
            && self.include_unreferenced_milestones.is_none()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapStyling {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub density: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub color_by: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_legend: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_timeline_scale: Option<bool>,
}

impl RoadmapStyling {
    fn is_empty(&self) -> bool {
        self.density.is_empty()
            && self.color_by.is_empty()
            && self.show_legend.is_none()
            && self.show_timeline_scale.is_none()
    }
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
    #[serde(default, skip_serializing_if = "ArchitectureDetails::is_empty")]
    pub architecture: ArchitectureDetails,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub classifier: Option<ClassifierDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_details: Option<ActorDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_case_details: Option<UseCaseDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_details: Option<ActivityDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequence_participant_details: Option<SequenceParticipantDetails>,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureDetails {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owners: Vec<ArchitectureOwner>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<ArchitectureLifecycle>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub criticality: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub technologies: Vec<TechnologyMapping>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub risks: Vec<RiskMapping>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<CapabilityMapping>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<ServiceMapping>,
}

impl ArchitectureDetails {
    fn is_empty(&self) -> bool {
        self.owners.is_empty()
            && self.lifecycle.is_none()
            && self.criticality.is_empty()
            && self.technologies.is_empty()
            && self.risks.is_empty()
            && self.capabilities.is_empty()
            && self.services.is_empty()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureOwner {
    #[serde(rename = "ref")]
    pub ref_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub role: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureLifecycle {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub state: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub phase: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub milestone_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_date: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TechnologyMapping {
    #[serde(rename = "ref")]
    pub ref_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub role: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub standard_state: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RiskMapping {
    #[serde(rename = "ref")]
    pub ref_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub severity: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub status: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityMapping {
    #[serde(rename = "ref")]
    pub ref_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fit: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub maturity: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceMapping {
    #[serde(rename = "ref")]
    pub ref_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub relationship: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub interface_ref: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClassifierDetails {
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_abstract: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_static: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<ClassifierAttribute>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<ClassifierOperation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClassifierAttribute {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub visibility: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub type_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiplicity: Option<Multiplicity>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub default_value: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_static: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_read_only: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub documentation: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClassifierOperation {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub visibility: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub return_type_ref: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<OperationParameter>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_abstract: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_static: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub documentation: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OperationParameter {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub type_ref: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub direction: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multiplicity: Option<Multiplicity>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub default_value: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Multiplicity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lower: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upper: Option<MultiplicityUpper>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_ordered: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_unique: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum MultiplicityUpper {
    Count(u64),
    Unbounded(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActorDetails {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub actor_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub responsibilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goals: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct UseCaseDetails {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub primary_actor_ref: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_actor_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preconditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub postconditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub main_flow: Vec<UseCaseStep>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternate_flows: Vec<UseCaseAlternateFlow>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extension_points: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UseCaseStep {
    pub step: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub actor_ref: String,
    pub action: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UseCaseAlternateFlow {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub trigger: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<UseCaseStep>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActivityDetails {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<ActivityParameter>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<ActivityNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flows: Vec<ActivityFlow>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActivityParameter {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub type_ref: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub direction: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActivityNode {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActivityFlow {
    pub id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub guard: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SequenceParticipantDetails {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub participant_kind: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub represents_ref: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub lifeline_name: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_external: bool,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub model_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub portfolio_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub roadmap_presentation_ref: String,
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
    pub portfolio: PortfolioFile,
    pub portfolio_saved_views: PortfolioSavedViewFile,
    pub roadmap_presentations: RoadmapPresentationFile,
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
    pub portfolio_objects_created: usize,
    pub portfolio_objects_updated: usize,
    pub portfolio_saved_view_operations_applied: usize,
    pub roadmap_presentation_operations_applied: usize,
    pub elements_created: usize,
    pub relationships_created: usize,
    pub diagrams_created: usize,
    pub trace_links_created: usize,
    pub model_element_detail_operations_applied: usize,
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
struct CreatePortfolioObjectArgs {
    id: String,
    kind: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_element_status")]
    status: String,
    #[serde(default)]
    lifecycle_state: String,
    #[serde(default)]
    lifecycle: Option<PortfolioLifecycle>,
    #[serde(default)]
    criticality: String,
    #[serde(default)]
    standard_state: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    owner_refs: Vec<String>,
    #[serde(default)]
    capability_refs: Vec<String>,
    #[serde(default)]
    technology_refs: Vec<String>,
    #[serde(default)]
    risk_refs: Vec<String>,
    #[serde(default)]
    related_element_refs: Vec<String>,
    #[serde(default)]
    source_refs: Vec<String>,
    #[serde(default)]
    external_references: Vec<ExternalReference>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePortfolioObjectArgs {
    object_id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    lifecycle_state: Option<String>,
    #[serde(default)]
    lifecycle: Option<PortfolioLifecycle>,
    #[serde(default)]
    criticality: Option<String>,
    #[serde(default)]
    standard_state: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    owner_refs: Option<Vec<String>>,
    #[serde(default)]
    capability_refs: Option<Vec<String>>,
    #[serde(default)]
    technology_refs: Option<Vec<String>>,
    #[serde(default)]
    risk_refs: Option<Vec<String>>,
    #[serde(default)]
    related_element_refs: Option<Vec<String>>,
    #[serde(default)]
    source_refs: Option<Vec<String>>,
    #[serde(default)]
    external_references: Option<Vec<ExternalReference>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePortfolioSavedViewArgs {
    id: String,
    title: String,
    #[serde(default)]
    description: String,
    scope: String,
    #[serde(default)]
    result_kinds: Vec<String>,
    #[serde(default)]
    query: PortfolioSavedViewQuery,
    #[serde(default)]
    sort: Vec<PortfolioSavedViewSort>,
    #[serde(default)]
    columns: Vec<String>,
    #[serde(default)]
    presentation: PortfolioSavedViewPresentation,
    #[serde(default)]
    provenance: ElementProvenance,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdatePortfolioSavedViewArgs {
    view_id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    result_kinds: Option<Vec<String>>,
    #[serde(default)]
    query: Option<PortfolioSavedViewQuery>,
    #[serde(default)]
    sort: Option<Vec<PortfolioSavedViewSort>>,
    #[serde(default)]
    columns: Option<Vec<String>>,
    #[serde(default)]
    presentation: Option<PortfolioSavedViewPresentation>,
    #[serde(default)]
    provenance: Option<ElementProvenance>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemovePortfolioSavedViewArgs {
    view_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateRoadmapPresentationArgs {
    id: String,
    title: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    applies_to_view_kinds: Vec<String>,
    #[serde(default)]
    timeline: RoadmapTimeline,
    #[serde(default)]
    swimlanes: RoadmapSwimlanes,
    #[serde(default)]
    target_states: RoadmapTargetStates,
    #[serde(default)]
    milestones: RoadmapMilestones,
    #[serde(default)]
    styling: RoadmapStyling,
    #[serde(default)]
    provenance: ElementProvenance,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateRoadmapPresentationArgs {
    presentation_id: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    applies_to_view_kinds: Option<Vec<String>>,
    #[serde(default)]
    timeline: Option<RoadmapTimeline>,
    #[serde(default)]
    swimlanes: Option<RoadmapSwimlanes>,
    #[serde(default)]
    target_states: Option<RoadmapTargetStates>,
    #[serde(default)]
    milestones: Option<RoadmapMilestones>,
    #[serde(default)]
    styling: Option<RoadmapStyling>,
    #[serde(default)]
    provenance: Option<ElementProvenance>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemoveRoadmapPresentationArgs {
    presentation_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssignRoadmapPresentationArgs {
    diagram_id: String,
    presentation_id: String,
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
    #[serde(default)]
    architecture: ArchitectureDetails,
    #[serde(default)]
    classifier: Option<ClassifierDetails>,
    #[serde(default)]
    actor_details: Option<ActorDetails>,
    #[serde(default)]
    use_case_details: Option<UseCaseDetails>,
    #[serde(default)]
    activity_details: Option<ActivityDetails>,
    #[serde(default)]
    sequence_participant_details: Option<SequenceParticipantDetails>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateModelElementDetailsArgs {
    element_id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    aliases: Option<Vec<String>>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    documentation: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    stereotypes: Option<Vec<String>>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    provenance: Option<ElementProvenance>,
    #[serde(default)]
    external_references: Option<Vec<ExternalReference>>,
    #[serde(default)]
    architecture: Option<ArchitectureDetails>,
    #[serde(default)]
    classifier: Option<ClassifierDetails>,
    #[serde(default)]
    actor_details: Option<ActorDetails>,
    #[serde(default)]
    use_case_details: Option<UseCaseDetails>,
    #[serde(default)]
    activity_details: Option<ActivityDetails>,
    #[serde(default)]
    sequence_participant_details: Option<SequenceParticipantDetails>,
    #[serde(default)]
    clear_details: Vec<String>,
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
    #[serde(default)]
    model_refs: Vec<String>,
    #[serde(default)]
    portfolio_refs: Vec<String>,
    #[serde(default)]
    roadmap_presentation_ref: String,
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
        portfolio: read_json(root.join("model/portfolio.json"))?,
        portfolio_saved_views: read_json(root.join("views/portfolio-views.json"))?,
        roadmap_presentations: read_json(root.join("views/roadmap-presentations.json"))?,
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
        portfolio_objects_created: 0,
        portfolio_objects_updated: 0,
        portfolio_saved_view_operations_applied: 0,
        roadmap_presentation_operations_applied: 0,
        elements_created: 0,
        relationships_created: 0,
        diagrams_created: 0,
        trace_links_created: 0,
        model_element_detail_operations_applied: 0,
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
            "create_portfolio_object" => {
                let args: CreatePortfolioObjectArgs = parse_args(operation)?;
                ensure_available_id(package, &args.id)?;
                package.portfolio.objects.push(PortfolioObject {
                    id: args.id,
                    kind: args.kind,
                    name: args.name,
                    description: args.description,
                    status: args.status,
                    lifecycle_state: args.lifecycle_state,
                    lifecycle: args.lifecycle,
                    criticality: args.criticality,
                    standard_state: args.standard_state,
                    tags: args.tags,
                    owner_refs: args.owner_refs,
                    capability_refs: args.capability_refs,
                    technology_refs: args.technology_refs,
                    risk_refs: args.risk_refs,
                    related_element_refs: args.related_element_refs,
                    source_refs: args.source_refs,
                    external_references: args.external_references,
                });
                summary.portfolio_objects_created += 1;
            }
            "update_portfolio_object" => {
                let args: UpdatePortfolioObjectArgs = parse_args(operation)?;
                update_portfolio_object(package, args)?;
                summary.portfolio_objects_updated += 1;
            }
            "create_portfolio_saved_view" => {
                let args: CreatePortfolioSavedViewArgs = parse_args(operation)?;
                ensure_available_id(package, &args.id)?;
                package
                    .portfolio_saved_views
                    .views
                    .push(PortfolioSavedView {
                        id: args.id,
                        title: args.title,
                        description: args.description,
                        scope: args.scope,
                        result_kinds: args.result_kinds,
                        query: args.query,
                        sort: args.sort,
                        columns: args.columns,
                        presentation: args.presentation,
                        provenance: args.provenance,
                    });
                summary.portfolio_saved_view_operations_applied += 1;
            }
            "update_portfolio_saved_view" => {
                let args: UpdatePortfolioSavedViewArgs = parse_args(operation)?;
                update_portfolio_saved_view(package, args)?;
                summary.portfolio_saved_view_operations_applied += 1;
            }
            "remove_portfolio_saved_view" => {
                let args: RemovePortfolioSavedViewArgs = parse_args(operation)?;
                remove_portfolio_saved_view(package, args)?;
                summary.portfolio_saved_view_operations_applied += 1;
            }
            "create_roadmap_presentation" => {
                let args: CreateRoadmapPresentationArgs = parse_args(operation)?;
                ensure_available_id(package, &args.id)?;
                package
                    .roadmap_presentations
                    .presentations
                    .push(RoadmapPresentation {
                        id: args.id,
                        title: args.title,
                        description: args.description,
                        applies_to_view_kinds: args.applies_to_view_kinds,
                        timeline: args.timeline,
                        swimlanes: args.swimlanes,
                        target_states: args.target_states,
                        milestones: args.milestones,
                        styling: args.styling,
                        provenance: args.provenance,
                    });
                summary.roadmap_presentation_operations_applied += 1;
            }
            "update_roadmap_presentation" => {
                let args: UpdateRoadmapPresentationArgs = parse_args(operation)?;
                update_roadmap_presentation(package, args)?;
                summary.roadmap_presentation_operations_applied += 1;
            }
            "remove_roadmap_presentation" => {
                let args: RemoveRoadmapPresentationArgs = parse_args(operation)?;
                remove_roadmap_presentation(package, args)?;
                summary.roadmap_presentation_operations_applied += 1;
            }
            "assign_roadmap_presentation" => {
                let args: AssignRoadmapPresentationArgs = parse_args(operation)?;
                assign_roadmap_presentation(package, args)?;
                summary.roadmap_presentation_operations_applied += 1;
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
                    architecture: args.architecture,
                    classifier: args.classifier,
                    actor_details: args.actor_details,
                    use_case_details: args.use_case_details,
                    activity_details: args.activity_details,
                    sequence_participant_details: args.sequence_participant_details,
                });
                summary.elements_created += 1;
            }
            "update_model_element_details" => {
                let args: UpdateModelElementDetailsArgs = parse_args(operation)?;
                update_model_element_details(package, args)?;
                summary.model_element_detail_operations_applied += 1;
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
                    portfolio_refs: args.portfolio_refs,
                    roadmap_presentation_ref: args.roadmap_presentation_ref,
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

fn update_portfolio_object(
    package: &mut ModelPackage,
    args: UpdatePortfolioObjectArgs,
) -> Result<()> {
    if args.object_id.trim().is_empty() {
        bail!("update_portfolio_object objectId must not be empty");
    }
    if !has_portfolio_object_update(&args) {
        bail!(
            "update_portfolio_object for {} must change at least one field",
            args.object_id
        );
    }

    let object = package
        .portfolio
        .objects
        .iter_mut()
        .find(|object| object.id == args.object_id)
        .ok_or_else(|| anyhow!("missing portfolio object {}", args.object_id))?;

    if let Some(name) = args.name {
        object.name = name;
    }
    if let Some(description) = args.description {
        object.description = description;
    }
    if let Some(status) = args.status {
        object.status = status;
    }
    if let Some(lifecycle_state) = args.lifecycle_state {
        object.lifecycle_state = lifecycle_state;
    }
    if let Some(lifecycle) = args.lifecycle {
        object.lifecycle = Some(lifecycle);
    }
    if let Some(criticality) = args.criticality {
        object.criticality = criticality;
    }
    if let Some(standard_state) = args.standard_state {
        object.standard_state = standard_state;
    }
    if let Some(tags) = args.tags {
        object.tags = tags;
    }
    if let Some(owner_refs) = args.owner_refs {
        object.owner_refs = owner_refs;
    }
    if let Some(capability_refs) = args.capability_refs {
        object.capability_refs = capability_refs;
    }
    if let Some(technology_refs) = args.technology_refs {
        object.technology_refs = technology_refs;
    }
    if let Some(risk_refs) = args.risk_refs {
        object.risk_refs = risk_refs;
    }
    if let Some(related_element_refs) = args.related_element_refs {
        object.related_element_refs = related_element_refs;
    }
    if let Some(source_refs) = args.source_refs {
        object.source_refs = source_refs;
    }
    if let Some(external_references) = args.external_references {
        object.external_references = external_references;
    }

    Ok(())
}

fn has_portfolio_object_update(args: &UpdatePortfolioObjectArgs) -> bool {
    args.name.is_some()
        || args.description.is_some()
        || args.status.is_some()
        || args.lifecycle_state.is_some()
        || args.lifecycle.is_some()
        || args.criticality.is_some()
        || args.standard_state.is_some()
        || args.tags.is_some()
        || args.owner_refs.is_some()
        || args.capability_refs.is_some()
        || args.technology_refs.is_some()
        || args.risk_refs.is_some()
        || args.related_element_refs.is_some()
        || args.source_refs.is_some()
        || args.external_references.is_some()
}

fn update_portfolio_saved_view(
    package: &mut ModelPackage,
    args: UpdatePortfolioSavedViewArgs,
) -> Result<()> {
    if args.view_id.trim().is_empty() {
        bail!("update_portfolio_saved_view viewId must not be empty");
    }
    if !has_portfolio_saved_view_update(&args) {
        bail!(
            "update_portfolio_saved_view for {} must change at least one field",
            args.view_id
        );
    }

    let view = package
        .portfolio_saved_views
        .views
        .iter_mut()
        .find(|view| view.id == args.view_id)
        .ok_or_else(|| anyhow!("missing portfolio saved view {}", args.view_id))?;

    if let Some(title) = args.title {
        view.title = title;
    }
    if let Some(description) = args.description {
        view.description = description;
    }
    if let Some(scope) = args.scope {
        view.scope = scope;
    }
    if let Some(result_kinds) = args.result_kinds {
        view.result_kinds = result_kinds;
    }
    if let Some(query) = args.query {
        view.query = query;
    }
    if let Some(sort) = args.sort {
        view.sort = sort;
    }
    if let Some(columns) = args.columns {
        view.columns = columns;
    }
    if let Some(presentation) = args.presentation {
        view.presentation = presentation;
    }
    if let Some(provenance) = args.provenance {
        view.provenance = provenance;
    }

    Ok(())
}

fn remove_portfolio_saved_view(
    package: &mut ModelPackage,
    args: RemovePortfolioSavedViewArgs,
) -> Result<()> {
    if args.view_id.trim().is_empty() {
        bail!("remove_portfolio_saved_view viewId must not be empty");
    }
    let initial_len = package.portfolio_saved_views.views.len();
    package
        .portfolio_saved_views
        .views
        .retain(|view| view.id != args.view_id);
    if package.portfolio_saved_views.views.len() == initial_len {
        bail!("missing portfolio saved view {}", args.view_id);
    }
    Ok(())
}

fn has_portfolio_saved_view_update(args: &UpdatePortfolioSavedViewArgs) -> bool {
    args.title.is_some()
        || args.description.is_some()
        || args.scope.is_some()
        || args.result_kinds.is_some()
        || args.query.is_some()
        || args.sort.is_some()
        || args.columns.is_some()
        || args.presentation.is_some()
        || args.provenance.is_some()
}

fn update_roadmap_presentation(
    package: &mut ModelPackage,
    args: UpdateRoadmapPresentationArgs,
) -> Result<()> {
    if args.presentation_id.trim().is_empty() {
        bail!("update_roadmap_presentation presentationId must not be empty");
    }
    if !has_roadmap_presentation_update(&args) {
        bail!(
            "update_roadmap_presentation for {} must change at least one field",
            args.presentation_id
        );
    }

    let presentation = package
        .roadmap_presentations
        .presentations
        .iter_mut()
        .find(|presentation| presentation.id == args.presentation_id)
        .ok_or_else(|| anyhow!("missing roadmap presentation {}", args.presentation_id))?;

    if let Some(title) = args.title {
        presentation.title = title;
    }
    if let Some(description) = args.description {
        presentation.description = description;
    }
    if let Some(applies_to_view_kinds) = args.applies_to_view_kinds {
        presentation.applies_to_view_kinds = applies_to_view_kinds;
    }
    if let Some(timeline) = args.timeline {
        presentation.timeline = timeline;
    }
    if let Some(swimlanes) = args.swimlanes {
        presentation.swimlanes = swimlanes;
    }
    if let Some(target_states) = args.target_states {
        presentation.target_states = target_states;
    }
    if let Some(milestones) = args.milestones {
        presentation.milestones = milestones;
    }
    if let Some(styling) = args.styling {
        presentation.styling = styling;
    }
    if let Some(provenance) = args.provenance {
        presentation.provenance = provenance;
    }

    Ok(())
}

fn remove_roadmap_presentation(
    package: &mut ModelPackage,
    args: RemoveRoadmapPresentationArgs,
) -> Result<()> {
    if args.presentation_id.trim().is_empty() {
        bail!("remove_roadmap_presentation presentationId must not be empty");
    }
    if package
        .diagrams
        .diagrams
        .iter()
        .any(|diagram| diagram.roadmap_presentation_ref == args.presentation_id)
    {
        bail!(
            "cannot remove roadmap presentation {} while it is assigned to a diagram",
            args.presentation_id
        );
    }
    let initial_len = package.roadmap_presentations.presentations.len();
    package
        .roadmap_presentations
        .presentations
        .retain(|presentation| presentation.id != args.presentation_id);
    if package.roadmap_presentations.presentations.len() == initial_len {
        bail!("missing roadmap presentation {}", args.presentation_id);
    }
    Ok(())
}

fn assign_roadmap_presentation(
    package: &mut ModelPackage,
    args: AssignRoadmapPresentationArgs,
) -> Result<()> {
    if args.diagram_id.trim().is_empty() {
        bail!("assign_roadmap_presentation diagramId must not be empty");
    }
    if args.presentation_id.trim().is_empty() {
        bail!("assign_roadmap_presentation presentationId must not be empty");
    }
    let presentation = package
        .roadmap_presentations
        .presentations
        .iter()
        .find(|presentation| presentation.id == args.presentation_id)
        .ok_or_else(|| anyhow!("missing roadmap presentation {}", args.presentation_id))?;
    if !roadmap_presentation_applies_to(presentation, "lifecycle_roadmap") {
        bail!(
            "{} does not apply to lifecycle_roadmap",
            args.presentation_id
        );
    }

    let diagram = package
        .diagrams
        .diagrams
        .iter_mut()
        .find(|diagram| diagram.id == args.diagram_id)
        .ok_or_else(|| anyhow!("missing diagram {}", args.diagram_id))?;
    if diagram.view_kind != "lifecycle_roadmap" {
        bail!(
            "{} is a {} diagram, not lifecycle_roadmap",
            args.diagram_id,
            diagram.view_kind
        );
    }
    diagram.roadmap_presentation_ref = args.presentation_id;
    Ok(())
}

fn has_roadmap_presentation_update(args: &UpdateRoadmapPresentationArgs) -> bool {
    args.title.is_some()
        || args.description.is_some()
        || args.applies_to_view_kinds.is_some()
        || args.timeline.is_some()
        || args.swimlanes.is_some()
        || args.target_states.is_some()
        || args.milestones.is_some()
        || args.styling.is_some()
        || args.provenance.is_some()
}

fn update_model_element_details(
    package: &mut ModelPackage,
    args: UpdateModelElementDetailsArgs,
) -> Result<()> {
    if args.element_id.trim().is_empty() {
        bail!("update_model_element_details elementId must not be empty");
    }
    if !has_model_element_detail_update(&args) {
        bail!(
            "update_model_element_details for {} must change at least one field",
            args.element_id
        );
    }

    let element = package
        .elements
        .elements
        .iter_mut()
        .find(|element| element.id == args.element_id)
        .ok_or_else(|| anyhow!("missing model element {}", args.element_id))?;

    for detail in &args.clear_details {
        match detail.as_str() {
            "architecture" => element.architecture = ArchitectureDetails::default(),
            "classifier" => element.classifier = None,
            "actorDetails" => element.actor_details = None,
            "useCaseDetails" => element.use_case_details = None,
            "activityDetails" => element.activity_details = None,
            "sequenceParticipantDetails" => element.sequence_participant_details = None,
            other => bail!("unsupported clearDetails entry {other}"),
        }
    }

    if let Some(name) = args.name {
        element.name = name;
    }
    if let Some(aliases) = args.aliases {
        element.aliases = aliases;
    }
    if let Some(description) = args.description {
        element.description = description;
    }
    if let Some(documentation) = args.documentation {
        element.documentation = documentation;
    }
    if let Some(status) = args.status {
        element.status = status;
    }
    if let Some(stereotypes) = args.stereotypes {
        element.stereotypes = stereotypes;
    }
    if let Some(tags) = args.tags {
        element.tags = tags;
    }
    if let Some(provenance) = args.provenance {
        element.provenance = provenance;
    }
    if let Some(external_references) = args.external_references {
        element.external_references = external_references;
    }
    if let Some(architecture) = args.architecture {
        element.architecture = architecture;
    }
    if let Some(classifier) = args.classifier {
        element.classifier = Some(classifier);
    }
    if let Some(actor_details) = args.actor_details {
        element.actor_details = Some(actor_details);
    }
    if let Some(use_case_details) = args.use_case_details {
        element.use_case_details = Some(use_case_details);
    }
    if let Some(activity_details) = args.activity_details {
        element.activity_details = Some(activity_details);
    }
    if let Some(sequence_participant_details) = args.sequence_participant_details {
        element.sequence_participant_details = Some(sequence_participant_details);
    }

    Ok(())
}

fn has_model_element_detail_update(args: &UpdateModelElementDetailsArgs) -> bool {
    args.name.is_some()
        || args.aliases.is_some()
        || args.description.is_some()
        || args.documentation.is_some()
        || args.status.is_some()
        || args.stereotypes.is_some()
        || args.tags.is_some()
        || args.provenance.is_some()
        || args.external_references.is_some()
        || args.architecture.is_some()
        || args.classifier.is_some()
        || args.actor_details.is_some()
        || args.use_case_details.is_some()
        || args.activity_details.is_some()
        || args.sequence_participant_details.is_some()
        || !args.clear_details.is_empty()
}

fn validate_portfolio_object(object: &PortfolioObject) -> Result<()> {
    ensure_non_empty(&object.name, &format!("{} name", object.id))?;
    if !PORTFOLIO_KINDS.contains(&object.kind.as_str()) {
        bail!(
            "{} has unsupported portfolio object kind {}",
            object.id,
            object.kind
        );
    }
    if !PORTFOLIO_STATUSES.contains(&object.status.as_str()) {
        bail!(
            "{} has unsupported portfolio object status {}",
            object.id,
            object.status
        );
    }
    validate_optional_value(
        &object.lifecycle_state,
        LIFECYCLE_STATES,
        &format!("{} lifecycleState", object.id),
    )?;
    if let Some(lifecycle) = &object.lifecycle {
        validate_portfolio_lifecycle(object, lifecycle)?;
    }
    validate_optional_value(
        &object.criticality,
        CRITICALITIES,
        &format!("{} criticality", object.id),
    )?;
    validate_optional_value(
        &object.standard_state,
        STANDARD_STATES,
        &format!("{} standardState", object.id),
    )?;
    ensure_non_empty_items(&object.tags, &format!("{} tag", object.id))?;
    ensure_non_empty_items(&object.owner_refs, &format!("{} ownerRef", object.id))?;
    ensure_non_empty_items(
        &object.capability_refs,
        &format!("{} capabilityRef", object.id),
    )?;
    ensure_non_empty_items(
        &object.technology_refs,
        &format!("{} technologyRef", object.id),
    )?;
    ensure_non_empty_items(&object.risk_refs, &format!("{} riskRef", object.id))?;
    ensure_non_empty_items(
        &object.related_element_refs,
        &format!("{} relatedElementRef", object.id),
    )?;
    ensure_non_empty_items(&object.source_refs, &format!("{} sourceRef", object.id))?;
    for reference in &object.external_references {
        ensure_non_empty(
            &reference.id,
            &format!("{} external reference id", object.id),
        )?;
        ensure_non_empty(
            &reference.label,
            &format!("{} external reference label", object.id),
        )?;
        ensure_non_empty(
            &reference.uri,
            &format!("{} external reference uri", object.id),
        )?;
    }
    Ok(())
}

fn validate_portfolio_lifecycle(
    object: &PortfolioObject,
    lifecycle: &PortfolioLifecycle,
) -> Result<()> {
    validate_optional_value(
        &lifecycle.state,
        LIFECYCLE_STATES,
        &format!("{} lifecycle state", object.id),
    )?;
    validate_optional_value(
        &lifecycle.target_state,
        LIFECYCLE_TARGET_STATES,
        &format!("{} lifecycle targetState", object.id),
    )?;
    if !lifecycle.phase.is_empty() {
        ensure_non_empty(&lifecycle.phase, &format!("{} lifecycle phase", object.id))?;
    }
    validate_optional_date(
        &lifecycle.current_from,
        &format!("{} lifecycle currentFrom", object.id),
    )?;
    validate_optional_date(
        &lifecycle.target_date,
        &format!("{} lifecycle targetDate", object.id),
    )?;
    validate_optional_date(
        &lifecycle.end_of_support_date,
        &format!("{} lifecycle endOfSupportDate", object.id),
    )?;
    validate_optional_date(
        &lifecycle.retirement_date,
        &format!("{} lifecycle retirementDate", object.id),
    )?;
    ensure_non_empty_items(
        &lifecycle.milestone_refs,
        &format!("{} lifecycle milestoneRef", object.id),
    )?;
    Ok(())
}

fn validate_portfolio_saved_view(
    view: &PortfolioSavedView,
    portfolio_ids: &BTreeSet<&str>,
    element_kinds: &BTreeMap<&str, &str>,
) -> Result<()> {
    ensure_non_empty(&view.title, &format!("{} title", view.id))?;
    validate_required_value(
        &view.scope,
        &["portfolio_summary", "portfolio_view_source", "export_set"],
        &format!("{} scope", view.id),
    )?;
    for kind in &view.result_kinds {
        validate_required_value(kind, PORTFOLIO_KINDS, &format!("{} resultKind", view.id))?;
    }
    validate_portfolio_saved_view_query(view, portfolio_ids, element_kinds)?;
    for sort in &view.sort {
        validate_required_value(
            &sort.field,
            SAVED_VIEW_SORT_FIELDS,
            &format!("{} sort field", view.id),
        )?;
        validate_required_value(
            &sort.direction,
            &["asc", "desc"],
            &format!("{} sort direction", view.id),
        )?;
    }
    for column in &view.columns {
        validate_required_value(column, SAVED_VIEW_COLUMNS, &format!("{} column", view.id))?;
    }
    if !view.presentation.density.is_empty() {
        validate_required_value(
            &view.presentation.density,
            &["compact", "comfortable", "detailed"],
            &format!("{} presentation density", view.id),
        )?;
    }
    if !view.presentation.group_by.is_empty() {
        validate_required_value(
            &view.presentation.group_by,
            &[
                "kind",
                "status",
                "lifecycleState",
                "criticality",
                "standardState",
                "owner",
                "capability",
            ],
            &format!("{} presentation groupBy", view.id),
        )?;
    }
    validate_element_provenance_for_id(&view.id, &view.provenance)?;
    Ok(())
}

fn validate_portfolio_saved_view_query(
    view: &PortfolioSavedView,
    portfolio_ids: &BTreeSet<&str>,
    element_kinds: &BTreeMap<&str, &str>,
) -> Result<()> {
    ensure_non_empty_items(&view.query.kinds, &format!("{} query kind", view.id))?;
    for kind in &view.query.kinds {
        validate_required_value(kind, PORTFOLIO_KINDS, &format!("{} query kind", view.id))?;
    }
    for status in &view.query.statuses {
        validate_required_value(
            status,
            PORTFOLIO_STATUSES,
            &format!("{} query status", view.id),
        )?;
    }
    for lifecycle_state in &view.query.lifecycle_states {
        validate_required_value(
            lifecycle_state,
            LIFECYCLE_STATES,
            &format!("{} query lifecycleState", view.id),
        )?;
    }
    for criticality in &view.query.criticalities {
        validate_required_value(
            criticality,
            CRITICALITIES,
            &format!("{} query criticality", view.id),
        )?;
    }
    for standard_state in &view.query.standard_states {
        validate_required_value(
            standard_state,
            STANDARD_STATES,
            &format!("{} query standardState", view.id),
        )?;
    }
    ensure_non_empty_items(&view.query.tags, &format!("{} query tag", view.id))?;
    validate_local_portfolio_refs(
        &view.query.owner_refs,
        portfolio_ids,
        &format!("{} query ownerRef", view.id),
    )?;
    validate_local_portfolio_refs(
        &view.query.capability_refs,
        portfolio_ids,
        &format!("{} query capabilityRef", view.id),
    )?;
    validate_local_portfolio_refs(
        &view.query.technology_refs,
        portfolio_ids,
        &format!("{} query technologyRef", view.id),
    )?;
    validate_local_portfolio_refs(
        &view.query.risk_refs,
        portfolio_ids,
        &format!("{} query riskRef", view.id),
    )?;
    for related_element_ref in &view.query.related_element_refs {
        if !element_kinds.contains_key(related_element_ref.as_str()) {
            bail!(
                "{} query relatedElementRef references missing model element {}",
                view.id,
                related_element_ref
            );
        }
    }
    Ok(())
}

fn validate_local_portfolio_refs(
    refs: &[String],
    portfolio_ids: &BTreeSet<&str>,
    field: &str,
) -> Result<()> {
    ensure_non_empty_items(refs, field)?;
    for reference in refs {
        if !portfolio_ids.contains(reference.as_str()) {
            bail!("{field} references missing portfolio object {reference}");
        }
    }
    Ok(())
}

fn validate_roadmap_presentation(presentation: &RoadmapPresentation) -> Result<()> {
    ensure_non_empty(&presentation.title, &format!("{} title", presentation.id))?;
    if presentation.applies_to_view_kinds.is_empty() {
        bail!("{} appliesToViewKinds must not be empty", presentation.id);
    }
    for view_kind in &presentation.applies_to_view_kinds {
        validate_required_value(
            view_kind,
            &["lifecycle_roadmap"],
            &format!("{} appliesToViewKind", presentation.id),
        )?;
    }

    if !presentation.timeline.bucket_source.is_empty() {
        validate_required_value(
            &presentation.timeline.bucket_source,
            ROADMAP_BUCKET_SOURCES,
            &format!("{} timeline bucketSource", presentation.id),
        )?;
    }
    if !presentation.timeline.bucket_granularity.is_empty() {
        validate_required_value(
            &presentation.timeline.bucket_granularity,
            ROADMAP_BUCKET_GRANULARITIES,
            &format!("{} timeline bucketGranularity", presentation.id),
        )?;
    }
    validate_optional_date(
        &presentation.timeline.range_start,
        &format!("{} timeline rangeStart", presentation.id),
    )?;
    validate_optional_date(
        &presentation.timeline.range_end,
        &format!("{} timeline rangeEnd", presentation.id),
    )?;
    if !presentation.timeline.range_start.is_empty()
        && !presentation.timeline.range_end.is_empty()
        && presentation.timeline.range_start.as_str() > presentation.timeline.range_end.as_str()
    {
        bail!(
            "{} timeline rangeStart must not be after rangeEnd",
            presentation.id
        );
    }
    if !presentation.timeline.date_label_format.is_empty() {
        validate_required_value(
            &presentation.timeline.date_label_format,
            ROADMAP_DATE_LABEL_FORMATS,
            &format!("{} timeline dateLabelFormat", presentation.id),
        )?;
    }

    if !presentation.swimlanes.group_by.is_empty() {
        validate_required_value(
            &presentation.swimlanes.group_by,
            ROADMAP_SWIMLANE_GROUPS,
            &format!("{} swimlanes groupBy", presentation.id),
        )?;
    }
    ensure_non_empty_items(
        &presentation.swimlanes.order,
        &format!("{} swimlanes order", presentation.id),
    )?;
    if !presentation.swimlanes.fallback_lane_title.is_empty() {
        ensure_non_empty(
            &presentation.swimlanes.fallback_lane_title,
            &format!("{} swimlanes fallbackLaneTitle", presentation.id),
        )?;
    }

    for state in &presentation.target_states.states {
        validate_required_value(
            state,
            LIFECYCLE_TARGET_STATES,
            &format!("{} targetStates state", presentation.id),
        )?;
    }
    if !presentation.milestones.link_style.is_empty() {
        validate_required_value(
            &presentation.milestones.link_style,
            ROADMAP_MILESTONE_LINK_STYLES,
            &format!("{} milestones linkStyle", presentation.id),
        )?;
    }
    if !presentation.styling.density.is_empty() {
        validate_required_value(
            &presentation.styling.density,
            ROADMAP_DENSITIES,
            &format!("{} styling density", presentation.id),
        )?;
    }
    if !presentation.styling.color_by.is_empty() {
        validate_required_value(
            &presentation.styling.color_by,
            ROADMAP_COLOR_FIELDS,
            &format!("{} styling colorBy", presentation.id),
        )?;
    }
    validate_element_provenance_for_id(&presentation.id, &presentation.provenance)?;
    Ok(())
}

fn roadmap_presentation_applies_to(presentation: &RoadmapPresentation, view_kind: &str) -> bool {
    presentation
        .applies_to_view_kinds
        .iter()
        .any(|candidate| candidate == view_kind)
}

pub fn validate_package(package: &ModelPackage) -> Result<Vec<String>> {
    let mut warnings = Vec::new();
    require_version("manifest", &package.manifest.schema_version)?;
    require_version("requirements", &package.requirements.schema_version)?;
    require_version("portfolio", &package.portfolio.schema_version)?;
    require_version(
        "portfolio saved views",
        &package.portfolio_saved_views.schema_version,
    )?;
    require_version(
        "roadmap presentations",
        &package.roadmap_presentations.schema_version,
    )?;
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

    for object in &package.portfolio.objects {
        ensure_unique(&mut ids, &object.id)?;
        validate_portfolio_object(object)?;
    }
    let portfolio_ids: BTreeSet<&str> = package
        .portfolio
        .objects
        .iter()
        .map(|object| object.id.as_str())
        .collect();

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
        validate_architecture_details(element)?;
        validate_classifier_details(element)?;
        validate_specialized_element_details(element)?;
        element_kinds.insert(element.id.as_str(), element.kind.as_str());
    }

    for object in &package.portfolio.objects {
        for element_ref in &object.related_element_refs {
            if !element_kinds.contains_key(element_ref.as_str()) {
                bail!(
                    "{} references missing model element {}",
                    object.id,
                    element_ref
                );
            }
        }
    }

    for saved_view in &package.portfolio_saved_views.views {
        ensure_unique(&mut ids, &saved_view.id)?;
        validate_portfolio_saved_view(saved_view, &portfolio_ids, &element_kinds)?;
    }

    let mut roadmap_presentations_by_id = BTreeMap::new();
    for presentation in &package.roadmap_presentations.presentations {
        ensure_unique(&mut ids, &presentation.id)?;
        validate_roadmap_presentation(presentation)?;
        roadmap_presentations_by_id.insert(presentation.id.as_str(), presentation);
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
        if !matches!(
            diagram.view_kind.as_str(),
            "use_case"
                | "capability_map"
                | "application_landscape"
                | "lifecycle_roadmap"
                | "risk_heatmap"
                | "dependency_map"
        ) {
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
        for portfolio_ref in &diagram.portfolio_refs {
            if !portfolio_ids.contains(portfolio_ref.as_str()) {
                bail!(
                    "{} references missing portfolio object {}",
                    diagram.id,
                    portfolio_ref
                );
            }
        }
        if !diagram.roadmap_presentation_ref.is_empty() {
            let Some(presentation) =
                roadmap_presentations_by_id.get(diagram.roadmap_presentation_ref.as_str())
            else {
                bail!(
                    "{} references missing roadmap presentation {}",
                    diagram.id,
                    diagram.roadmap_presentation_ref
                );
            };
            if diagram.view_kind != "lifecycle_roadmap" {
                bail!(
                    "{} assigns roadmap presentation {} to non-roadmap view kind {}",
                    diagram.id,
                    diagram.roadmap_presentation_ref,
                    diagram.view_kind
                );
            }
            if !roadmap_presentation_applies_to(presentation, &diagram.view_kind) {
                bail!(
                    "{} roadmap presentation {} does not apply to {}",
                    diagram.id,
                    diagram.roadmap_presentation_ref,
                    diagram.view_kind
                );
            }
        }
        if let Some(layout) = &diagram.layout {
            validate_diagram_layout(
                diagram,
                layout,
                &element_kinds,
                &portfolio_ids,
                &relationships_by_id,
            )?;
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

pub fn portfolio_summary_lines(package: &ModelPackage, query: Option<&str>) -> Vec<String> {
    let query = query.map(str::trim).filter(|value| !value.is_empty());
    let objects: Vec<&PortfolioObject> = package
        .portfolio
        .objects
        .iter()
        .filter(|object| match query {
            Some(query) => portfolio_object_matches_query(object, query),
            None => true,
        })
        .collect();
    let mut lines = vec![
        match query {
            Some(query) => format!(
                "portfolio objects: {} of {} matching \"{}\"",
                objects.len(),
                package.portfolio.objects.len(),
                query
            ),
            None => format!("portfolio objects: {}", objects.len()),
        },
        format!(
            "related model links: {}",
            objects
                .iter()
                .map(|object| object.related_element_refs.len())
                .sum::<usize>()
        ),
    ];

    push_count_lines(
        &mut lines,
        "kind",
        objects.iter().map(|object| object.kind.as_str()),
    );
    push_count_lines(
        &mut lines,
        "lifecycle",
        objects.iter().map(|object| {
            if object.lifecycle_state.is_empty() {
                "unspecified"
            } else {
                object.lifecycle_state.as_str()
            }
        }),
    );
    push_count_lines(
        &mut lines,
        "criticality",
        objects.iter().map(|object| {
            if object.criticality.is_empty() {
                "unspecified"
            } else {
                object.criticality.as_str()
            }
        }),
    );
    push_count_lines(
        &mut lines,
        "standard",
        objects.iter().filter_map(|object| {
            if object.standard_state.is_empty() {
                None
            } else {
                Some(object.standard_state.as_str())
            }
        }),
    );

    lines.push("objects:".to_string());
    for object in objects {
        let lifecycle = if object.lifecycle_state.is_empty() {
            "unspecified"
        } else {
            object.lifecycle_state.as_str()
        };
        let criticality = if object.criticality.is_empty() {
            "unspecified"
        } else {
            object.criticality.as_str()
        };
        lines.push(format!(
            "- {} [{}] status={} lifecycle={} criticality={}",
            object.id, object.kind, object.status, lifecycle, criticality
        ));
    }

    lines
}

fn portfolio_object_matches_query(object: &PortfolioObject, query: &str) -> bool {
    let query = query.to_ascii_lowercase();
    let fields = [
        object.id.as_str(),
        object.kind.as_str(),
        object.name.as_str(),
        object.description.as_str(),
        object.status.as_str(),
        object.lifecycle_state.as_str(),
        object.criticality.as_str(),
        object.standard_state.as_str(),
    ];
    fields
        .iter()
        .copied()
        .chain(object.tags.iter().map(String::as_str))
        .chain(object.source_refs.iter().map(String::as_str))
        .any(|value| value.to_ascii_lowercase().contains(&query))
}

fn push_count_lines<'a>(
    lines: &mut Vec<String>,
    label: &str,
    values: impl Iterator<Item = &'a str>,
) {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    if counts.is_empty() {
        lines.push(format!("{label}: none"));
        return;
    }
    lines.push(format!("{label}:"));
    for (value, count) in counts {
        lines.push(format!("- {value}: {count}"));
    }
}

fn validate_element_provenance(element: &ModelElement) -> Result<()> {
    validate_element_provenance_for_id(&element.id, &element.provenance)
}

fn validate_element_provenance_for_id(id: &str, provenance: &ElementProvenance) -> Result<()> {
    for source_ref in &provenance.source_refs {
        ensure_non_empty(source_ref, &format!("{id} provenance sourceRef"))?;
    }
    if let Some(created_by) = &provenance.created_by {
        ensure_non_empty(created_by, &format!("{id} provenance createdBy"))?;
    }
    if let Some(created_at) = &provenance.created_at {
        ensure_non_empty(created_at, &format!("{id} provenance createdAt"))?;
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

fn validate_architecture_details(element: &ModelElement) -> Result<()> {
    let architecture = &element.architecture;

    validate_optional_enum(
        &architecture.criticality,
        &["low", "medium", "high", "critical"],
        &format!("{} architecture criticality", element.id),
    )?;

    let mut owner_refs = BTreeSet::new();
    for owner in &architecture.owners {
        ensure_unique(&mut owner_refs, owner.ref_id.as_str())?;
        validate_optional_enum(
            &owner.role,
            &[
                "accountable",
                "responsible",
                "technical",
                "business",
                "support",
            ],
            &format!("{} architecture owner {} role", element.id, owner.ref_id),
        )?;
        ensure_non_empty(
            &owner.ref_id,
            &format!("{} architecture owner ref", element.id),
        )?;
        if !owner.name.is_empty() {
            ensure_non_empty(
                &owner.name,
                &format!("{} architecture owner {} name", element.id, owner.ref_id),
            )?;
        }
    }

    if let Some(lifecycle) = &architecture.lifecycle {
        validate_optional_enum(
            &lifecycle.state,
            &[
                "idea",
                "planned",
                "active",
                "deprecated",
                "retiring",
                "retired",
            ],
            &format!("{} architecture lifecycle state", element.id),
        )?;
        validate_string_list(
            &lifecycle.milestone_refs,
            &format!("{} architecture lifecycle milestoneRef", element.id),
        )?;
        if !lifecycle.phase.is_empty() {
            ensure_non_empty(
                &lifecycle.phase,
                &format!("{} architecture lifecycle phase", element.id),
            )?;
        }
        if !lifecycle.target_date.is_empty() {
            ensure_non_empty(
                &lifecycle.target_date,
                &format!("{} architecture lifecycle targetDate", element.id),
            )?;
        }
    }

    let mut technology_refs = BTreeSet::new();
    for technology in &architecture.technologies {
        ensure_unique(&mut technology_refs, technology.ref_id.as_str())?;
        ensure_non_empty(
            &technology.ref_id,
            &format!("{} architecture technology ref", element.id),
        )?;
        validate_optional_enum(
            &technology.role,
            &[
                "platform",
                "runtime",
                "framework",
                "database",
                "protocol",
                "tool",
                "standard",
            ],
            &format!(
                "{} architecture technology {} role",
                element.id, technology.ref_id
            ),
        )?;
        validate_optional_enum(
            &technology.standard_state,
            &["approved", "tolerated", "discouraged", "banned", "emerging"],
            &format!(
                "{} architecture technology {} standardState",
                element.id, technology.ref_id
            ),
        )?;
    }

    let mut risk_refs = BTreeSet::new();
    for risk in &architecture.risks {
        ensure_unique(&mut risk_refs, risk.ref_id.as_str())?;
        ensure_non_empty(
            &risk.ref_id,
            &format!("{} architecture risk ref", element.id),
        )?;
        validate_optional_enum(
            &risk.severity,
            &["low", "medium", "high", "critical"],
            &format!("{} architecture risk {} severity", element.id, risk.ref_id),
        )?;
        validate_optional_enum(
            &risk.status,
            &[
                "identified",
                "accepted",
                "mitigating",
                "mitigated",
                "closed",
            ],
            &format!("{} architecture risk {} status", element.id, risk.ref_id),
        )?;
    }

    let mut capability_refs = BTreeSet::new();
    for capability in &architecture.capabilities {
        ensure_unique(&mut capability_refs, capability.ref_id.as_str())?;
        ensure_non_empty(
            &capability.ref_id,
            &format!("{} architecture capability ref", element.id),
        )?;
        validate_optional_enum(
            &capability.fit,
            &["primary", "supporting", "enabling", "impacted"],
            &format!(
                "{} architecture capability {} fit",
                element.id, capability.ref_id
            ),
        )?;
        validate_optional_enum(
            &capability.maturity,
            &[
                "emerging",
                "developing",
                "established",
                "optimized",
                "legacy",
            ],
            &format!(
                "{} architecture capability {} maturity",
                element.id, capability.ref_id
            ),
        )?;
    }

    let mut service_refs = BTreeSet::new();
    for service in &architecture.services {
        ensure_unique(&mut service_refs, service.ref_id.as_str())?;
        ensure_non_empty(
            &service.ref_id,
            &format!("{} architecture service ref", element.id),
        )?;
        validate_optional_enum(
            &service.relationship,
            &["provides", "consumes", "depends_on", "exposes", "supports"],
            &format!(
                "{} architecture service {} relationship",
                element.id, service.ref_id
            ),
        )?;
        if !service.interface_ref.is_empty() {
            ensure_non_empty(
                &service.interface_ref,
                &format!(
                    "{} architecture service {} interfaceRef",
                    element.id, service.ref_id
                ),
            )?;
        }
    }

    Ok(())
}

fn validate_classifier_details(element: &ModelElement) -> Result<()> {
    let Some(classifier) = &element.classifier else {
        return Ok(());
    };
    if !matches!(element.kind.as_str(), "class" | "component") {
        bail!(
            "{} has classifier details but kind {}",
            element.id,
            element.kind
        );
    }

    let mut attribute_names = BTreeSet::new();
    for attribute in &classifier.attributes {
        ensure_unique(&mut attribute_names, attribute.name.as_str())?;
        validate_visibility(
            &attribute.visibility,
            &format!("{} attribute {} visibility", element.id, attribute.name),
        )?;
        validate_optional_type_ref(
            &attribute.type_ref,
            &format!("{} attribute {} typeRef", element.id, attribute.name),
        )?;
        validate_multiplicity(
            attribute.multiplicity.as_ref(),
            &format!("{} attribute {} multiplicity", element.id, attribute.name),
        )?;
    }

    for operation in &classifier.operations {
        ensure_non_empty(&operation.name, &format!("{} operation name", element.id))?;
        validate_visibility(
            &operation.visibility,
            &format!("{} operation {} visibility", element.id, operation.name),
        )?;
        validate_optional_type_ref(
            &operation.return_type_ref,
            &format!("{} operation {} returnTypeRef", element.id, operation.name),
        )?;
        let mut parameter_names = BTreeSet::new();
        for parameter in &operation.parameters {
            ensure_unique(&mut parameter_names, parameter.name.as_str())?;
            validate_optional_type_ref(
                &parameter.type_ref,
                &format!(
                    "{} operation {} parameter {} typeRef",
                    element.id, operation.name, parameter.name
                ),
            )?;
            if !parameter.direction.is_empty()
                && !matches!(
                    parameter.direction.as_str(),
                    "in" | "out" | "inout" | "return"
                )
            {
                bail!(
                    "{} operation {} parameter {} has unsupported direction {}",
                    element.id,
                    operation.name,
                    parameter.name,
                    parameter.direction
                );
            }
            validate_multiplicity(
                parameter.multiplicity.as_ref(),
                &format!(
                    "{} operation {} parameter {} multiplicity",
                    element.id, operation.name, parameter.name
                ),
            )?;
        }
    }

    Ok(())
}

fn validate_visibility(visibility: &str, field: &str) -> Result<()> {
    if visibility.is_empty() {
        return Ok(());
    }
    if !matches!(visibility, "public" | "private" | "protected" | "package") {
        bail!("{field} has unsupported value {visibility}");
    }
    Ok(())
}

fn validate_optional_type_ref(type_ref: &str, field: &str) -> Result<()> {
    if !type_ref.is_empty() {
        ensure_non_empty(type_ref, field)?;
    }
    Ok(())
}

fn validate_multiplicity(multiplicity: Option<&Multiplicity>, field: &str) -> Result<()> {
    let Some(multiplicity) = multiplicity else {
        return Ok(());
    };
    if let Some(MultiplicityUpper::Unbounded(upper)) = &multiplicity.upper {
        if upper != "*" {
            bail!("{field} upper must be a non-negative integer or *");
        }
    }
    if let (Some(lower), Some(MultiplicityUpper::Count(upper))) =
        (multiplicity.lower, &multiplicity.upper)
    {
        if lower > *upper {
            bail!("{field} lower bound {lower} exceeds upper bound {upper}");
        }
    }
    Ok(())
}

fn validate_specialized_element_details(element: &ModelElement) -> Result<()> {
    validate_detail_kind(
        element.actor_details.is_some(),
        element,
        "actorDetails",
        "actor",
    )?;
    validate_detail_kind(
        element.use_case_details.is_some(),
        element,
        "useCaseDetails",
        "use_case",
    )?;
    validate_detail_kind(
        element.activity_details.is_some(),
        element,
        "activityDetails",
        "activity",
    )?;
    validate_detail_kind(
        element.sequence_participant_details.is_some(),
        element,
        "sequenceParticipantDetails",
        "sequence_participant",
    )?;

    if let Some(details) = &element.actor_details {
        validate_optional_enum(
            &details.actor_type,
            &[
                "person",
                "role",
                "organization",
                "system",
                "external_system",
            ],
            &format!("{} actorDetails actorType", element.id),
        )?;
        validate_string_list(
            &details.responsibilities,
            &format!("{} actorDetails responsibility", element.id),
        )?;
        validate_string_list(&details.goals, &format!("{} actorDetails goal", element.id))?;
        validate_string_list(
            &details.constraints,
            &format!("{} actorDetails constraint", element.id),
        )?;
    }

    if let Some(details) = &element.use_case_details {
        validate_optional_type_ref(
            &details.primary_actor_ref,
            &format!("{} useCaseDetails primaryActorRef", element.id),
        )?;
        validate_string_list(
            &details.supporting_actor_refs,
            &format!("{} useCaseDetails supportingActorRef", element.id),
        )?;
        validate_string_list(
            &details.preconditions,
            &format!("{} useCaseDetails precondition", element.id),
        )?;
        validate_string_list(
            &details.postconditions,
            &format!("{} useCaseDetails postcondition", element.id),
        )?;
        validate_use_case_steps(
            &details.main_flow,
            &format!("{} useCaseDetails mainFlow", element.id),
        )?;
        for flow in &details.alternate_flows {
            ensure_non_empty(
                &flow.name,
                &format!("{} useCaseDetails alternateFlow name", element.id),
            )?;
            validate_optional_type_ref(
                &flow.trigger,
                &format!(
                    "{} useCaseDetails alternateFlow {} trigger",
                    element.id, flow.name
                ),
            )?;
            validate_use_case_steps(
                &flow.steps,
                &format!(
                    "{} useCaseDetails alternateFlow {} steps",
                    element.id, flow.name
                ),
            )?;
        }
        validate_string_list(
            &details.extension_points,
            &format!("{} useCaseDetails extensionPoint", element.id),
        )?;
    }

    if let Some(details) = &element.activity_details {
        let mut parameter_names = BTreeSet::new();
        for parameter in &details.parameters {
            ensure_unique(&mut parameter_names, parameter.name.as_str())?;
            validate_optional_type_ref(
                &parameter.type_ref,
                &format!(
                    "{} activityDetails parameter {} typeRef",
                    element.id, parameter.name
                ),
            )?;
            validate_optional_enum(
                &parameter.direction,
                &["in", "out", "inout"],
                &format!(
                    "{} activityDetails parameter {} direction",
                    element.id, parameter.name
                ),
            )?;
        }

        let mut node_ids = BTreeSet::new();
        for node in &details.nodes {
            ensure_unique(&mut node_ids, node.id.as_str())?;
            ensure_non_empty(
                &node.name,
                &format!("{} activityDetails node {} name", element.id, node.id),
            )?;
            validate_required_enum(
                &node.kind,
                &[
                    "initial", "action", "decision", "merge", "fork", "join", "object", "final",
                ],
                &format!("{} activityDetails node {} kind", element.id, node.id),
            )?;
        }

        let mut flow_ids = BTreeSet::new();
        for flow in &details.flows {
            ensure_unique(&mut flow_ids, flow.id.as_str())?;
            if !node_ids.contains(flow.source_node_id.as_str()) {
                bail!(
                    "{} activityDetails flow {} references missing source node {}",
                    element.id,
                    flow.id,
                    flow.source_node_id
                );
            }
            if !node_ids.contains(flow.target_node_id.as_str()) {
                bail!(
                    "{} activityDetails flow {} references missing target node {}",
                    element.id,
                    flow.id,
                    flow.target_node_id
                );
            }
        }
    }

    if let Some(details) = &element.sequence_participant_details {
        validate_optional_enum(
            &details.participant_kind,
            &["actor", "component", "class", "service", "external_system"],
            &format!("{} sequenceParticipantDetails participantKind", element.id),
        )?;
        validate_optional_type_ref(
            &details.represents_ref,
            &format!("{} sequenceParticipantDetails representsRef", element.id),
        )?;
        validate_optional_type_ref(
            &details.lifeline_name,
            &format!("{} sequenceParticipantDetails lifelineName", element.id),
        )?;
    }

    Ok(())
}

fn validate_detail_kind(
    present: bool,
    element: &ModelElement,
    details_field: &str,
    expected_kind: &str,
) -> Result<()> {
    if present && element.kind != expected_kind {
        bail!(
            "{} has {} but kind {}",
            element.id,
            details_field,
            element.kind
        );
    }
    Ok(())
}

fn validate_use_case_steps(steps: &[UseCaseStep], field: &str) -> Result<()> {
    let mut step_numbers = BTreeSet::new();
    for step in steps {
        if step.step == 0 {
            bail!("{field} step number must be greater than zero");
        }
        if !step_numbers.insert(step.step) {
            bail!("{field} has duplicate step {}", step.step);
        }
        validate_optional_type_ref(
            &step.actor_ref,
            &format!("{field} step {} actorRef", step.step),
        )?;
        ensure_non_empty(&step.action, &format!("{field} step {} action", step.step))?;
    }
    Ok(())
}

fn validate_string_list(values: &[String], field: &str) -> Result<()> {
    for value in values {
        ensure_non_empty(value, field)?;
    }
    Ok(())
}

fn validate_optional_enum(value: &str, allowed: &[&str], field: &str) -> Result<()> {
    if value.is_empty() {
        return Ok(());
    }
    validate_required_enum(value, allowed, field)
}

fn validate_required_enum(value: &str, allowed: &[&str], field: &str) -> Result<()> {
    ensure_non_empty(value, field)?;
    if !allowed.contains(&value) {
        bail!("{field} has unsupported value {value}");
    }
    Ok(())
}

fn validate_diagram_layout(
    diagram: &DiagramView,
    layout: &DiagramLayout,
    element_kinds: &BTreeMap<&str, &str>,
    portfolio_ids: &BTreeSet<&str>,
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

    let diagram_refs: BTreeSet<&str> = diagram
        .model_refs
        .iter()
        .chain(diagram.portfolio_refs.iter())
        .map(String::as_str)
        .collect();
    let mut node_refs = BTreeSet::new();
    for node in &layout.nodes {
        ensure_unique(&mut node_refs, &node.model_ref)?;
        if !element_kinds.contains_key(node.model_ref.as_str())
            && !portfolio_ids.contains(node.model_ref.as_str())
        {
            bail!(
                "{} layout references missing model or portfolio object {}",
                diagram.id,
                node.model_ref
            );
        }
        if !diagram_refs.contains(node.model_ref.as_str()) {
            bail!(
                "{} layout node {} is not in modelRefs or portfolioRefs",
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
            "create_portfolio_object" => require_args(&operation.args, &["id", "kind", "name"])?,
            "update_portfolio_object" => {
                require_args(&operation.args, &["objectId"])?;
                require_any_update_arg(&operation.args, "update_portfolio_object")?;
            }
            "create_portfolio_saved_view" => {
                require_args(&operation.args, &["id", "title", "scope"])?
            }
            "update_portfolio_saved_view" => {
                require_args(&operation.args, &["viewId"])?;
                require_any_update_arg(&operation.args, "update_portfolio_saved_view")?;
            }
            "remove_portfolio_saved_view" => require_args(&operation.args, &["viewId"])?,
            "create_roadmap_presentation" => {
                require_args(&operation.args, &["id", "title", "appliesToViewKinds"])?
            }
            "update_roadmap_presentation" => {
                require_args(&operation.args, &["presentationId"])?;
                require_any_update_arg(&operation.args, "update_roadmap_presentation")?;
            }
            "remove_roadmap_presentation" => require_args(&operation.args, &["presentationId"])?,
            "assign_roadmap_presentation" => {
                require_args(&operation.args, &["diagramId", "presentationId"])?
            }
            "create_model_element" => require_args(&operation.args, &["id", "kind", "name"])?,
            "update_model_element_details" => {
                require_args(&operation.args, &["elementId"])?;
                require_any_update_arg(&operation.args, "update_model_element_details")?;
            }
            "create_relationship" => require_args(
                &operation.args,
                &["id", "relationshipKind", "sourceId", "targetId"],
            )?,
            "create_diagram_view" => require_args(&operation.args, &["id", "title", "viewKind"])?,
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

pub fn render_lifecycle_roadmap_svg(
    package: &ModelPackage,
    diagram_id: Option<&str>,
) -> Result<String> {
    render_lifecycle_roadmap_svg_with_presentation(package, diagram_id, None)
}

pub fn render_lifecycle_roadmap_svg_with_presentation(
    package: &ModelPackage,
    diagram_id: Option<&str>,
    presentation_id: Option<&str>,
) -> Result<String> {
    render_dot_to_svg(&render_lifecycle_roadmap_dot_with_presentation(
        package,
        diagram_id,
        presentation_id,
    )?)
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

pub fn render_lifecycle_roadmap_dot(
    package: &ModelPackage,
    diagram_id: Option<&str>,
) -> Result<String> {
    render_lifecycle_roadmap_dot_with_presentation(package, diagram_id, None)
}

pub fn render_lifecycle_roadmap_dot_with_presentation(
    package: &ModelPackage,
    diagram_id: Option<&str>,
    presentation_id: Option<&str>,
) -> Result<String> {
    let diagram = find_lifecycle_roadmap_diagram(package, diagram_id)?;
    let presentation = resolve_roadmap_presentation(package, diagram, presentation_id)?;
    let portfolio: BTreeMap<&str, &PortfolioObject> = package
        .portfolio
        .objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect();
    let objects: Vec<&PortfolioObject> = diagram
        .portfolio_refs
        .iter()
        .filter_map(|id| portfolio.get(id.as_str()).copied())
        .filter(|object| include_roadmap_object(object, &diagram.portfolio_refs, presentation))
        .collect();
    let nodes = lifecycle_roadmap_nodes(&objects, presentation);
    let timeline_buckets = lifecycle_timeline_buckets(&nodes, presentation);
    let (nodesep, ranksep) = roadmap_spacing(presentation);
    let show_timeline_scale = presentation
        .and_then(|presentation| presentation.styling.show_timeline_scale)
        .unwrap_or(true);

    let mut dot = String::new();
    writeln!(
        dot,
        "digraph {} {{",
        dot_id(diagram.id.strip_prefix("diagram.").unwrap_or(&diagram.id))
    )?;
    writeln!(
        dot,
        "  graph [rankdir=LR, bgcolor=\"{}\", pad=\"0.35\", nodesep=\"{}\", ranksep=\"{}\", label=\"{}\", labelloc=t, fontsize=20, fontname=\"Inter, Arial, sans-serif\", fontcolor=\"{}\", compound=true]",
        "#f8fafc",
        nodesep,
        ranksep,
        escape_dot_label(&diagram.title),
        "#0f172a"
    )?;
    writeln!(
        dot,
        "  node [fontname=\"Inter, Arial, sans-serif\", fontsize=11, style=\"rounded,filled\", color=\"{}\", fontcolor=\"{}\"]",
        "#334155", "#0f172a"
    )?;
    writeln!(
        dot,
        "  edge [fontname=\"Inter, Arial, sans-serif\", fontsize=10, color=\"{}\", fontcolor=\"{}\", arrowsize=0.75]",
        "#64748b", "#475569"
    )?;

    if show_timeline_scale {
        writeln!(
            dot,
            "  subgraph cluster_timeline {{\n    label=\"Timeline scale\"\n    color=\"{}\"\n    fillcolor=\"{}\"\n    style=\"rounded,filled\"\n    fontname=\"Inter, Arial, sans-serif\"\n    fontsize=13\n    fontcolor=\"{}\"",
            "#cbd5e1", "#f8fafc", "#334155"
        )?;
        for bucket in &timeline_buckets {
            writeln!(
                dot,
                "    {} [id=\"{}\", label=\"{}\", shape=plain, fontcolor=\"{}\", tooltip=\"{}\"]",
                timeline_bucket_node_id(bucket),
                escape_dot_label(&format!("timeline.{}", bucket.key)),
                escape_dot_label(&bucket.label),
                "#475569",
                escape_dot_label(&bucket.label)
            )?;
        }
        for pair in timeline_buckets.windows(2) {
            writeln!(
                dot,
                "    {} -> {} [style=invis, weight=8]",
                timeline_bucket_node_id(&pair[0]),
                timeline_bucket_node_id(&pair[1])
            )?;
        }
        writeln!(dot, "  }}")?;
    }

    for lane in lifecycle_swimlanes(&nodes) {
        let lane_nodes: Vec<&RoadmapNode<'_>> = nodes
            .iter()
            .filter(|node| node.swimlane_key == lane.key)
            .collect();
        if lane_nodes.is_empty() {
            continue;
        }
        writeln!(
            dot,
            "  subgraph cluster_lane_{} {{\n    label=\"{}\"\n    color=\"{}\"\n    fillcolor=\"{}\"\n    style=\"rounded,filled\"\n    fontname=\"Inter, Arial, sans-serif\"\n    fontsize=14\n    fontcolor=\"{}\"",
            dot_id(&lane.key),
            escape_dot_label(&lane.label),
            "#cbd5e1",
            "#ffffff",
            "#334155"
        )?;
        for node in lane_nodes {
            let object = node.object;
            let shape = if object.kind == "lifecycle_milestone" {
                "diamond"
            } else {
                "box"
            };
            writeln!(
                dot,
                "    {} [id=\"{}\", label=\"{}\", shape={}, fillcolor=\"{}\", color=\"{}\", tooltip=\"{}\"]",
                dot_id(&object.id),
                escape_dot_label(&object.id),
                escape_dot_label(&portfolio_roadmap_label(object)),
                shape,
                roadmap_fill_color(object, presentation),
                roadmap_border_color(object, presentation),
                escape_dot_label(&object.id)
            )?;
            if let Some(target) = target_transition_label(object, presentation) {
                writeln!(
                    dot,
                    "    {} [id=\"{}\", label=\"{}\", shape=note, fillcolor=\"{}\", color=\"{}\", fontcolor=\"{}\", tooltip=\"{}\"]",
                    target_node_id(object),
                    escape_dot_label(&format!("{}.target", object.id)),
                    escape_dot_label(&target),
                    "#eef2ff",
                    "#4f46e5",
                    "#312e81",
                    escape_dot_label(&target)
                )?;
            }
        }
        writeln!(dot, "  }}")?;
    }

    for node in &nodes {
        if show_timeline_scale
            && let Some(bucket) = timeline_buckets
                .iter()
                .find(|bucket| bucket.key == node.timeline_key)
        {
            writeln!(
                dot,
                "  {} -> {} [style=invis, weight=3]",
                timeline_bucket_node_id(bucket),
                dot_id(&node.object.id)
            )?;
        }
        if target_transition_label(node.object, presentation).is_some() {
            writeln!(
                dot,
                "  {} -> {} [id=\"{}\", label=\"target state\", color=\"{}\", fontcolor=\"{}\"]",
                dot_id(&node.object.id),
                target_node_id(node.object),
                escape_dot_label(&format!("{}.target-state", node.object.id)),
                "#4f46e5",
                "#3730a3"
            )?;
        }
    }

    if roadmap_show_milestone_links(presentation) {
        let milestone_link_style = roadmap_milestone_link_style(presentation);
        let included: BTreeSet<&str> = objects.iter().map(|object| object.id.as_str()).collect();
        for object in &objects {
            let Some(lifecycle) = &object.lifecycle else {
                continue;
            };
            for milestone_ref in &lifecycle.milestone_refs {
                if included.contains(milestone_ref.as_str()) {
                    writeln!(
                        dot,
                        "  {} -> {} [id=\"{}\", label=\"milestone\", style={}]",
                        dot_id(&object.id),
                        dot_id(milestone_ref),
                        escape_dot_label(&format!("{}.{}", object.id, milestone_ref)),
                        milestone_link_style
                    )?;
                }
            }
        }
    }

    if presentation
        .and_then(|presentation| presentation.styling.show_legend)
        .unwrap_or(false)
    {
        let color_by = presentation
            .map(roadmap_color_by)
            .unwrap_or("lifecycleState");
        writeln!(
            dot,
            "  roadmap_legend [id=\"roadmap.legend\", label=\"Color by: {}\", shape=note, fillcolor=\"{}\", color=\"{}\", fontcolor=\"{}\"]",
            escape_dot_label(&format_object_label(color_by)),
            "#f8fafc",
            "#64748b",
            "#334155"
        )?;
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

fn find_lifecycle_roadmap_diagram<'a>(
    package: &'a ModelPackage,
    diagram_id: Option<&str>,
) -> Result<&'a DiagramView> {
    match diagram_id {
        Some(id) => package
            .diagrams
            .diagrams
            .iter()
            .find(|diagram| diagram.id == id && diagram.view_kind == "lifecycle_roadmap"),
        None => package
            .diagrams
            .diagrams
            .iter()
            .find(|diagram| diagram.view_kind == "lifecycle_roadmap"),
    }
    .ok_or_else(|| anyhow!("no matching lifecycle roadmap diagram found"))
}

fn resolve_roadmap_presentation<'a>(
    package: &'a ModelPackage,
    diagram: &DiagramView,
    presentation_id: Option<&str>,
) -> Result<Option<&'a RoadmapPresentation>> {
    let selected_id = presentation_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            if diagram.roadmap_presentation_ref.is_empty() {
                None
            } else {
                Some(diagram.roadmap_presentation_ref.as_str())
            }
        });
    let Some(selected_id) = selected_id else {
        return Ok(None);
    };
    let presentation = package
        .roadmap_presentations
        .presentations
        .iter()
        .find(|presentation| presentation.id == selected_id)
        .ok_or_else(|| anyhow!("missing roadmap presentation {selected_id}"))?;
    if !roadmap_presentation_applies_to(presentation, &diagram.view_kind) {
        bail!(
            "{} does not apply to {}",
            presentation.id,
            diagram.view_kind
        );
    }
    Ok(Some(presentation))
}

fn lifecycle_bucket(object: &PortfolioObject) -> &str {
    if object.lifecycle_state.is_empty() {
        "unspecified"
    } else {
        object.lifecycle_state.as_str()
    }
}

fn format_lifecycle_state(state: &str) -> String {
    if state == "unspecified" {
        return "Unspecified".to_string();
    }
    let mut chars = state.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

struct RoadmapNode<'a> {
    object: &'a PortfolioObject,
    swimlane_key: String,
    timeline_key: String,
}

struct TimelineBucket {
    key: String,
    label: String,
}

struct Swimlane {
    key: String,
    label: String,
}

fn lifecycle_roadmap_nodes<'a>(
    objects: &[&'a PortfolioObject],
    presentation: Option<&RoadmapPresentation>,
) -> Vec<RoadmapNode<'a>> {
    let mut nodes: Vec<RoadmapNode<'a>> = objects
        .iter()
        .copied()
        .map(|object| RoadmapNode {
            object,
            swimlane_key: lifecycle_swimlane_key(object, presentation),
            timeline_key: lifecycle_timeline_key(object, presentation),
        })
        .collect();
    nodes.sort_by(|left, right| {
        (
            left.swimlane_key.as_str(),
            left.timeline_key.as_str(),
            left.object.id.as_str(),
        )
            .cmp(&(
                right.swimlane_key.as_str(),
                right.timeline_key.as_str(),
                right.object.id.as_str(),
            ))
    });
    nodes
}

fn lifecycle_timeline_buckets(
    nodes: &[RoadmapNode<'_>],
    presentation: Option<&RoadmapPresentation>,
) -> Vec<TimelineBucket> {
    let mut keys: BTreeSet<String> = nodes.iter().map(|node| node.timeline_key.clone()).collect();
    if keys.is_empty() {
        keys.insert("0000-current".to_string());
    }
    keys.into_iter()
        .map(|key| TimelineBucket {
            label: format_timeline_key(&key, presentation),
            key,
        })
        .collect()
}

fn lifecycle_swimlanes(nodes: &[RoadmapNode<'_>]) -> Vec<Swimlane> {
    let keys: BTreeSet<String> = nodes.iter().map(|node| node.swimlane_key.clone()).collect();
    keys.into_iter()
        .map(|key| Swimlane {
            label: format_swimlane_key(&key),
            key,
        })
        .collect()
}

fn lifecycle_swimlane_key(
    object: &PortfolioObject,
    presentation: Option<&RoadmapPresentation>,
) -> String {
    match presentation
        .map(roadmap_swimlane_group_by)
        .unwrap_or("portfolioKind")
    {
        "none" => "10.all".to_string(),
        "lifecycleState" => format!("20.{}", lifecycle_bucket(object).replace('_', "-")),
        "criticality" => format!(
            "30.{}",
            if object.criticality.is_empty() {
                "unspecified"
            } else {
                object.criticality.as_str()
            }
        ),
        "owner" => keyed_ref_lane("40", &object.owner_refs),
        "capability" => keyed_ref_lane("50", &object.capability_refs),
        "technology" => keyed_ref_lane("60", &object.technology_refs),
        _ => match object.kind.as_str() {
            "portfolio_application" => "10.application".to_string(),
            "portfolio_service" => "20.service".to_string(),
            "technology_component" => "30.technology-component".to_string(),
            "technology_standard" => "40.technology-standard".to_string(),
            "lifecycle_milestone" => "90.milestone".to_string(),
            other => format!("80.{}", other.replace('_', "-")),
        },
    }
}

fn format_swimlane_key(key: &str) -> String {
    if key == "10.all" {
        return "All roadmap items".to_string();
    }
    let (_, label) = key.split_once('.').unwrap_or(("80", key));
    format!(
        "{} swimlane",
        label
            .split('-')
            .map(format_lifecycle_state)
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn keyed_ref_lane(prefix: &str, refs: &[String]) -> String {
    refs.first()
        .map(|reference| format!("{}.{}", prefix, reference.replace('_', "-")))
        .unwrap_or_else(|| format!("{prefix}.unassigned"))
}

fn lifecycle_timeline_key(
    object: &PortfolioObject,
    presentation: Option<&RoadmapPresentation>,
) -> String {
    let date = roadmap_date_for_object(object, presentation).unwrap_or_else(|| {
        if lifecycle_bucket(object) == "active" {
            "0000-current".to_string()
        } else {
            "9999-unscheduled".to_string()
        }
    });
    if date == "0000-current" || date == "9999-unscheduled" {
        return date;
    }
    let mut parts = date.split('-');
    let year = parts.next().unwrap_or("9999");
    let month = parts
        .next()
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(12);
    match presentation
        .map(roadmap_bucket_granularity)
        .unwrap_or("quarter")
    {
        "month" => format!("{}-m{:02}", year, month),
        "half_year" => {
            let half = if month <= 6 { 1 } else { 2 };
            format!("{}-h{}", year, half)
        }
        "year" => year.to_string(),
        _ => {
            let quarter = ((month.saturating_sub(1)) / 3) + 1;
            format!("{}-q{}", year, quarter)
        }
    }
}

fn primary_lifecycle_date(lifecycle: &PortfolioLifecycle) -> Option<String> {
    [
        lifecycle.target_date.as_str(),
        lifecycle.retirement_date.as_str(),
        lifecycle.end_of_support_date.as_str(),
        lifecycle.current_from.as_str(),
    ]
    .iter()
    .find(|date| !date.is_empty())
    .map(|date| (*date).to_string())
}

fn roadmap_date_for_object(
    object: &PortfolioObject,
    presentation: Option<&RoadmapPresentation>,
) -> Option<String> {
    let lifecycle = object.lifecycle.as_ref()?;
    match presentation.map(roadmap_bucket_source).unwrap_or("auto") {
        "targetDate" => non_empty_string(&lifecycle.target_date),
        "retirementDate" => non_empty_string(&lifecycle.retirement_date),
        "endOfSupportDate" => non_empty_string(&lifecycle.end_of_support_date),
        "currentFrom" => non_empty_string(&lifecycle.current_from),
        _ => primary_lifecycle_date(lifecycle),
    }
}

fn non_empty_string(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn include_roadmap_object(
    object: &PortfolioObject,
    _diagram_portfolio_refs: &[String],
    presentation: Option<&RoadmapPresentation>,
) -> bool {
    if object.kind == "lifecycle_milestone" {
        if !roadmap_show_milestone_nodes(presentation) {
            return false;
        }
    }

    let Some(date) = roadmap_date_for_object(object, presentation) else {
        return presentation
            .and_then(|presentation| presentation.timeline.include_undated_bucket)
            .unwrap_or(true);
    };
    let Some(presentation) = presentation else {
        return true;
    };
    if !presentation.timeline.range_start.is_empty()
        && date.as_str() < presentation.timeline.range_start.as_str()
    {
        return false;
    }
    if !presentation.timeline.range_end.is_empty()
        && date.as_str() > presentation.timeline.range_end.as_str()
    {
        return false;
    }
    true
}

fn roadmap_bucket_source(presentation: &RoadmapPresentation) -> &str {
    if presentation.timeline.bucket_source.is_empty() {
        "auto"
    } else {
        presentation.timeline.bucket_source.as_str()
    }
}

fn roadmap_bucket_granularity(presentation: &RoadmapPresentation) -> &str {
    if presentation.timeline.bucket_granularity.is_empty() {
        "quarter"
    } else {
        presentation.timeline.bucket_granularity.as_str()
    }
}

fn roadmap_date_label_format(presentation: &RoadmapPresentation) -> &str {
    if presentation.timeline.date_label_format.is_empty() {
        roadmap_bucket_granularity(presentation)
    } else {
        presentation.timeline.date_label_format.as_str()
    }
}

fn roadmap_swimlane_group_by(presentation: &RoadmapPresentation) -> &str {
    if presentation.swimlanes.group_by.is_empty() {
        "portfolioKind"
    } else {
        presentation.swimlanes.group_by.as_str()
    }
}

fn roadmap_show_target_callouts(presentation: Option<&RoadmapPresentation>) -> bool {
    presentation
        .and_then(|presentation| presentation.target_states.show_callouts)
        .unwrap_or(true)
}

fn roadmap_show_target_dates(presentation: Option<&RoadmapPresentation>) -> bool {
    presentation
        .and_then(|presentation| presentation.target_states.show_target_dates)
        .unwrap_or(true)
}

fn roadmap_show_milestone_nodes(presentation: Option<&RoadmapPresentation>) -> bool {
    presentation
        .and_then(|presentation| presentation.milestones.show_milestone_nodes)
        .unwrap_or(true)
}

fn roadmap_show_milestone_links(presentation: Option<&RoadmapPresentation>) -> bool {
    presentation
        .and_then(|presentation| presentation.milestones.show_milestone_links)
        .unwrap_or(true)
}

fn roadmap_milestone_link_style(presentation: Option<&RoadmapPresentation>) -> &str {
    presentation
        .map(|presentation| presentation.milestones.link_style.as_str())
        .filter(|style| !style.is_empty())
        .unwrap_or("dashed")
}

fn roadmap_color_by(presentation: &RoadmapPresentation) -> &str {
    if presentation.styling.color_by.is_empty() {
        "lifecycleState"
    } else {
        presentation.styling.color_by.as_str()
    }
}

fn roadmap_spacing(presentation: Option<&RoadmapPresentation>) -> (&'static str, &'static str) {
    match presentation
        .map(|presentation| presentation.styling.density.as_str())
        .unwrap_or("comfortable")
    {
        "compact" => ("0.35", "0.75"),
        "detailed" => ("0.75", "1.25"),
        _ => ("0.55", "1.0"),
    }
}

fn format_object_label(value: &str) -> String {
    value
        .split(|character: char| character == '_' || character == '-')
        .map(format_lifecycle_state)
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_timeline_key(key: &str, presentation: Option<&RoadmapPresentation>) -> String {
    if key == "0000-current" {
        return "Current".to_string();
    }
    if key == "9999-unscheduled" {
        return "Unscheduled".to_string();
    }
    if let Some((year, month)) = key.split_once("-m") {
        return match presentation
            .map(roadmap_date_label_format)
            .unwrap_or("quarter")
        {
            "date" | "month" => format!("{year}-{month}"),
            "year" => year.to_string(),
            _ => {
                let month = month.parse::<u8>().unwrap_or(12);
                let quarter = ((month.saturating_sub(1)) / 3) + 1;
                format!("{year} Q{quarter}")
            }
        };
    }
    if let Some((year, quarter)) = key.split_once("-q") {
        return format!("{year} Q{quarter}");
    }
    if let Some((year, half)) = key.split_once("-h") {
        return format!("{year} H{half}");
    }
    key.to_string()
}

fn timeline_bucket_node_id(bucket: &TimelineBucket) -> String {
    dot_id(&format!("timeline.{}", bucket.key))
}

fn target_node_id(object: &PortfolioObject) -> String {
    dot_id(&format!("{}.target", object.id))
}

fn target_transition_label(
    object: &PortfolioObject,
    presentation: Option<&RoadmapPresentation>,
) -> Option<String> {
    if !roadmap_show_target_callouts(presentation) {
        return None;
    }
    let lifecycle = object.lifecycle.as_ref()?;
    if lifecycle.target_state.is_empty() {
        return None;
    }
    if let Some(presentation) = presentation
        && !presentation.target_states.states.is_empty()
        && !presentation
            .target_states
            .states
            .iter()
            .any(|state| state == &lifecycle.target_state)
    {
        return None;
    }
    let current = if lifecycle.state.is_empty() {
        lifecycle_bucket(object)
    } else {
        lifecycle.state.as_str()
    };
    if lifecycle.target_state == current
        && !presentation
            .and_then(|presentation| presentation.target_states.show_no_change_targets)
            .unwrap_or(false)
    {
        return None;
    }
    let mut label = format!(
        "Target: {}",
        format_lifecycle_state(lifecycle.target_state.as_str())
    );
    if roadmap_show_target_dates(presentation) && !lifecycle.target_date.is_empty() {
        label.push('\n');
        label.push_str(&lifecycle.target_date);
    }
    Some(label)
}

fn portfolio_roadmap_label(object: &PortfolioObject) -> String {
    let target = object
        .lifecycle
        .as_ref()
        .and_then(|lifecycle| {
            if lifecycle.target_date.is_empty() {
                None
            } else {
                Some(lifecycle.target_date.as_str())
            }
        })
        .unwrap_or("no target date");
    format!(
        "{}\n{}\n{}",
        object.name,
        object.kind.replace('_', " "),
        target
    )
}

fn lifecycle_fill_color(object: &PortfolioObject) -> &'static str {
    match lifecycle_bucket(object) {
        "idea" => "#f8fafc",
        "planned" => "#eff6ff",
        "active" => "#ecfdf5",
        "deprecated" => "#fff7ed",
        "retiring" => "#fef2f2",
        "retired" => "#f1f5f9",
        _ => "#ffffff",
    }
}

fn roadmap_fill_color(
    object: &PortfolioObject,
    presentation: Option<&RoadmapPresentation>,
) -> &'static str {
    match presentation
        .map(roadmap_color_by)
        .unwrap_or("lifecycleState")
    {
        "criticality" => match object.criticality.as_str() {
            "critical" => "#fef2f2",
            "high" => "#fff7ed",
            "medium" => "#fefce8",
            "low" => "#ecfdf5",
            _ => "#ffffff",
        },
        "standardState" => match object.standard_state.as_str() {
            "approved" => "#ecfdf5",
            "tolerated" => "#eff6ff",
            "discouraged" => "#fff7ed",
            "banned" => "#fef2f2",
            "emerging" => "#f5f3ff",
            _ => "#ffffff",
        },
        "portfolioKind" => match object.kind.as_str() {
            "portfolio_application" => "#eff6ff",
            "portfolio_service" => "#ecfeff",
            "technology_component" => "#f5f3ff",
            "technology_standard" => "#eef2ff",
            "lifecycle_milestone" => "#f8fafc",
            _ => "#ffffff",
        },
        "none" => "#ffffff",
        _ => lifecycle_fill_color(object),
    }
}

fn roadmap_border_color(
    object: &PortfolioObject,
    presentation: Option<&RoadmapPresentation>,
) -> &'static str {
    match presentation
        .map(roadmap_color_by)
        .unwrap_or("lifecycleState")
    {
        "criticality" => match object.criticality.as_str() {
            "critical" => "#dc2626",
            "high" => "#ea580c",
            "medium" => "#ca8a04",
            "low" => "#059669",
            _ => "#94a3b8",
        },
        "standardState" => match object.standard_state.as_str() {
            "approved" => "#059669",
            "tolerated" => "#2563eb",
            "discouraged" => "#ea580c",
            "banned" => "#dc2626",
            "emerging" => "#7c3aed",
            _ => "#94a3b8",
        },
        "portfolioKind" => match object.kind.as_str() {
            "portfolio_application" => "#2563eb",
            "portfolio_service" => "#0891b2",
            "technology_component" => "#7c3aed",
            "technology_standard" => "#4f46e5",
            "lifecycle_milestone" => "#64748b",
            _ => "#94a3b8",
        },
        "none" => "#94a3b8",
        _ => lifecycle_border_color(object),
    }
}

fn lifecycle_border_color(object: &PortfolioObject) -> &'static str {
    match lifecycle_bucket(object) {
        "idea" => "#64748b",
        "planned" => "#2563eb",
        "active" => "#059669",
        "deprecated" => "#ea580c",
        "retiring" => "#dc2626",
        "retired" => "#475569",
        _ => "#94a3b8",
    }
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
    write_json(
        package.root.join("model/portfolio.json"),
        &package.portfolio,
    )?;
    write_json(
        package.root.join("views/portfolio-views.json"),
        &package.portfolio_saved_views,
    )?;
    write_json(
        package.root.join("views/roadmap-presentations.json"),
        &package.roadmap_presentations,
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
    for existing in &package.portfolio.objects {
        ids.insert(existing.id.as_str());
    }
    for existing in &package.portfolio_saved_views.views {
        ids.insert(existing.id.as_str());
    }
    for existing in &package.roadmap_presentations.presentations {
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
        .portfolio
        .objects
        .sort_by(|left, right| left.id.cmp(&right.id));
    package
        .portfolio_saved_views
        .views
        .sort_by(|left, right| left.id.cmp(&right.id));
    package
        .roadmap_presentations
        .presentations
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

fn ensure_non_empty_items(values: &[String], field: &str) -> Result<()> {
    for value in values {
        ensure_non_empty(value, field)?;
    }
    Ok(())
}

fn validate_optional_value(value: &str, supported: &[&str], field: &str) -> Result<()> {
    if value.is_empty() || supported.contains(&value) {
        return Ok(());
    }
    bail!("{field} has unsupported value {value}");
}

fn validate_required_value(value: &str, supported: &[&str], field: &str) -> Result<()> {
    ensure_non_empty(value, field)?;
    if supported.contains(&value) {
        return Ok(());
    }
    bail!("{field} has unsupported value {value}");
}

fn validate_optional_date(value: &str, field: &str) -> Result<()> {
    if value.is_empty() {
        return Ok(());
    }
    let bytes = value.as_bytes();
    if bytes.len() == 10
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
    {
        return Ok(());
    }
    bail!("{field} must use YYYY-MM-DD");
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

fn require_any_update_arg(args: &Value, operation: &str) -> Result<()> {
    let object = args
        .as_object()
        .ok_or_else(|| anyhow!("operation args must be an object"))?;
    if object.len() < 2 {
        bail!("{operation} must change at least one field");
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

fn default_sort_direction() -> String {
    "asc".to_string()
}

fn is_default_element_status(status: &str) -> bool {
    status == "accepted"
}

fn is_false(value: &bool) -> bool {
    !*value
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
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
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
    fn proposal_validation_rejects_noop_model_element_detail_update() {
        let proposal: Proposal = serde_json::from_str(
            r#"{
  "proposalId": "proposal.noop-update-details",
  "schemaVersion": "0.1.0",
  "state": "accepted",
  "createdAt": "2026-07-20T22:45:00Z",
  "intent": "Attempt a no-op model element update.",
  "operations": [
    {
      "opId": "op.noop-update",
      "op": "update_model_element_details",
      "args": {
        "elementId": "component.workbench"
      },
      "rationale": "No-op updates should fail validation."
    }
  ]
}
"#,
        )
        .unwrap();

        let error = validate_proposal(&proposal).unwrap_err().to_string();
        assert!(
            error.contains("must change at least one field"),
            "expected no-op update validation error, got {error}"
        );
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
        let roadmap_dot =
            render_lifecycle_roadmap_dot(&package, Some("diagram.portfolio-lifecycle-roadmap"))
                .unwrap();
        assert!(roadmap_dot.contains("application.redshield-architect"));
        assert!(roadmap_dot.contains("milestone.alpha"));
        assert!(roadmap_dot.contains("Timeline scale"));
        assert!(roadmap_dot.contains("Application swimlane"));
        assert!(roadmap_dot.contains("2026 Q3"));
        assert!(roadmap_dot.contains("Target: Active"));
        assert!(roadmap_dot.contains("target state"));
        let roadmap_svg =
            render_lifecycle_roadmap_svg(&package, Some("diagram.portfolio-lifecycle-roadmap"))
                .unwrap();
        assert!(roadmap_svg.contains("<svg"));
        assert!(roadmap_svg.contains("Portfolio Lifecycle Roadmap"));
        assert!(roadmap_svg.contains("Timeline scale"));
        assert!(roadmap_svg.contains("Application swimlane"));
        assert!(roadmap_svg.contains("Target: Active"));
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
        "architecture": {
          "owners": [
            {
              "ref": "owner.product-architecture",
              "role": "accountable",
              "name": "Product Architecture"
            }
          ],
          "lifecycle": {
            "state": "planned",
            "phase": "prototype",
            "milestoneRefs": ["milestone.svg-export"],
            "targetDate": "2026-08-15"
          },
          "criticality": "medium",
          "technologies": [
            {
              "ref": "technology.graphviz",
              "role": "tool",
              "standardState": "approved"
            }
          ],
          "risks": [
            {
              "ref": "risk.export-drift",
              "severity": "medium",
              "status": "identified"
            }
          ],
          "capabilities": [
            {
              "ref": "capability.diagram-export",
              "fit": "primary",
              "maturity": "emerging"
            }
          ],
          "services": [
            {
              "ref": "service.rendering",
              "relationship": "consumes",
              "interfaceRef": "operation.render-use-case"
            }
          ]
        },
        "useCaseDetails": {
          "primaryActorRef": "actor.architect",
          "preconditions": ["An accepted diagram view exists"],
          "postconditions": ["An SVG document is generated"],
          "mainFlow": [
            {
              "step": 1,
              "actorRef": "actor.architect",
              "action": "Request SVG export for the selected diagram"
            }
          ]
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
      "opId": "op.create-export-component",
      "op": "create_model_element",
      "args": {
        "id": "component.svg-exporter",
        "kind": "component",
        "name": "SVG Exporter",
        "description": "Component responsible for producing SVG documents from accepted diagrams.",
        "classifier": {
          "operations": [
            {
              "name": "export",
              "visibility": "public",
              "returnTypeRef": "SvgDocument",
              "parameters": [
                {
                  "name": "diagram",
                  "typeRef": "DiagramView",
                  "direction": "in"
                }
              ]
            }
          ]
        }
      },
      "rationale": "Component classifier details should survive accepted proposal application.",
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
        assert_eq!(summary.elements_created, 2);
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
        assert_eq!(exported.architecture.criticality, "medium");
        assert_eq!(
            exported.architecture.owners[0].ref_id,
            "owner.product-architecture"
        );
        assert_eq!(
            exported
                .architecture
                .lifecycle
                .as_ref()
                .unwrap()
                .milestone_refs,
            vec!["milestone.svg-export"]
        );
        assert_eq!(
            exported.architecture.technologies[0].ref_id,
            "technology.graphviz"
        );
        assert_eq!(exported.architecture.risks[0].ref_id, "risk.export-drift");
        assert_eq!(
            exported.architecture.capabilities[0].ref_id,
            "capability.diagram-export"
        );
        assert_eq!(
            exported.architecture.services[0].ref_id,
            "service.rendering"
        );
        let use_case_details = exported.use_case_details.as_ref().unwrap();
        assert_eq!(use_case_details.primary_actor_ref, "actor.architect");
        assert_eq!(
            use_case_details.main_flow[0].action,
            "Request SVG export for the selected diagram"
        );
        let exporter = package
            .elements
            .elements
            .iter()
            .find(|element| element.id == "component.svg-exporter")
            .unwrap();
        let classifier = exporter.classifier.as_ref().unwrap();
        assert_eq!(classifier.operations[0].name, "export");
        assert_eq!(
            classifier.operations[0].parameters[0].type_ref,
            "DiagramView"
        );
        let applied = fs::read_to_string(summary.applied_proposal_path).unwrap();
        assert!(applied.contains(r#""state": "applied""#));
    }

    #[test]
    fn accepted_proposal_applies_portfolio_operations() {
        let root = copy_example_to_temp();
        let proposal_path = root.join("proposals/open/accepted-portfolio-ops.json");
        fs::write(
            &proposal_path,
            r#"{
  "proposalId": "proposal.portfolio-ops",
  "schemaVersion": "0.1.0",
  "state": "accepted",
  "createdAt": "2026-07-20T20:50:00Z",
  "intent": "Add a native portfolio object through typed operations.",
  "operations": [
    {
      "opId": "op.create-portfolio-service",
      "op": "create_portfolio_object",
      "args": {
        "id": "service.proposal-application",
        "kind": "portfolio_service",
        "name": "Proposal application",
        "description": "Applies accepted proposal operations to canonical package files.",
        "status": "accepted",
        "lifecycleState": "active",
        "lifecycle": {
          "state": "active",
          "phase": "supported service",
          "currentFrom": "2026-07-20",
          "targetState": "active",
          "targetDate": "2026-09-30",
          "milestoneRefs": ["milestone.alpha"]
        },
        "criticality": "high",
        "capabilityRefs": ["capability.model-review"],
        "technologyRefs": ["technology.tauri"],
        "riskRefs": ["risk.silent-model-mutation"],
        "relatedElementRefs": ["component.workbench"],
        "sourceRefs": ["docs/MODEL_PACKAGE.md"]
      },
      "rationale": "Portfolio services should be represented as native RedShield facts, not only text metadata."
    },
    {
      "opId": "op.update-model-review-capability",
      "op": "update_portfolio_object",
      "args": {
        "objectId": "capability.model-review",
        "tags": ["proposal-review", "portfolio"],
        "technologyRefs": ["technology.react-flow"]
      },
      "rationale": "Existing portfolio objects need typed updates with validation."
    }
  ]
}
"#,
        )
        .unwrap();

        let summary = apply_accepted_proposal_file(&root, &proposal_path).unwrap();
        assert_eq!(summary.portfolio_objects_created, 1);
        assert_eq!(summary.portfolio_objects_updated, 1);

        let package = load_package(&root).unwrap();
        validate_package(&package).unwrap();
        let service = package
            .portfolio
            .objects
            .iter()
            .find(|object| object.id == "service.proposal-application")
            .unwrap();
        assert_eq!(service.kind, "portfolio_service");
        assert_eq!(service.related_element_refs, vec!["component.workbench"]);
        assert_eq!(
            service.lifecycle.as_ref().unwrap().milestone_refs,
            vec!["milestone.alpha"]
        );
        let capability = package
            .portfolio
            .objects
            .iter()
            .find(|object| object.id == "capability.model-review")
            .unwrap();
        assert_eq!(capability.tags, vec!["proposal-review", "portfolio"]);
        assert_eq!(capability.technology_refs, vec!["technology.react-flow"]);
    }

    #[test]
    fn accepted_proposal_applies_portfolio_saved_view_operations() {
        let root = copy_example_to_temp();
        let proposal_path = root.join("proposals/open/accepted-portfolio-saved-view.json");

        let summary = apply_accepted_proposal_file(&root, &proposal_path).unwrap();
        assert_eq!(summary.portfolio_saved_view_operations_applied, 3);

        let package = load_package(&root).unwrap();
        validate_package(&package).unwrap();
        assert!(
            package
                .portfolio_saved_views
                .views
                .iter()
                .all(|view| view.id != "portfolio-view.prototype-technologies")
        );
        let saved_view = package
            .portfolio_saved_views
            .views
            .iter()
            .find(|view| view.id == "portfolio-view.active-critical")
            .unwrap();
        assert_eq!(
            saved_view.title,
            "Active and planned critical portfolio facts"
        );
        assert_eq!(
            saved_view.columns,
            vec!["name", "kind", "lifecycleState", "criticality", "ownerRefs"]
        );
    }

    #[test]
    fn accepted_proposal_applies_roadmap_presentation_operations() {
        let root = copy_example_to_temp();
        let proposal_path = root.join("proposals/open/accepted-roadmap-presentation.json");

        let summary = apply_accepted_proposal_file(&root, &proposal_path).unwrap();
        assert_eq!(summary.roadmap_presentation_operations_applied, 5);

        let package = load_package(&root).unwrap();
        validate_package(&package).unwrap();
        assert!(
            package
                .roadmap_presentations
                .presentations
                .iter()
                .all(|presentation| presentation.id != "roadmap-presentation.prototype-critical")
        );
        let diagram = package
            .diagrams
            .diagrams
            .iter()
            .find(|diagram| diagram.id == "diagram.portfolio-lifecycle-roadmap")
            .unwrap();
        assert_eq!(
            diagram.roadmap_presentation_ref,
            "roadmap-presentation.default-lifecycle"
        );
    }

    #[test]
    fn accepted_proposal_applies_portfolio_view_operation() {
        let root = copy_example_to_temp();
        let proposal_path = root.join("proposals/open/accepted-portfolio-view.json");
        fs::write(
            &proposal_path,
            r#"{
  "proposalId": "proposal.portfolio-view",
  "schemaVersion": "0.1.0",
  "state": "accepted",
  "createdAt": "2026-07-20T21:25:00Z",
  "intent": "Create a lifecycle roadmap view over portfolio facts.",
  "operations": [
    {
      "opId": "op.create-lifecycle-roadmap",
      "op": "create_diagram_view",
      "args": {
        "id": "diagram.lifecycle-roadmap",
        "title": "Lifecycle Roadmap",
        "viewKind": "lifecycle_roadmap",
        "portfolioRefs": [
          "application.redshield-architect",
          "technology.tauri",
          "milestone.alpha"
        ]
      },
      "rationale": "Portfolio views should be package views before they have full UI rendering."
    }
  ]
}
"#,
        )
        .unwrap();

        let summary = apply_accepted_proposal_file(&root, &proposal_path).unwrap();
        assert_eq!(summary.diagrams_created, 1);

        let package = load_package(&root).unwrap();
        validate_package(&package).unwrap();
        let diagram = package
            .diagrams
            .diagrams
            .iter()
            .find(|diagram| diagram.id == "diagram.lifecycle-roadmap")
            .unwrap();
        assert_eq!(diagram.view_kind, "lifecycle_roadmap");
        assert_eq!(
            diagram.portfolio_refs,
            vec![
                "application.redshield-architect",
                "technology.tauri",
                "milestone.alpha"
            ]
        );
    }

    #[test]
    fn portfolio_summary_reports_object_counts() {
        let root = copy_example_to_temp();
        let package = load_package(&root).unwrap();
        let lines = portfolio_summary_lines(&package, None);

        assert!(lines.contains(&"portfolio objects: 8".to_string()));
        assert!(lines.contains(&"- technology_component: 1".to_string()));
        assert!(lines.contains(&"- planned: 3".to_string()));
        assert!(lines.iter().any(|line| {
            line == "- application.redshield-architect [portfolio_application] status=accepted lifecycle=planned criticality=high"
        }));
        let filtered = portfolio_summary_lines(&package, Some("tauri"));
        assert!(filtered.contains(&"portfolio objects: 1 of 8 matching \"tauri\"".to_string()));
        assert!(
            filtered
                .iter()
                .any(|line| line.contains("technology.tauri"))
        );
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
    fn accepted_proposal_updates_model_element_details() {
        let root = copy_example_to_temp();
        let proposal_path = root.join("proposals/open/accepted-update-element-details.json");
        fs::write(
            &proposal_path,
            r#"{
  "proposalId": "proposal.update-workbench-details",
  "schemaVersion": "0.1.0",
  "state": "accepted",
  "createdAt": "2026-07-20T22:35:00Z",
  "intent": "Update existing workbench semantic element details.",
  "operations": [
    {
      "opId": "op.update-workbench-details",
      "op": "update_model_element_details",
      "args": {
        "elementId": "component.workbench",
        "name": "Workbench Shell",
        "documentation": "Semantic details were updated through an accepted proposal.",
        "status": "proposed",
        "tags": ["ui", "diagram", "semantic-edit"],
        "architecture": {
          "owners": [
            {
              "ref": "owner.workbench",
              "role": "technical",
              "name": "Workbench Architecture"
            }
          ],
          "lifecycle": {
            "state": "planned",
            "phase": "semantic editing"
          },
          "criticality": "critical"
        },
        "clearDetails": ["classifier"]
      },
      "rationale": "Existing model element details should be updated through typed proposal operations."
    }
  ]
}
"#,
        )
        .unwrap();

        let summary = apply_accepted_proposal_file(&root, &proposal_path).unwrap();
        assert_eq!(summary.model_element_detail_operations_applied, 1);

        let package = load_package(&root).unwrap();
        validate_package(&package).unwrap();
        let workbench = package
            .elements
            .elements
            .iter()
            .find(|element| element.id == "component.workbench")
            .unwrap();
        assert_eq!(workbench.name, "Workbench Shell");
        assert_eq!(workbench.status, "proposed");
        assert!(workbench.classifier.is_none());
        assert_eq!(workbench.tags, vec!["ui", "diagram", "semantic-edit"]);
        assert_eq!(workbench.architecture.criticality, "critical");
        assert_eq!(workbench.architecture.owners[0].ref_id, "owner.workbench");
        assert_eq!(
            workbench.architecture.lifecycle.as_ref().unwrap().phase,
            "semantic editing"
        );
        let applied = fs::read_to_string(summary.applied_proposal_path).unwrap();
        assert!(applied.contains(r#""op": "update_model_element_details""#));
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
