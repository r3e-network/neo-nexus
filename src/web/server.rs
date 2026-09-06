//! The web server entrypoint: resolve security posture before starting threads,
//! bind, print the operator banner, and serve until the process is stopped.

use std::{
    fs::File,
    io::{IsTerminal, Read},
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use zeroize::Zeroizing;

use crate::repository::Repository;
use crate::workspace_lock::WorkspaceSupervisorLock;

use super::{
    auth::{AuthStore, WebSecurity, LEGACY_TOKEN_ENV, PUBLIC_ORIGIN_ENV, TOKEN_FILE_ENV},
    router::build_router,
    WebState,
};

const MAX_TOKEN_FILE_BYTES: u64 = 4 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebLaunch {
    pub bind: IpAddr,
    pub port: u16,
    pub token_file: Option<PathBuf>,
    pub public_origin: Option<String>,
    pub data_dir: PathBuf,
}

struct ResolvedStartup {
    auth: AuthStore,
    security: WebSecurity,
    bootstrap_token: Option<Zeroizing<String>>,
}

pub fn run_web_server(launch: WebLaunch) -> Result<()> {
    // Resolve credentials before Tokio creates worker threads. Environment-only
    // secret state is rejected here, and a noninteractive process cannot emit a
    // bootstrap credential into a service log.
    let startup = resolve_startup(&launch, std::io::stdout().is_terminal())?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to start the async runtime")?;
    runtime.block_on(serve(launch, startup))
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

async fn serve(launch: WebLaunch, mut startup: ResolvedStartup) -> Result<()> {
    let _workspace_lock = WorkspaceSupervisorLock::acquire(&launch.data_dir)?;
    let db_path = launch.data_dir.join("neonexus.db");
    let repository = Repository::open(&db_path)
        .with_context(|| format!("failed to open workspace database {}", db_path.display()))?;
    let addr = SocketAddr::new(launch.bind, launch.port);
    let state = WebState::new(
        repository,
        launch.data_dir.clone(),
        startup.auth,
        startup.security.clone(),
    )
    .context("signer service configuration is incomplete or invalid")?;
    let router = build_router(state.clone());
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    // The engine shares the server's supervisor rather than making its own, so
    // a node the watchdog restarts is a node the browser can stop. Held until
    // this returns, which stops the loop on shutdown.
    let _supervision = crate::supervision::Engine::start(state.engine_state())?;
    let bootstrap_token = startup.bootstrap_token.take();
    print_banner(
        &addr,
        &startup.security,
        bootstrap_token.as_ref().map(|token| token.as_str()),
    );
    drop(bootstrap_token);

    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("web server stopped unexpectedly")
}

fn resolve_startup(launch: &WebLaunch, interactive_stdout: bool) -> Result<ResolvedStartup> {
    let requested_origin = match launch.public_origin.clone() {
        Some(origin) => Some(origin),
        None => read_optional_unicode_env(PUBLIC_ORIGIN_ENV)?,
    };
    let security = WebSecurity::resolve(launch.bind, requested_origin.as_deref())?;

    let env_token_file = std::env::var_os(TOKEN_FILE_ENV).map(PathBuf::from);
    let legacy_token_present = std::env::var_os(LEGACY_TOKEN_ENV).is_some();
    let token = resolve_operator_token(
        launch.token_file.as_deref(),
        env_token_file.as_deref(),
        legacy_token_present,
        interactive_stdout,
    )?;
    let auth = AuthStore::from_token(&token.value)?;
    let bootstrap_token = token.generated.then_some(token.value);
    Ok(ResolvedStartup {
        auth,
        security,
        bootstrap_token,
    })
}

struct ResolvedToken {
    value: Zeroizing<String>,
    generated: bool,
}

fn resolve_operator_token(
    cli_token_file: Option<&Path>,
    env_token_file: Option<&Path>,
    legacy_token_present: bool,
    interactive_stdout: bool,
) -> Result<ResolvedToken> {
    if legacy_token_present {
        bail!(
            "{LEGACY_TOKEN_ENV} is refused because process environments expose secrets; store the token in a protected file and set {TOKEN_FILE_ENV}"
        );
    }
    if let Some(path) = cli_token_file.or(env_token_file) {
        return Ok(ResolvedToken {
            value: read_token_file(path)?,
            generated: false,
        });
    }
    if !interactive_stdout {
        bail!(
            "noninteractive web startup requires --web-token-file or {TOKEN_FILE_ENV}; a bootstrap token is never written to redirected output"
        );
    }
    Ok(ResolvedToken {
        value: Zeroizing::new(uuid::Uuid::new_v4().to_string()),
        generated: true,
    })
}

fn read_token_file(path: &Path) -> Result<Zeroizing<String>> {
    let file = File::open(path)
        .with_context(|| format!("failed to open web token file {}", path.display()))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("failed to inspect web token file {}", path.display()))?;
    if !metadata.is_file() {
        bail!("web token path {} is not a regular file", path.display());
    }
    if metadata.len() > MAX_TOKEN_FILE_BYTES {
        bail!(
            "web token file {} exceeds the {MAX_TOKEN_FILE_BYTES}-byte limit",
            path.display()
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            bail!(
                "web token file {} must not be readable or writable by group or other users",
                path.display()
            );
        }
    }

    let mut token = Zeroizing::new(String::new());
    file.take(MAX_TOKEN_FILE_BYTES + 1)
        .read_to_string(&mut token)
        .with_context(|| format!("web token file {} is not valid UTF-8", path.display()))?;
    Ok(Zeroizing::new(
        super::auth::validated_operator_token(&token)?.to_string(),
    ))
}

fn read_optional_unicode_env(name: &str) -> Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => bail!("{name} must contain valid Unicode"),
    }
}

fn print_banner(addr: &SocketAddr, security: &WebSecurity, bootstrap_token: Option<&str>) {
    println!("NeoNexus web workbench ready");
    println!(
        "  address: {}",
        security
            .public_origin()
            .map(str::to_string)
            .unwrap_or_else(|| format!("http://{addr}"))
    );
    println!("  production sign-in token: --web-token-file or {TOKEN_FILE_ENV}");
    if let Some(token) = bootstrap_token {
        println!("  one-time interactive web token: {token}");
    }
    println!("  runs in the foreground; interrupt the terminal to stop");
}

#[cfg(test)]
#[path = "../../tests/unit/web/server/tests.rs"]
mod tests;
