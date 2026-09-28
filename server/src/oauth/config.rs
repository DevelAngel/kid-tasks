use miette::{IntoDiagnostic, Result};
use secrecy::SecretString;
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::Path;
use url::Url;

/// Fixed set of prefixes a client's id can carry, recorded as the leading
/// `<prefix>:` segment of the actor string (`McpService::actor`) attributed
/// to its changes. Deliberately closed - an arbitrary free-text prefix would
/// let a config typo silently produce a new, unintended prefix instead of
/// failing to parse.
///
/// This is purely about the actor string, not about OAuth authentication:
/// the OAuth client_id a client authenticates with is its `client-id`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientPrefix {
    /// This client is an AI assistant, not a human-operated or purely
    /// mechanical (e.g. cron-triggered) client.
    Ai,
}

impl fmt::Display for ClientPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ai => write!(f, "ai"),
        }
    }
}

/// A single OAuth client allowed to authenticate against the MCP server,
/// as read from the `--mcp-clients-file` TOML file.
///
/// `redirect_uri` is required for clients that use the `authorization_code`
/// grant (they're redirected back to it after user approval), but has no
/// meaning for machine clients that only use the `client_credentials` grant
/// - those can leave it unset.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct McpClientConfig {
    /// Prepended to `name` (as `<prefix>:<name>`) in the actor string
    /// attributed to this client's changes - not part of the OAuth
    /// client_id it authenticates with. Omit for clients that aren't AI
    /// assistants.
    #[serde(default)]
    pub prefix: Option<ClientPrefix>,
    /// The OAuth client_id this client authenticates with. Must be unique.
    pub client_id: String,
    /// The client's identity in the actor string attributed to its
    /// changes - independent of `client_id`. Not required to be unique:
    /// rotating a client's credential can keep the same `name` across old
    /// and new `client_id`, so the actor trail stays attributed to the
    /// same identity.
    pub name: String,
    #[serde(default)]
    pub redirect_uri: Option<Url>,
    pub secret: SecretString,
    /// Human this client acts on behalf of, recorded as the trailing
    /// `:<on-behalf-of>` segment of the actor string attributed to its
    /// changes. Fixed per client rather than a tool parameter, since a
    /// client claiming this for itself can't be trusted any more than it
    /// claiming its own identity.
    ///
    /// Omit for read-only clients: with no human to attribute changes to,
    /// this client can't be allowed to make any - all mutating tools are
    /// rejected for it.
    #[serde(default)]
    pub on_behalf_of: Option<String>,
}

/// Top-level shape of the `--mcp-clients-file` TOML file:
///
/// ```toml
/// [[client]]
/// client-id = "mcp-inspector"
/// name = "mcp-inspector"
/// redirect-uri = "http://localhost:6274/oauth/callback"
/// secret = "..."
/// on-behalf-of = "Jane"
///
/// [[client]]
/// prefix = "ai"
/// client-id = "example-ai-client"
/// name = "example.ai"
/// redirect-uri = "https://example.ai/api/mcp/auth_callback"
/// secret = "..."
/// on-behalf-of = "Jane"
///
/// # machine client, only uses the client_credentials grant, read-only
/// # (no on-behalf-of, so mutating tools are rejected)
/// [[client]]
/// client-id = "matrix-relay-client"
/// name = "matrix-relay"
/// secret = "..."
/// ```
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpClientsConfig {
    #[serde(rename = "client", default)]
    pub clients: Vec<McpClientConfig>,
}

impl McpClientsConfig {
    /// Loads the client registry from `path`, or returns an empty registry
    /// (MCP OAuth disabled) if `path` is `None`.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            tracing::warn!(
                "no --mcp-clients-file given: MCP OAuth is disabled, no client can authenticate"
            );
            return Ok(Self::default());
        };

        let raw = fs::read_to_string(path).into_diagnostic()?;
        let config: Self = toml::from_str(&raw).into_diagnostic()?;
        config.ensure_unique_client_ids()?;

        if config.clients.is_empty() {
            tracing::error!(
                "{} lists no clients: MCP OAuth is disabled, no client can authenticate",
                path.display()
            );
        } else {
            if tracing::enabled!(tracing::Level::DEBUG) {
                config.clients.iter().for_each(|c| {
                    let name = &c.name;
                    let prefix = &c.prefix.map(|p| p.to_string()).unwrap_or_default();
                    let uri = c
                        .redirect_uri
                        .as_ref()
                        .map(|uri| format!(" with uri {uri}"))
                        .unwrap_or_default();
                    let on_behalf_of = &c
                        .on_behalf_of
                        .as_ref()
                        .map(|name| format!(" on behalf of {name}"))
                        .unwrap_or_default();
                    tracing::debug!("{prefix}{name}{on_behalf_of}{uri}");
                });
            }
            tracing::info!(
                "{} MCP OAuth client(s) loaded from {}",
                config.clients.len(),
                path.display()
            );
        }

        Ok(config)
    }

    fn ensure_unique_client_ids(&self) -> Result<()> {
        let mut seen = HashSet::new();
        match self.clients.iter().find(|c| !seen.insert(&c.client_id)) {
            Some(c) => Err(miette::miette!("duplicate client-id: {}", c.client_id)),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_prefix_display() {
        assert_eq!(ClientPrefix::Ai.to_string(), "ai");
    }

    #[test]
    fn unknown_prefix_value_fails_to_parse() {
        let toml = r#"
            [[client]]
            prefix = "bot"
            client-id = "example-id"
            name = "example"
            secret = "s"
            on-behalf-of = "Jane"
        "#;
        let err = toml::from_str::<McpClientsConfig>(toml).unwrap_err();
        assert!(err.to_string().contains("unknown variant"), "{err}");
    }

    #[test]
    fn missing_prefix_defaults_to_none() {
        let toml = r#"
            [[client]]
            client-id = "example-id"
            name = "example"
            secret = "s"
            on-behalf-of = "Jane"
        "#;
        let config: McpClientsConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.clients[0].prefix, None);
    }

    #[test]
    fn missing_client_id_fails_to_parse() {
        let toml = r#"
            [[client]]
            name = "example"
            secret = "s"
        "#;
        let err = toml::from_str::<McpClientsConfig>(toml).unwrap_err();
        assert!(err.to_string().contains("client-id"), "{err}");
    }

    #[test]
    fn name_and_client_id_are_independent() {
        let toml = r#"
            [[client]]
            client-id = "example-id"
            name = "example"
            secret = "s"
        "#;
        let config: McpClientsConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.clients[0].client_id, "example-id");
        assert_eq!(config.clients[0].name, "example");
    }

    #[test]
    fn duplicate_client_id_is_rejected() {
        let toml = r#"
            [[client]]
            client-id = "same-id"
            name = "first"
            secret = "s"

            [[client]]
            client-id = "same-id"
            name = "second"
            secret = "s"
        "#;
        let config: McpClientsConfig = toml::from_str(toml).unwrap();
        let err = config.ensure_unique_client_ids().unwrap_err();
        assert!(err.to_string().contains("same-id"), "{err}");
    }

    #[test]
    fn example_file_parses() {
        let raw = include_str!("../../mcp-clients.example.toml");
        let config: McpClientsConfig = toml::from_str(raw).unwrap();
        config.ensure_unique_client_ids().unwrap();
        assert_eq!(config.clients.len(), 3);
        assert_eq!(config.clients[0].name, "mcp-inspector");
        assert_eq!(config.clients[1].name, "example.ai");
        assert_eq!(config.clients[1].prefix, Some(ClientPrefix::Ai));
        assert_eq!(config.clients[2].name, "matrix-relay");
        assert_eq!(config.clients[2].on_behalf_of, None);
    }

    #[test]
    fn missing_on_behalf_of_defaults_to_none() {
        let toml = r#"
            [[client]]
            client-id = "matrix-relay-id"
            name = "matrix-relay"
            secret = "s"
        "#;
        let config: McpClientsConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.clients[0].on_behalf_of, None);
    }
}

