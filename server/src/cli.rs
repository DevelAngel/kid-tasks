use clap::Args;
pub use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};
use url::Url;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// Family task management server, with an MCP interface for AI assistants
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[clap(flatten)]
    pub server: ServerArgs,

    /// Directory where task JSON files are stored.
    /// Defaults to the current working directory.
    #[clap(long, env = "KID_TASKS_DIR")]
    pub tasks_dir: Option<PathBuf>,

    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,
}

#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Sets the address the MCP server listens on.
    ///
    /// Kept on its own port, separate from the browser-facing HTTP
    /// server, so AI assistant clients never have to pass through
    /// Tinyauth's interactive login flow (see ADR: rmcp-mcp-server).
    #[clap(long = "mcp-listen", global = true, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9100))]
    pub mcp_addr: SocketAddr,

    /// This server's own public URL, used when issuing OAuth metadata
    /// (issuer, authorization/token endpoints), e.g. "https://mcp.example.com".
    #[clap(
        long = "mcp-base-url",
        global = true,
        env = "KID_MCP_BASE_URL",
        value_name = "URL",
        default_value = "http://127.0.0.1:9100"
    )]
    pub mcp_base_url: Url,

    /// Origins allowed to access the MCP server cross-origin, e.g.
    /// "https://claude.ai". Can be repeated or comma-separated.
    #[clap(
        long = "mcp-allowed-origin",
        global = true,
        env = "KID_MCP_ALLOWED_ORIGINS",
        value_name = "URL",
        value_delimiter = ','
    )]
    pub mcp_allowed_origins: Vec<Url>,

    /// TOML file listing OAuth clients allowed to authenticate against the
    /// MCP server (name, redirect URI, secret). If unset or empty, no client
    /// can complete the authorization flow; use --mcp-disable-oauth to skip
    /// OAuth instead.
    #[clap(
        long = "mcp-clients-file",
        global = true,
        env = "KID_MCP_CLIENTS_FILE",
        value_name = "PATH"
    )]
    pub mcp_clients_file: Option<PathBuf>,

    /// Skip OAuth on the MCP server: anyone who can reach its port can
    /// use it, acting as `--mcp-actor`. Only for trusted networks.
    #[clap(
        long = "mcp-disable-oauth",
        global = true,
        env = "KID_MCP_DISABLE_OAUTH",
        conflicts_with = "mcp_clients_file",
        requires = "mcp_actor"
    )]
    pub mcp_oauth_disabled: bool,

    /// Author recorded for every change made through the MCP server
    /// while OAuth is disabled, e.g. "ai:claude:alice".
    #[clap(
        long = "mcp-actor",
        global = true,
        env = "KID_MCP_ACTOR",
        value_name = "ACTOR",
        requires = "mcp_oauth_disabled"
    )]
    pub mcp_actor: Option<String>,

    /// Address for the test-control admin channel (switch_dir/count/
    /// set_time_offset/reset_time_offset/flush), used by the e2e
    /// browser test harness. Only compiled in when the `test-control`
    /// Cargo feature is enabled; never set this in production.
    #[cfg(feature = "test-control")]
    #[clap(
        long = "test-control-listen",
        global = true,
        env = "KID_TEST_CONTROL_ADDR"
    )]
    pub test_control_addr: Option<SocketAddr>,
}
