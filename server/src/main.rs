#![recursion_limit = "256"]
mod builder;
mod cache;
mod cli;
mod http;
mod mcp;
mod oauth;
#[cfg(feature = "test-control")]
mod testctl;

use crate::builder::ServerBuilder;
use crate::cache::{SharedTaskCache, SharedTimeOffset, TaskCacheFlush};
use crate::cli::{Cli, Parser};
use crate::oauth::McpClientsConfig;

use leptos::prelude::get_configuration;
use miette::Result;
use miette::{IntoDiagnostic, MietteHandlerOpts};
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::Level;

use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    init_miette_report();
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .init();

    let leptos_conf = get_configuration(None).into_diagnostic()?;
    let http_addr = leptos_conf.leptos_options.site_addr;
    let mcp_addr = args.server.mcp_addr;

    let task_cache = match args.tasks_dir {
        None => SharedTaskCache::default(),
        Some(dir) => {
            if dir.is_absolute() {
                tracing::info!("tasks directory: {}", dir.display());
            } else if tracing::enabled!(Level::INFO) {
                tracing::info!("tasks directory (relative): {}", dir.display());
                let cwd = env::current_dir().expect("CWD available");
                let dir = cwd.join(&dir);
                tracing::info!("tasks directory (absolute): {}", dir.display());
            }
            SharedTaskCache::with_dir(dir)
        }
    };
    task_cache
        .write()
        .await
        .load()
        .await
        .map(|(num_loaded, num_to_migrate)| {
            if num_loaded > 0 {
                tracing::info!("{num_loaded} tasks loaded");
                if num_to_migrate > 0 {
                    tracing::info!("{num_to_migrate} tasks has to be migrated with next flush");
                }
            } else {
                tracing::warn!("no tasks loaded");
            }
        })?;

    let time_offset = SharedTimeOffset::default();
    let mcp_clients = McpClientsConfig::load(args.server.mcp_clients_file.as_deref())?;

    let shutdown = CancellationToken::new();
    #[allow(unused_mut)]
    let mut server = ServerBuilder::new(&shutdown, &task_cache, &time_offset)
        .with_mcp_addr(&mcp_addr)
        .with_mcp_base_url(&args.server.mcp_base_url)
        .with_mcp_allowed_origins(&args.server.mcp_allowed_origins)
        .with_mcp_clients(mcp_clients)
        .with_http_addr(&http_addr)
        .with_leptos_options(&leptos_conf.leptos_options);
    #[cfg(feature = "test-control")]
    {
        server = server.with_test_control_addr(args.server.test_control_addr);
    }
    let server = server.try_spawn().await?;

    setup_signals().await;
    graceful_shutdown(shutdown, task_cache).await;
    server.join().await?;
    Ok(())
}

async fn setup_signals() {
    use signal::ctrl_c;
    use signal::unix::{SignalKind, signal};
    tokio::select! {
        _ = ctrl_c() => tracing::info!("signal received: Ctrl+C"),
        _ = async {
            signal(SignalKind::terminate())
                .expect("failed to install terminate signal handler")
                .recv()
                .await
        } => tracing::info!("signal received: terminate"),
    }
}

async fn graceful_shutdown(shutdown: CancellationToken, task_cache: SharedTaskCache) {
    shutdown.cancel();
    tracing::warn!("Gracefully shutdown started..");
    task_cache.final_flush().await;
    tracing::info!("Gracefully shutdown finished.");
}

fn init_miette_report() {
    let _ = miette::set_hook(Box::new(|_| {
        Box::new(
            MietteHandlerOpts::new()
                .color(false)
                .without_syntax_highlighting()
                .terminal_links(true)
                .unicode(true)
                .build(),
        )
    }));
}
