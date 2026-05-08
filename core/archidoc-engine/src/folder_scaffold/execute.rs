use std::path::Path;

use super::types::{ActionOutcome, HookResult, ScaffoldAction, ScaffoldError, ScaffoldPlan, ScaffoldResult};

/// Execute a scaffold plan — create directories and files.
pub fn execute_plan(plan: &ScaffoldPlan) -> Result<ScaffoldResult, ScaffoldError> {
    let mut outcomes = Vec::new();

    for action in &plan.actions {
        match action {
            ScaffoldAction::CreateDir { path } => {
                if path.exists() {
                    outcomes.push(ActionOutcome::Skipped {
                        path: path.clone(),
                        reason: "directory already exists".to_string(),
                    });
                    continue;
                }
                match std::fs::create_dir_all(path) {
                    Ok(_) => outcomes.push(ActionOutcome::Created(path.clone())),
                    Err(e) => {
                        return Err(ScaffoldError::Io(e));
                    }
                }
            }
            ScaffoldAction::CreateFile { path, contents } => {
                // Ensure parent directory exists
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(ScaffoldError::Io)?;
                    }
                }
                match std::fs::write(path, contents) {
                    Ok(_) => outcomes.push(ActionOutcome::Created(path.clone())),
                    Err(e) => {
                        return Err(ScaffoldError::Io(e));
                    }
                }
            }
        }
    }

    // Run post-hooks
    let hook_results = run_post_hooks(&plan.post_hooks, &plan.target_root);

    Ok(ScaffoldResult {
        outcomes,
        hook_results,
    })
}

/// Run post-scaffold hooks in the target directory.
fn run_post_hooks(hooks: &[super::types::PostHook], working_dir: &Path) -> Vec<HookResult> {
    let mut results = Vec::new();

    for hook in hooks {
        let output = if cfg!(windows) {
            std::process::Command::new("cmd")
                .args(["/C", &hook.command])
                .current_dir(working_dir)
                .output()
        } else {
            std::process::Command::new("sh")
                .args(["-c", &hook.command])
                .current_dir(working_dir)
                .output()
        };

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                let combined = format!("{}{}", stdout, stderr);
                results.push(HookResult {
                    command: hook.command.clone(),
                    success: result.status.success(),
                    output: combined,
                });
            }
            Err(e) => {
                results.push(HookResult {
                    command: hook.command.clone(),
                    success: false,
                    output: e.to_string(),
                });
            }
        }
    }

    results
}
