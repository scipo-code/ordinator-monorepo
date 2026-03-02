use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use chrono::TimeZone;
use chrono_tz::Europe::Copenhagen;
use ordinator_api_server::start_application;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::logging::setup_logging;
use tracing::info;

pub const RESEARCH: &str = "research";

#[tokio::main]
async fn main() -> Result<()>
{
    info!(target: "stdout", "System initialized (0 of 4): loading environment");
    dotenvy::dotenv()
        .context("You need to provide an .env file. Look at the .env.example for guidance")?;

    // TODO: Refactor to use `match dotenvy::var("DEPLOY_ENVIRONMENT")` pattern
    // instead of hardcoded value

    let denmark_time = Copenhagen.with_ymd_and_hms(2025, 1, 13, 7, 00, 00).unwrap();
    let current_time = denmark_time.to_utc();

    // ISSUE #000: Replace nested `std::sync::Mutex` with `tokio::sync::Mutex`

    let asset = Asset::DF;
    let environment = ordinator_orchestrator::Environment::Test(current_time);
    let (orchestrator, error_receiver, _system_clock_handle) =
        Orchestrator::<TotalSystemSolution>::builder()
            .logging(setup_logging()?)
            .system_clock(&environment)
            .system_configurations()
            .scheduling_environment_from_database(&asset)?
            .build()?;

    // WARN: Assets are manually added here. In production, assets should be added
    // via the API. This is a temporary solution.

    orchestrator.asset_factory(&asset)?;

    tokio::select! {
        result = start_application(orchestrator.clone(), &environment) => {
            info!(target: "stdout", server_shutdown_message = ?result, "Main server shutting down");
            Ok(())
        }
        result = error_receiver.recv_async() => {
            tracing::error!(ordinator_error_message = ?result, "Ordinator Scheduling Systems experienced a catastrophic error");
            bail!("Ordinator Scheduling Systems experienced a catastrophic error:\n{:?}", result);
        }
    }
}

// WARN: Move this to the API module. This code provides good OpenAPI/Swagger UI
// configuration inspiration. let openapi = OpenApiBuilder::from(openapi)
//     // ‣ info (title, version, description, license)
//     .info(
//         Info::new("My Service", "1.0.0")
//             .description("This is a **full-featured** example of Utoipa’s
// builder API.")             .license(License::new("MIT", "https://opensource.org/licenses/MIT")),
//     )
//     // ‣ tags
//     .tags(vec![
//         Tag::new("Users").description(Some("Operations about users")),
//     ])
//     // ‣ components (your schemas)
//     .components(
//         ComponentsBuilder::new()
//             .schema(
//                 "User",
//                 RefOr::T(Schema::new("User").properties(/* … */)),
//             )
//             .schema("CreateUser",
// RefOr::T(Schema::new("CreateUser").properties(/* … */)))
// .schema("ErrorResponse", RefOr::T(Schema::new("ErrorResponse").properties(/*
// … */))),     )
//     // ‣ security schemes + global requirement
//     .security_schemes(vec![(
//         "api_key".to_string(),
//         SecurityScheme::ApiKey(ApiKey::header("X-API-KEY")),
//     )])
//     .security(vec![SecurityRequirement::new("api_key")])
//     // ‣ servers
//     .servers(vec![
//         ServerBuilder::new()
//             .url("https://api.example.com")
//             .description(Some("production"))
//             .build(),
//         ServerBuilder::new()
//             .url("http://localhost:3000")
//             .description(Some("development"))
//             .build(),
//     ])
//     // ‣ vendor extensions
//     .extensions(vec![("x-global".to_string(), json!({ "info": "global
// extension" }))])     .build();  // ← produce the final OpenApi struct
// //
// :contentReference[oaicite:0]{index=0}

// // 3. Merge with your SwaggerUi just like before
// let app = router.merge(
//     SwaggerUi::new("/swagger")
//         .config(
//             Config::new(["/api-doc/openapi.json"])
//                 .display_request_duration(true)
//                 .try_it_out_enabled(true),
//         )
//         .url("/api-doc/openapi.json", openapi),
// );
//
//
//
//
//
// use axum::{
// WARN: Move this to appropriate module. Provides good code organization
// reference.     routing::{get, post},
//     Router,
// };
// use serde::{Deserialize, Serialize};
// use utoipa::{
//     openapi::{Info, License, Server},
//     Modify, OpenApi,
// };
// use utoipa_swagger_ui::SwaggerUi;

// /// A simple data model: a user.
// #[derive(Serialize, Deserialize, utoipa::ToSchema)]
// pub struct User {
//     /// Unique identifier for the user
//     pub id: u32,
//     /// User’s preferred display name
//     pub name: String,
// }

// /// A request body for creating a user.
// #[derive(Deserialize, utoipa::ToSchema)]
// pub struct CreateUser {
//     /// The name for the new user
//     pub name: String,
// }

// /// A simple API error response.
// #[derive(Serialize, utoipa::ToSchema)]
// pub struct ErrorResponse {
//     /// Error message
//     pub message: String,
// }

// /// Handler to list all users.
// ///
// /// Returns a JSON array of `User`.
// #[utoipa::path(
//     get,
//     path = "/users",
//     responses(
//         (status = 200, description = "List users", body = [User]),
//         (status = 500, description = "Internal server error", body =
// ErrorResponse),     ),
//     tag = "Users"
// )]
// async fn list_users() -> axum::Json<Vec<User>> {
//     // ...
//     axum::Json(vec![])
// }

// /// Handler to create a new user.
// ///
// /// Expects a JSON `CreateUser` body, returns the created `User`.
// #[utoipa::path(
//     post,
//     path = "/users",
//     request_body(content = CreateUser, description = "New user data"),
//     responses(
//         (status = 201, description = "User created", body = User),
//         (status = 400, description = "Invalid input", body = ErrorResponse),
//     ),
//     tag = "Users",
//     security(
//         ("api_key" = [])
//     )
// )]
// async fn create_user(
//     axum::Json(payload): axum::Json<CreateUser>,
// ) -> axum::Json<User> {
//     // ...
//     axum::Json(User { id: 1, name: payload.name })
// }

// /// You can modify the generated OpenAPI document if you need to inject
// /// vendor extensions or tweak things at the very end.
// struct MyExtra;
// impl Modify for MyExtra {
//     fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
//         // add a custom extension to the root
//         openapi.info = openapi.info.clone().title("🚀 My API");
//         openapi
//             .extensions
//             .insert("x-logo".into(), serde_json::json!({"url":"https://example.com/logo.png"}));
//     }
// }
