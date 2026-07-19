use anyhow::{Context, Result, bail};
use redshield_architect::{
    apply_accepted_proposal_file, load_package, render_use_case_svg, validate_package,
    validate_proposals,
};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        return Ok(());
    };

    match command.as_str() {
        "validate" => {
            let root = args
                .next()
                .unwrap_or_else(|| "examples/minimal/redshield".to_string());
            let package = load_package(&root)?;
            let mut warnings = validate_package(&package)?;
            warnings.extend(validate_proposals(&root)?);
            if warnings.is_empty() {
                println!("validated {}", package.root.display());
            } else {
                println!("validated {} with warnings:", package.root.display());
                for warning in warnings {
                    println!("- {warning}");
                }
            }
        }
        "render-use-case" => {
            let root = args
                .next()
                .unwrap_or_else(|| "examples/minimal/redshield".to_string());
            let output = args
                .next()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("target/redshield/first-use-case.svg"));
            let package = load_package(&root)?;
            validate_package(&package)?;
            let svg = render_use_case_svg(&package, None)?;
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            fs::write(&output, svg).with_context(|| format!("writing {}", output.display()))?;
            println!("rendered {}", output.display());
        }
        "apply-proposal" => {
            let root = args
                .next()
                .unwrap_or_else(|| "examples/minimal/redshield".to_string());
            let Some(proposal_path) = args.next() else {
                bail!("apply-proposal requires a proposal JSON path");
            };
            let summary = apply_accepted_proposal_file(&root, proposal_path)?;
            println!("applied proposal:");
            println!("- requirements created: {}", summary.requirements_created);
            println!("- elements created: {}", summary.elements_created);
            println!("- relationships created: {}", summary.relationships_created);
            println!("- diagrams created: {}", summary.diagrams_created);
            println!("- trace links created: {}", summary.trace_links_created);
            println!(
                "- diagram layout operations applied: {}",
                summary.diagram_layout_operations_applied
            );
            println!(
                "- applied proposal copy: {}",
                summary.applied_proposal_path.display()
            );
        }
        "help" | "--help" | "-h" => print_usage(),
        other => bail!("unknown command {other}"),
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  redshield-architect validate [redshield-dir]");
    println!("  redshield-architect render-use-case [redshield-dir] [output.svg]");
    println!("  redshield-architect apply-proposal [redshield-dir] <proposal.json>");
}
