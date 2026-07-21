use anyhow::{Context, Result, anyhow};
use redshield_architect::Proposal;
use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
fn redshield_save_proposal_draft(key: String, draft: Proposal) -> Result<(), String> {
    let path = proposal_draft_path(&key).map_err(error_message)?;
    write_proposal(&path, &draft).map_err(error_message)
}

#[tauri::command]
fn redshield_load_proposal_draft(key: String) -> Result<Option<Proposal>, String> {
    let path = proposal_draft_path(&key).map_err(error_message)?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    serde_json::from_str(&raw)
        .with_context(|| format!("reading proposal draft {}", path.display()))
        .map(Some)
        .map_err(error_message)
}

#[tauri::command]
fn redshield_export_proposal_draft(draft: Proposal) -> Result<String, String> {
    let file_name = format!(
        "{}.{}.json",
        sanitize_path_segment(&draft.proposal_id),
        sanitize_path_segment(&draft.state)
    );
    let path = package_root()
        .map_err(error_message)?
        .join("proposals")
        .join("open")
        .join(file_name);
    write_proposal(&path, &draft).map_err(error_message)?;
    Ok(path.display().to_string())
}

fn write_proposal(path: &Path, draft: &Proposal) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let mut body = serde_json::to_string_pretty(draft)?;
    body.push('\n');
    fs::write(path, body).with_context(|| format!("writing {}", path.display()))
}

fn proposal_draft_path(key: &str) -> Result<PathBuf> {
    Ok(package_root()?
        .join("proposals")
        .join("open")
        .join(format!("{}.json", sanitize_path_segment(key))))
}

fn package_root() -> Result<PathBuf> {
    if let Ok(root) = std::env::var("REDSHIELD_PACKAGE_ROOT") {
        let path = PathBuf::from(root);
        if path.is_absolute() {
            return Ok(path);
        }
        return std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .context("resolving REDSHIELD_PACKAGE_ROOT");
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow!("could not resolve repository root from Tauri manifest dir"))?;
    Ok(repo_root.join("examples").join("minimal").join("redshield"))
}

fn sanitize_path_segment(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = sanitized.trim_matches(['.', '-']).to_string();
    if trimmed.is_empty() {
        "workbench-draft".to_string()
    } else {
        trimmed
    }
}

fn error_message(error: anyhow::Error) -> String {
    error.to_string()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            redshield_save_proposal_draft,
            redshield_load_proposal_draft,
            redshield_export_proposal_draft,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RedShield Architect workbench");
}
