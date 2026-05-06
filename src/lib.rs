use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffPlan {
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub base_dir: PathBuf,
    #[serde(default)]
    pub ops: Vec<EditOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DslPlan {
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub base_dir: PathBuf,
    #[serde(default)]
    pub actions: Vec<DslAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DslAction {
    ReplaceLineContaining {
        path: PathBuf,
        needle: String,
        line: String,
    },
    InsertAfterContaining {
        path: PathBuf,
        needle: String,
        lines: Vec<String>,
    },
    AppendFile {
        path: PathBuf,
        lines: Vec<String>,
    },
    EnsureFile {
        path: PathBuf,
        lines: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EditOp {
    Replace {
        path: PathBuf,
        start: usize,
        end: usize,
        lines: Vec<String>,
    },
    Insert {
        path: PathBuf,
        at: usize,
        lines: Vec<String>,
    },
    Delete {
        path: PathBuf,
        start: usize,
        end: usize,
    },
    WriteFile {
        path: PathBuf,
        lines: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePatch {
    pub path: PathBuf,
    pub before: String,
    pub after: String,
    pub op_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffBundle {
    pub intent: String,
    pub file_patches: Vec<FilePatch>,
    pub unified_diff: String,
    pub receipt: Receipt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub state: String,
    pub delta: String,
    pub control: Vec<String>,
    pub next_state: String,
    pub op_count: usize,
    pub file_count: usize,
    pub elapsed_ms: f64,
    pub ops_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchReceipt {
    pub iterations: usize,
    pub ops_per_iteration: usize,
    pub elapsed_ms: f64,
    pub ops_per_second: f64,
    pub generated_diff_bytes: usize,
}

pub fn load_plan(path: &Path) -> Result<DiffPlan> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut plan: DiffPlan = serde_json::from_str(&raw).context("parsing diff plan json")?;
    if plan.base_dir.as_os_str().is_empty() {
        plan.base_dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
    }
    Ok(plan)
}

pub fn load_dsl_plan(path: &Path) -> Result<DslPlan> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut plan: DslPlan = serde_json::from_str(&raw).context("parsing dsl plan json")?;
    if plan.base_dir.as_os_str().is_empty() {
        plan.base_dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
    }
    Ok(plan)
}

pub fn compile_dsl(plan: &DslPlan) -> Result<DiffPlan> {
    let mut ops = Vec::with_capacity(plan.actions.len());
    for action in &plan.actions {
        match action {
            DslAction::ReplaceLineContaining { path, needle, line } => {
                validate_relative_path(path)?;
                let line_no = find_line_containing(&plan.base_dir.join(path), needle)?;
                ops.push(EditOp::Replace {
                    path: path.clone(),
                    start: line_no,
                    end: line_no,
                    lines: vec![line.clone()],
                });
            }
            DslAction::InsertAfterContaining {
                path,
                needle,
                lines,
            } => {
                validate_relative_path(path)?;
                let line_no = find_line_containing(&plan.base_dir.join(path), needle)?;
                ops.push(EditOp::Insert {
                    path: path.clone(),
                    at: line_no + 1,
                    lines: lines.clone(),
                });
            }
            DslAction::AppendFile { path, lines } => {
                validate_relative_path(path)?;
                let existing = fs::read_to_string(plan.base_dir.join(path)).unwrap_or_default();
                let at = split_lines(&existing).len() + 1;
                ops.push(EditOp::Insert {
                    path: path.clone(),
                    at,
                    lines: lines.clone(),
                });
            }
            DslAction::EnsureFile { path, lines } => {
                validate_relative_path(path)?;
                ops.push(EditOp::WriteFile {
                    path: path.clone(),
                    lines: lines.clone(),
                });
            }
        }
    }
    Ok(DiffPlan {
        intent: plan.intent.clone(),
        base_dir: plan.base_dir.clone(),
        ops,
    })
}

pub fn build_bundle(plan: &DiffPlan) -> Result<DiffBundle> {
    let started = Instant::now();
    let mut by_path: BTreeMap<PathBuf, Vec<EditOp>> = BTreeMap::new();
    for op in &plan.ops {
        by_path
            .entry(op.path().to_path_buf())
            .or_default()
            .push(op.clone());
    }

    let mut file_patches = Vec::with_capacity(by_path.len());
    for (path, ops) in by_path {
        validate_relative_path(&path)?;
        let abs = plan.base_dir.join(&path);
        let before = fs::read_to_string(&abs).unwrap_or_default();
        let after = apply_ops_to_text(&before, &ops)?;
        file_patches.push(FilePatch {
            path,
            before,
            after,
            op_count: ops.len(),
        });
    }

    let unified_diff = render_unified_diff(&file_patches);
    let elapsed = started.elapsed().as_secs_f64();
    let op_count = plan.ops.len();
    let receipt = Receipt {
        state: "workspace_files_read".to_string(),
        delta: format!("{} structured edit ops", op_count),
        control: vec![
            "relative_paths_only".to_string(),
            "line_ranges_validated".to_string(),
            "unified_diff_rendered".to_string(),
            "no_write_without_apply".to_string(),
        ],
        next_state: "candidate_diff_generated".to_string(),
        op_count,
        file_count: file_patches.len(),
        elapsed_ms: elapsed * 1000.0,
        ops_per_second: if elapsed > 0.0 {
            op_count as f64 / elapsed
        } else {
            0.0
        },
    };

    Ok(DiffBundle {
        intent: plan.intent.clone(),
        file_patches,
        unified_diff,
        receipt,
    })
}

pub fn apply_bundle(base_dir: &Path, bundle: &DiffBundle) -> Result<()> {
    for patch in &bundle.file_patches {
        validate_relative_path(&patch.path)?;
        let abs = base_dir.join(&patch.path);
        if let Some(parent) = abs.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        fs::write(&abs, &patch.after).with_context(|| format!("writing {}", abs.display()))?;
    }
    Ok(())
}

pub fn check_bundle(plan: &DiffPlan) -> Result<Receipt> {
    let bundle = build_bundle(plan)?;
    if bundle.file_patches.is_empty() {
        bail!("plan contains no file patches");
    }
    Ok(bundle.receipt)
}

pub fn benchmark(iterations: usize, ops_per_iteration: usize) -> Result<BenchReceipt> {
    let started = Instant::now();
    let mut generated_diff_bytes = 0usize;
    for i in 0..iterations {
        let mut ops = Vec::with_capacity(ops_per_iteration);
        for j in 0..ops_per_iteration {
            ops.push(EditOp::Insert {
                path: PathBuf::from("bench.txt"),
                at: j + 1,
                lines: vec![format!("bench-{i}-{j}")],
            });
        }
        let before = (0..ops_per_iteration)
            .map(|n| format!("line-{n}\n"))
            .collect::<String>();
        let after = apply_ops_to_text(&before, &ops)?;
        let patch = FilePatch {
            path: PathBuf::from("bench.txt"),
            before,
            after,
            op_count: ops.len(),
        };
        generated_diff_bytes += render_unified_diff(&[patch]).len();
    }
    let elapsed = started.elapsed().as_secs_f64();
    let total_ops = iterations.saturating_mul(ops_per_iteration);
    Ok(BenchReceipt {
        iterations,
        ops_per_iteration,
        elapsed_ms: elapsed * 1000.0,
        ops_per_second: if elapsed > 0.0 {
            total_ops as f64 / elapsed
        } else {
            0.0
        },
        generated_diff_bytes,
    })
}

pub fn apply_ops_to_text(before: &str, ops: &[EditOp]) -> Result<String> {
    let mut lines = split_lines(before);
    let mut indexed = ops.iter().enumerate().collect::<Vec<_>>();
    indexed.sort_by(|(a_idx, a), (b_idx, b)| {
        b.anchor().cmp(&a.anchor()).then_with(|| b_idx.cmp(a_idx))
    });

    for (_, op) in indexed {
        match op {
            EditOp::Replace {
                start,
                end,
                lines: new_lines,
                ..
            } => {
                validate_range(*start, *end, lines.len())?;
                lines.splice(start - 1..*end, normalize_new_lines(new_lines));
            }
            EditOp::Insert {
                at,
                lines: new_lines,
                ..
            } => {
                if *at == 0 || *at > lines.len() + 1 {
                    bail!("insert line {at} outside 1..{}", lines.len() + 1);
                }
                lines.splice(at - 1..at - 1, normalize_new_lines(new_lines));
            }
            EditOp::Delete { start, end, .. } => {
                validate_range(*start, *end, lines.len())?;
                lines.drain(start - 1..*end);
            }
            EditOp::WriteFile {
                lines: new_lines, ..
            } => {
                lines = normalize_new_lines(new_lines);
            }
        }
    }

    Ok(lines.concat())
}

pub fn render_unified_diff(file_patches: &[FilePatch]) -> String {
    let mut out = String::new();
    for patch in file_patches {
        out.push_str(&format!("--- a/{}\n", patch.path.display()));
        out.push_str(&format!("+++ b/{}\n", patch.path.display()));
        let diff = TextDiff::from_lines(&patch.before, &patch.after);
        for group in diff.grouped_ops(3) {
            for op in group {
                for change in diff.iter_changes(&op) {
                    let sign = match change.tag() {
                        ChangeTag::Delete => "-",
                        ChangeTag::Insert => "+",
                        ChangeTag::Equal => " ",
                    };
                    out.push_str(sign);
                    out.push_str(change.value());
                    if !change.value().ends_with('\n') {
                        out.push('\n');
                    }
                }
            }
        }
    }
    out
}

impl EditOp {
    pub fn path(&self) -> &Path {
        match self {
            EditOp::Replace { path, .. }
            | EditOp::Insert { path, .. }
            | EditOp::Delete { path, .. }
            | EditOp::WriteFile { path, .. } => path,
        }
    }

    fn anchor(&self) -> usize {
        match self {
            EditOp::Replace { start, .. } | EditOp::Delete { start, .. } => *start,
            EditOp::Insert { at, .. } => *at,
            EditOp::WriteFile { .. } => usize::MAX,
        }
    }
}

fn validate_relative_path(path: &Path) -> Result<()> {
    if path.is_absolute() {
        bail!("absolute paths are not allowed: {}", path.display());
    }
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            bail!("parent path components are not allowed: {}", path.display());
        }
    }
    Ok(())
}

fn validate_range(start: usize, end: usize, len: usize) -> Result<()> {
    if start == 0 || end < start || end > len {
        return Err(anyhow!("range {start}..{end} outside 1..{len}"));
    }
    Ok(())
}

fn split_lines(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split_inclusive('\n')
        .map(ToString::to_string)
        .collect()
}

fn normalize_new_lines(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            if line.ends_with('\n') {
                line.clone()
            } else {
                format!("{line}\n")
            }
        })
        .collect()
}

fn find_line_containing(path: &Path, needle: &str) -> Result<usize> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    for (idx, line) in raw.lines().enumerate() {
        if line.contains(needle) {
            return Ok(idx + 1);
        }
    }
    bail!("no line in {} contains {:?}", path.display(), needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_insert_replace_delete_from_bottom_up() {
        let before = "a\nb\nc\nd\n";
        let ops = vec![
            EditOp::Insert {
                path: "x.txt".into(),
                at: 2,
                lines: vec!["x".to_string()],
            },
            EditOp::Replace {
                path: "x.txt".into(),
                start: 3,
                end: 3,
                lines: vec!["C".to_string()],
            },
            EditOp::Delete {
                path: "x.txt".into(),
                start: 4,
                end: 4,
            },
        ];
        let after = apply_ops_to_text(before, &ops).unwrap();
        assert_eq!(after, "a\nx\nb\nC\n");
    }

    #[test]
    fn renders_unified_diff() {
        let patch = FilePatch {
            path: "x.txt".into(),
            before: "a\nb\n".to_string(),
            after: "a\nc\n".to_string(),
            op_count: 1,
        };
        let diff = render_unified_diff(&[patch]);
        assert!(diff.contains("--- a/x.txt"));
        assert!(diff.contains("-b"));
        assert!(diff.contains("+c"));
    }
}
