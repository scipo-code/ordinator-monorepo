// TODO ISSUE #000 [ ] make the `handle_state_link` method into a
// function that accpect `handle_state_link(&mut Parameters,
// &SchedulingEnvironment)`
//
//
//
// #[cfg(not(target_env = "msvc"))]
// use tikv_jemallocator::Jemalloc;
// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;
mod handlers;
mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use axum::routing::get;
use chrono::TimeZone;
use chrono_tz::Europe::Copenhagen;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Asset;
// use std::fs::File;
// use std::io::Read;
use ordinator_orchestrator::Orchestrator;
use routes::api::v1::api_scope;
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;
use tracing::info;
use utoipa::openapi::Info;
use utoipa::openapi::OpenApiBuilder;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::Config;
use utoipa_swagger_ui::SwaggerUi;

pub const RESEARCH: &str = "research";

#[tokio::main]
async fn main() -> Result<()>
{
    dotenvy::dotenv()
        .context("You need to provide an .env file. Look at the .env.example for guidance")?;

    // TODO [ ] 2025-06-29 turn this into `match
    // dotenvy::var("DEPLOY_ENVIRONMENT");`

    let denmark_time = Copenhagen.with_ymd_and_hms(2025, 1, 13, 7, 00, 00).unwrap();
    let current_time = denmark_time.to_utc();

    // ISSUE #000 Turn the nested `std::sync::Mutex` into `tokio::sync::Mutex`
    // ISSUE #000 TODO [ ] 2025-06-29 turn this into `match
    //  dotenvy::var("DEPLOY_ENVIRONMENT");` instead of `Option::Some(current_time)`
    // TODO [ ] This is quite annoying.}
    let (orchestrator, error_handle, system_clock_handle): (
        Arc<Orchestrator<TotalSystemSolution>>,
        JoinHandle<Result<()>>,
        JoinHandle<()>,
    ) = Orchestrator::new(Some(current_time)).context("Orchestrator could not be created")?;

    // WARN: Manually add `Asset`s here. Everything added here should be done from
    // the API in actual production. So this is only a temporary solution.

    orchestrator.asset_factory(&Asset::DF)?;

    let scheduler_files = ServeDir::new("./static_files/scheduler/dist");
    let supervisor_files =
        ServeDir::new("./static_files/supervisor/dist/supervisor-calendar/browser");

    let app = OpenApiRouter::new()
        .nest("/api/v1", api_scope(orchestrator.clone()).await)
        .nest_service("/scheduler", scheduler_files)
        .nest_service("/supervisor", supervisor_files)
        .route("/hello", get(|| async { "Hello, world!" }))
        .with_state(orchestrator)
        .split_for_parts();

    // Here you can modify the the 'OpenApi' specification
    let openapi = OpenApiBuilder::from(app.1)
        .info(Info::new("Ordinator API Specification", "0.2.2"))
        .build();

    // Here you can modify the [`SwaggerUi`]
    let swagger_config = Config::new(["/api-doc/openapi.json"])
        .display_request_duration(true)
        .try_it_out_enabled(true);

    let merged_app = app.0.merge(
        SwaggerUi::new("/swagger")
            .config(swagger_config)
            .url("/api-doc/openapi.json", openapi),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = axum_server::bind(addr).serve(merged_app.into_make_service());

    info!(target: "stdout", "System initialized (4 of 4): ordinator-api-server");
    tokio::select! {
        res = server => res?,
        res = error_handle => res??,
        res = system_clock_handle => res?,
    }

    Ok(())
}

// WARN [ ] 2025-07-02 move this to the api module. This is good inspiration.
// let openapi = OpenApiBuilder::from(openapi)
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
// WARN [ ] move this, good inspiration.
//     routing::{get, post},
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
