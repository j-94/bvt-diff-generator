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
