use anyhow::{Context, Result};

use super::render::{ToolPaths, list_pipelines, render_pipeline_json};
use crate::cli::RunArgs;
use crate::context::RunContext;
use crate::output::format::banner;

fn decode_plan_to_wire(bytes: &[u8]) -> anyhow::Result<hm_plugin_protocol::Pipeline> {
    serde_json::from_slice(bytes).map_err(|e| anyhow::anyhow!("decode pipeline JSON: {e}"))
}

/// Run a pipeline locally via Docker.
///
/// # Errors
///
/// Returns an error if the working directory cannot be resolved, no
/// pipeline slug was given when more than one is declared (or none are
/// declared), the Python DSL transpile or Scheme evaluator step fails,
/// the resulting plan does not decode, the Docker daemon is unreachable,
/// or the orchestrator surfaces an internal scheduler error. Non-zero
/// step exit codes are returned as the `i32`, not as an Err.
pub async fn handle(args: RunArgs, _ctx: RunContext) -> Result<i32> {
    let repo_root = match args.dir.clone() {
        Some(p) => p,
        None => std::env::current_dir().context("cannot determine current directory")?,
    };

    let tools = ToolPaths::discover()?;

    let slug = if let Some(s) = &args.pipeline {
        s.clone()
    } else {
        let metas = list_pipelines(&tools, &repo_root).await?;
        let slugs: Vec<String> = metas.into_iter().map(|m| m.slug).collect();
        match slugs.as_slice() {
            [only] => only.clone(),
            [] => anyhow::bail!(
                "no pipelines declared in this repo\n  \
                 hint: define one with `@hm.pipeline(\"slug\")` in `.harmont/pipeline.py`"
            ),
            many => anyhow::bail!(
                "this repo declares pipelines: {}\n  → pass one as the first argument",
                many.join(", ")
            ),
        }
    };

    if args.format == "human" {
        banner("run --local", &format!("slug={slug}"));
    }

    let json = render_pipeline_json(&tools, &repo_root, &slug).await?;
    let pipeline_wire = decode_plan_to_wire(&json)?;
    let parallelism = args.parallelism.unwrap_or_else(|| {
        std::thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get)
    });
    let exit_code =
        crate::orchestrator::run(pipeline_wire, repo_root, parallelism, args.format.clone())
            .await?;
    Ok(exit_code)
}
