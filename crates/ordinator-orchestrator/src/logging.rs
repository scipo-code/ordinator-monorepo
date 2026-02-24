use std::env;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use file_rotate::ContentLimit;
use file_rotate::FileRotate;
use file_rotate::compression::Compression;
use file_rotate::suffix::AppendCount;
use tracing::Level;
use tracing::event;
use tracing_appender::non_blocking;
use tracing_appender::non_blocking::NonBlocking;
use tracing_flame::FlameLayer;
use tracing_subscriber::Registry;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::filter::Filtered;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::fmt::format::Format;
use tracing_subscriber::fmt::format::Json;
use tracing_subscriber::fmt::format::JsonFields;
use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;
use tracing_subscriber::reload::Handle;

type LogLayer = Handle<
    Filtered<Layer<Registry, JsonFields, Format<Json>, NonBlocking>, EnvFilter, Registry>,
    Registry,
>;
type ProfilingLayer = Filtered<FlameLayer<Registry, BufWriter<File>>, EnvFilter, Registry>;

#[derive(Debug)]
pub struct LogHandles
{
    pub file_handle: Option<LogLayer>,
    pub _flame_handle: Option<Handle<ProfilingLayer, Registry>>,
}

// TODO [ ]
// I think that this should be removed and replaced by the
// `tracing` crate. Yes you should
// #[derive(Subcommand, Debug, Clone, Serialize, Deserialize)]
// pub enum LogLevel {
//     Trace,
//     Debug,
//     Info,
//     Warn,
//     Error,
// }

// impl LogLevel {
//     pub fn to_level_string(&self) -> String {
//         match self {
//             LogLevel::Trace => "trace".to_string(),
//             LogLevel::Debug => "debug".to_string(),
//             LogLevel::Info => "info".to_string(),
//             LogLevel::Warn => "warn".to_string(),
//             LogLevel::Error => "error".to_string(),
//         }
//     }
// }

pub fn setup_logging() -> anyhow::Result<LogHandles>
{
    let log_dir = env::var("ORDINATOR_LOG_DIR")
        .expect("A logging/tracing directory should be set in the .env file");

    let log_dir_path = Path::new(&log_dir).join("ordinator");
    setup_logging_directory_structure(&log_dir_path)
        .context("Could not setup logging directories")?;

    let research_path: PathBuf = log_dir_path.clone().join("ordinator.research.log");
    let developer_path: PathBuf = log_dir_path.clone().join("ordinator.developer.log");
    let debug_path: PathBuf = log_dir_path.clone().join("ordinator.debug.log");
    let business_events_path: PathBuf = log_dir_path.clone().join("ordinator.business_events.log");

    let research_file = FileRotate::new(
        research_path,
        AppendCount::new(1),
        ContentLimit::Bytes(1024 * 1024 * 1024),
        Compression::None,
        None,
    );

    let developer_file = FileRotate::new(
        developer_path,
        AppendCount::new(0),
        ContentLimit::Bytes(50 * 1024 * 1024),
        Compression::None,
        None,
    );

    let debug_file = FileRotate::new(
        debug_path,
        AppendCount::new(0),
        ContentLimit::Bytes(50 * 1024 * 1024),
        Compression::None,
        None,
    );

    let business_events_file = FileRotate::new(
        business_events_path,
        AppendCount::new(5),
        ContentLimit::Time(file_rotate::TimeFrequency::Weekly),
        Compression::None,
        None,
    );

    let (research_writer, research_log_guard) = non_blocking(research_file);
    std::mem::forget(research_log_guard);
    let (developer_writer, developer_log_guard) = non_blocking(developer_file);
    std::mem::forget(developer_log_guard);
    let (debug_writer, developer_log_guard) = non_blocking(debug_file);
    std::mem::forget(developer_log_guard);
    let (business_events_writer, business_events_guard) = non_blocking(business_events_file);
    std::mem::forget(business_events_guard);

    // Configure targets to route logs to correct files via event!(target: "...")

    let research_targets = Targets::new().with_target("research", Level::INFO);
    let debug_targets = Targets::new().with_target("debug", Level::TRACE);

    let developer_targets = Targets::new().with_target("developer", Level::TRACE);
    let business_event_targets = Targets::new().with_target("business_events", Level::INFO);
    let stdout_targets = Targets::new().with_target("stdout", Level::TRACE);

    let research_layer = fmt::layer()
        .with_writer(research_writer)
        .json()
        .with_ansi(true)
        .with_file(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true)
        .with_filter(research_targets);

    let developer_layer = fmt::layer()
        .with_writer(developer_writer)
        .with_ansi(true)
        .with_file(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true)
        .with_filter(developer_targets);

    let debug_layer = fmt::layer()
        .with_writer(debug_writer)
        .with_ansi(true)
        .with_file(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true)
        .with_filter(debug_targets);

    let business_events_layer = fmt::layer()
        .with_writer(business_events_writer)
        .json()
        .with_thread_names(true)
        .with_filter(business_event_targets);

    let stdout_layer = tracing_subscriber::fmt::layer().with_filter(stdout_targets);

    // let flame_layer = FlameLayer::with_file(
    //     env::var("PROFILING_FILE").expect("A file name for the profiling data has
    // to be set"), )
    // .unwrap()
    // .0
    // .with_filter(EnvFilter::from_env("PROFILING_LEVEL"));

    // let layers = vec![
    //     research_layer.boxed(),
    //     flame_layer.boxed(),
    //     developer_layer.boxed(),
    // ];

    // TODO: Implement tracing::reload for dynamic logging reconfiguration
    tracing_subscriber::registry()
        .with(research_layer)
        .with(developer_layer)
        .with(debug_layer)
        .with(business_events_layer)
        .with(stdout_layer)
        // .with(flame_layer)
        .init();

    event!(target: "debug", Level::INFO, "TESTING TRACING");
    event!(target: "stdout", Level::INFO, "System initialized (1 of 4): logging");
    Ok(LogHandles {
        file_handle: None,
        _flame_handle: None,
    })
}

fn setup_logging_directory_structure(log_directory_path: &PathBuf) -> anyhow::Result<()>
{
    if !log_directory_path.exists() {
        std::fs::create_dir(log_directory_path).context("Could not create the log directory")?
    }

    let entries = fs::read_dir(log_directory_path).context("Could not read the log directory")?;

    for entry in entries {
        let path = entry?.path();
        fs::remove_file(path).context("Could not remove the previous logging files")?;
    }

    Ok(())
}
