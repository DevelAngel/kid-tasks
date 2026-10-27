use anyhow::{Context, Result};
use assert_fs::TempDir;
use cucumber::World;
use rmcp::model::{ClientCapabilities, ClientConfig, Implementation};
use rmcp::service::RunningService;
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::{RoleClient, ServiceExt};
use thirtyfour::prelude::*;

use std::fmt::{self, Debug, Formatter};

use crate::helpers::MCP_ADDR;
use crate::oauth;

#[derive(World)]
#[world(init = Self::new)]
pub struct AppWorld {
    pub http: WebDriver,
    /// MCP client for actual task mutations (`add`), see
    /// `docs/adr/rmcp-mcp-server.md`. Lazily connected on first use via
    /// [`AppWorld::mcp`] — most scenarios (e.g. pure browser navigation)
    /// never touch it, and eagerly connecting here would make every
    /// scenario pay for (and depend on) the OAuth handshake.
    pub mcp: Option<RunningService<RoleClient, ClientConfig>>,
    /// HTTP client for the e2e test harness's admin channel
    /// (`switch_dir`/`count`/`set_time_offset`/`reset_time_offset`/`flush`).
    pub admin: reqwest::Client,
    pub tasks_dir: Option<TempDir>,
    /// Time offset in seconds set via `set_time_offset` on the admin channel.
    /// Seeds use `Utc::now() + offset` as reference point.
    pub time_offset_seconds: Option<i64>,
}

impl Debug for AppWorld {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppWorld")
            .field("http", &self.http)
            .field("mcp", &"Option<RunningService> { .. }")
            .field("admin", &"reqwest::Client { .. }")
            .field("tasks_dir", &self.tasks_dir)
            .field("time_offset_seconds", &self.time_offset_seconds)
            .finish()
    }
}

impl AppWorld {
    async fn new() -> Result<Self> {
        let mut capa = DesiredCapabilities::chrome();
        capa.set_headless()?;
        capa.add_arg("--window-size=360,1400")?;
        capa.add_arg("--hide-scrollbars")?;
        let http = WebDriver::managed(capa).await?;

        // Enable CDP Network domain so steps can inject headers
        // (e.g. Remote-User) per scenario.
        http.cdp().network().enable().await?;

        let admin = reqwest::Client::new();

        Ok(Self {
            http,
            mcp: None,
            admin,
            tasks_dir: None,
            time_offset_seconds: None,
        })
    }

    /// Returns the MCP client, connecting (incl. the OAuth handshake) on
    /// first use and caching the connection for the rest of the scenario.
    pub async fn mcp(&mut self) -> Result<&RunningService<RoleClient, ClientConfig>> {
        if self.mcp.is_none() {
            let access_token = oauth::fetch_access_token(MCP_ADDR).await?;
            let transport = StreamableHttpClientTransport::with_client(
                reqwest::Client::new(),
                StreamableHttpClientTransportConfig::with_uri(format!("http://{MCP_ADDR}/mcp"))
                    .auth_header(access_token),
            );
            let client_info = ClientConfig::new(
                ClientCapabilities::default(),
                Implementation::new("kid-e2e", env!("CARGO_PKG_VERSION")),
            );
            let mcp = client_info
                .serve(transport)
                .await
                .context("MCP connect failed")?;
            self.mcp = Some(mcp);
        }
        Ok(self.mcp.as_ref().expect("just set above"))
    }
}
