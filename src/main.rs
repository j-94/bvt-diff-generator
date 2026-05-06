use anyhow::{Context, Result};
use bvt_diff_generator::{
    apply_bundle, benchmark, build_bundle, check_bundle, compile_dsl, load_dsl_plan, load_plan,
};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "bvt-diff")]
#[command(about = "Compile typed edit ops into unified diffs with BVT-style receipts.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Generate {
        plan: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        receipt: Option<PathBuf>,
    },
    Apply {
        plan: PathBuf,
        #[arg(long)]
        receipt: Option<PathBuf>,
    },
    Check {
        plan: PathBuf,
    },
    CompileDsl {
        dsl_plan: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    RunDsl {
        dsl_plan: PathBuf,
        #[arg(long)]
        run_dir: PathBuf,
    },
    Bench {
        #[arg(long, default_value_t = 1000)]
        iterations: usize,
        #[arg(long, default_value_t = 32)]
        ops: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Generate { plan, out, receipt } => {
            let plan = load_plan(&plan)?;
            let bundle = build_bundle(&plan)?;
            write_or_print(out, &bundle.unified_diff)?;
            if let Some(path) = receipt {
                write_json(path, &bundle.receipt)?;
            }
        }
        Command::Apply { plan, receipt } => {
            let plan = load_plan(&plan)?;
            let bundle = build_bundle(&plan)?;
            apply_bundle(&plan.base_dir, &bundle)?;
            if let Some(path) = receipt {
                write_json(path, &bundle.receipt)?;
            } else {
                println!("{}", serde_json::to_string_pretty(&bundle.receipt)?);
            }
        }
        Command::Check { plan } => {
            let plan = load_plan(&plan)?;
            let receipt = check_bundle(&plan)?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
        Command::CompileDsl { dsl_plan, out } => {
            let dsl = load_dsl_plan(&dsl_plan)?;
            let plan = compile_dsl(&dsl)?;
            let raw = serde_json::to_string_pretty(&plan)?;
            write_or_print(out, &format!("{raw}\n"))?;
        }
        Command::RunDsl { dsl_plan, run_dir } => {
            let dsl = load_dsl_plan(&dsl_plan)?;
            let plan = compile_dsl(&dsl)?;
            fs::create_dir_all(&run_dir)
                .with_context(|| format!("creating {}", run_dir.display()))?;

            let plan_path = run_dir.join("plan.json");
            let diff_path = run_dir.join("diff.patch");
            let generate_receipt_path = run_dir.join("generate-receipt.json");
            let apply_receipt_path = run_dir.join("apply-receipt.json");
            let run_receipt_path = run_dir.join("run-receipt.json");

            write_json(plan_path.clone(), &plan)?;
            let bundle = build_bundle(&plan)?;
            write_or_print(Some(diff_path.clone()), &bundle.unified_diff)?;
            write_json(generate_receipt_path.clone(), &bundle.receipt)?;
            apply_bundle(&plan.base_dir, &bundle)?;
            write_json(apply_receipt_path.clone(), &bundle.receipt)?;

            let run_receipt = serde_json::json!({
                "state": "dsl_packet_loaded",
                "delta": dsl.intent,
                "control": [
                    "compiled_dsl",
                    "generated_unified_diff",
                    "wrote_receipts",
                    "applied_candidate"
                ],
                "next_state": "transition_applied",
                "dsl_plan": dsl_plan,
                "plan": plan_path,
                "diff": diff_path,
                "generate_receipt": generate_receipt_path,
                "apply_receipt": apply_receipt_path,
                "op_count": bundle.receipt.op_count,
                "file_count": bundle.receipt.file_count,
            });
            write_json(run_receipt_path.clone(), &run_receipt)?;
            println!("{}", serde_json::to_string_pretty(&run_receipt)?);
        }
        Command::Bench { iterations, ops } => {
            let receipt = benchmark(iterations, ops)?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
        }
    }
    Ok(())
}

fn write_or_print(path: Option<PathBuf>, content: &str) -> Result<()> {
    if let Some(path) = path {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
        fs::write(&path, content).with_context(|| format!("writing {}", path.display()))?;
    } else {
        print!("{content}");
    }
    Ok(())
}

fn write_json<T: serde::Serialize>(path: PathBuf, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    let raw = serde_json::to_string_pretty(value)?;
    fs::write(&path, format!("{raw}\n")).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}
