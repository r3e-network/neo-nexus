//! The web server entrypoint: bind, print the operator banner, serve until the
//! process is stopped. Intentionally synchronous from `main`'s perspective —
//! the function owns its tokio runtime and blocks.

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use anyhow::{Context, Result};

use crate::repository::Repository;

use super::{auth::AuthStore, router::build_router, WebState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebLaunch {
    pub bind: IpAddr,
    pub port: u16,
    pub token: Option<String>,
    pub data_dir: PathBuf,
}

pub fn run_web_server(launch: WebLaunch) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to start the async runtime")?;
    runtime.block_on(serve(launch))
}

/// The workspace root: `NEONEXUS_DATA_DIR` wins, otherwise the OS data
/// directory — the same convention the workbench has always used, so the web
/// server manages the same `neonexus.db` the CLI writes to.
pub fn default_data_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("NEONEXUS_DATA_DIR") {
        return PathBuf::from(path);
    }
    dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("NeoNexus")
}

async fn serve(launch: WebLaunch) -> Result<()> {
    let db_path = launch.data_dir.join("neonexus.db");
    let repository = Repository::open(&db_path)
        .with_context(|| format!("failed to open workspace database {}", db_path.display()))?;

    let (auth, generated) = auth_from_launch(&launch);
    let addr = SocketAddr::new(launch.bind, launch.port);
    let state = WebState::new(repository, launch.data_dir.clone(), auth);
    let router = build_router(state);

    print_banner(&addr, generated);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    axum::serve(listener, router)
        .await
        .context("web server stopped unexpectedly")
}

/// Resolve the admin token: explicit flag beats environment beats a generated
/// value. Generated tokens are printed once — the server keeps only a digest.
fn auth_from_launch(launch: &WebLaunch) -> (AuthStore, Option<String>) {
    if let Some(token) = launch
        .token
        .as_ref()
        .filter(|token| !token.trim().is_empty())
    {
        return (AuthStore::from_token(token.trim()), None);
    }
    if let Ok(token) = std::env::var("NEONEXUS_WEB_TOKEN") {
        if !token.trim().is_empty() {
            return (AuthStore::from_token(token.trim()), None);
        }
    }
    let generated = uuid::Uuid::new_v4().to_string();
    (AuthStore::from_token(&generated), Some(generated))
}

fn print_banner(addr: &SocketAddr, generated_token: Option<String>) {
    println!("NeoNexus web workbench ready");
    println!("  address: http://{addr}");
    println!("  sign in with --web-token or NEONEXUS_WEB_TOKEN on cloud hosts");
    if let Some(token) = generated_token {
        println!("  web-token: {token}");
    }
    println!("  runs in the foreground; interrupt the terminal to stop");
}
