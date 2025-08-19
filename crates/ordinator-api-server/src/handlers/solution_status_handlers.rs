use std::sync::Arc;

use anyhow::Context;
use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use ordinator_contracts::AssetNames;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use serde::Serialize;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::routes::api::AppError;

#[derive(ToSchema, Serialize, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct SolutionVersionDto
{
    pub stagnation_iterations: u64,
    pub version: u64,
}
#[debug_handler]
#[utoipa::path(
    get,
    tag = "Solution Status",
    path = "/{asset}/tactical",
    params (
        ("asset" = AssetNames, Path),
    ),
    responses(
        (status = 200, body = SolutionVersionDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn tactical_solution_status(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(asset): Path<Asset>,
) -> Result<Json<SolutionVersionDto>, AppError>
{
    let stagnation_and_version = orchestrator
        .system_solutions
        .lock()
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .get(&asset)
        .with_context(|| format!("{asset:#?} does not exist in the SystemSolution"))
        .map_err(|e| AppError::Anyhow(format!("{e:?}")))?
        .load()
        .tactical
        .as_ref()
        .context("TacticalSolution not present in SystemSolution")
        .map_err(|e| AppError::Anyhow(format!("{e:?}")))?
        .stagnation_and_version();

    Ok(Json(SolutionVersionDto {
        stagnation_iterations: stagnation_and_version.0,
        version: stagnation_and_version.1,
    }))
}

#[debug_handler]
#[utoipa::path(
    get,
    tag = "Solution Status",
    path = "/{asset}/strategic",
    params (
        ("asset" = AssetNames, Path),
    ),
    responses(
        (status = 200, body = SolutionVersionDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn strategic_solution_status(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(asset): Path<Asset>,
) -> Result<Json<SolutionVersionDto>, AppError>
{
    let stagnation_and_version = orchestrator
        .system_solutions
        .lock()
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .get(&asset)
        .with_context(|| format!("{asset:#?} does not exist in the SystemSolution"))
        .map_err(|e| AppError::Anyhow(format!("{e:?}")))?
        .load()
        .strategic
        .as_ref()
        .context("TacticalSolution not present in SystemSolution")
        .map_err(|e| AppError::Anyhow(format!("{e:?}")))?
        .stagnation_and_version();

    Ok(Json(SolutionVersionDto {
        stagnation_iterations: stagnation_and_version.0,
        version: stagnation_and_version.1,
    }))
}

#[debug_handler]
#[utoipa::path(
    get,
    tag = "Solution Status",
    path = "/{asset}/supervisor",
    params (
        ("asset" = AssetNames, Path),
    ),
    responses(
        (status = 200, body = SolutionVersionDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn supervisor_solution_status(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(asset): Path<Asset>,
) -> Result<Json<SolutionVersionDto>, AppError>
{
    let stagnation_and_version = orchestrator
        .system_solutions
        .lock()
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .get(&asset)
        .with_context(|| format!("{asset:#?} does not exist in the SystemSolution"))
        .map_err(|e| AppError::Anyhow(format!("{e:?}")))?
        .load()
        .supervisor
        .as_ref()
        .context("TacticalSolution not present in SystemSolution")
        .map_err(|e| AppError::Anyhow(format!("{e:?}")))?
        .stagnation_and_version();

    Ok(Json(SolutionVersionDto {
        stagnation_iterations: stagnation_and_version.0,
        version: stagnation_and_version.1,
    }))
}
