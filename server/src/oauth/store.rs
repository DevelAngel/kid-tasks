use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use base64::Engine;
use chrono::{DateTime, Utc};
use derive_more::{Deref, Display, From, FromStr};
use rmcp::transport::auth::OAuthClientConfig;
use secrecy::ExposeSecret;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio::sync::RwLock;
use tokio::time;
use tokio_util::sync::CancellationToken;
use url::Url;
use uuid::Uuid;

use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::sync::Arc;

use super::config::{ClientPrefix, McpClientsConfig};

pub const ACCESS_TOKEN_EXPIRES_IN: u64 = 3600;

/// How long an authorization code stays redeemable, per RFC 6749 §4.1.2
/// ("MUST expire shortly after it is issued"). Also the single-use cutoff:
/// a code is removed on first (successful or expired) use.
pub const AUTH_CODE_EXPIRES_IN: u64 = 600;

/// How long a refresh token stays usable. Unlike access tokens, refresh
/// tokens are meant to be long-lived, so this is measured in days rather
/// than hours (30 days).
pub const REFRESH_TOKEN_EXPIRES_IN: u64 = 30 * 24 * 60 * 60;

/// Stores access and refresh tokens per client
#[derive(Clone, Debug)]
pub struct McpOAuthStore {
    /// This server's own public URL, used when issuing OAuth metadata
    /// (issuer, authorization/token endpoints).
    base_url: Url,
    clients: Arc<RwLock<HashMap<ClientId, OAuthClientConfig>>>,
    /// Actor-string components for each client, keyed by the same
    /// `ClientId` as `clients`. Kept separate since none of it is part of the
    /// OAuth protocol (`OAuthClientConfig`), just our own configuration.
    actors: Arc<RwLock<HashMap<ClientId, ActorConfig>>>,
    auth_sessions: Arc<RwLock<HashMap<AuthCode, AdditionalData>>>,
    access_tokens: Arc<RwLock<HashMap<AccessCode, AdditionalData>>>,
    refresh_tokens: Arc<RwLock<HashMap<RefreshCode, AdditionalData>>>,
}

#[derive(Clone, Debug, Deref, Display, Eq, From, Hash, PartialEq)]
#[from(forward)]
pub struct ClientId(String);

/// Actor-string components of a client's `--mcp-clients-file` entry:
/// `name`, `prefix` (e.g. distinguishing AI assistants) and the human it
/// acts on behalf of. None of them is the OAuth client_id it authenticates
/// with - all are stashed into request extensions by
/// [`crate::oauth::validate_access_token`] alongside [`ClientId`], so
/// handlers don't need to trust a tool parameter for any of them.
#[derive(Clone, Debug, PartialEq)]
pub struct ActorConfig {
    pub name: String,
    pub prefix: Option<ClientPrefix>,
    /// `None` marks the client read-only: with no human to attribute a
    /// change to, mutating tools must reject it (see [`OnBehalfOf`]).
    pub on_behalf_of: Option<String>,
}

/// Human a client acts on behalf of, per its `--mcp-clients-file` entry.
/// Only present in request extensions for clients whose entry configures
/// `on-behalf-of`; its absence marks a client read-only.
#[derive(Clone, Debug, Deref, Display, Eq, From, Hash, PartialEq)]
#[from(forward)]
pub struct OnBehalfOf(String);

/// Name of a client in its actor string, per its `--mcp-clients-file`
/// entry. Distinct from [`ClientId`], which is only used to authenticate.
#[derive(Clone, Debug, Deref, Display, Eq, From, Hash, PartialEq)]
#[from(forward)]
pub struct ClientName(String);

/// AI-assistant-distinguishing prefix of a client's actor string, per its
/// `--mcp-clients-file` entry. Only present in request extensions for
/// clients whose entry configures one.
#[derive(Clone, Debug, From, PartialEq)]
pub struct Prefix(pub ClientPrefix);

impl Display for Prefix {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Eq, From, FromStr, Hash, PartialEq)]
#[from(forward)]
pub struct AuthCode(Uuid);

#[derive(Clone, Eq, From, FromStr, Hash, PartialEq)]
#[from(forward)]
pub struct AccessCode(Uuid);

#[derive(Clone, Eq, From, FromStr, Hash, PartialEq)]
#[from(forward)]
pub struct RefreshCode(Uuid);

// No `Display`/`Deref` here on purpose: both would let `{}` or a
// `.as_simple()` deref-call print the raw secret. Access is only possible
// through `ExposeSecret::expose_secret`, mirroring how `secrecy` gates
// access to `SecretString`/`SecretBox`.
impl ExposeSecret<Uuid> for AuthCode {
    fn expose_secret(&self) -> &Uuid {
        &self.0
    }
}

impl ExposeSecret<Uuid> for AccessCode {
    fn expose_secret(&self) -> &Uuid {
        &self.0
    }
}

impl ExposeSecret<Uuid> for RefreshCode {
    fn expose_secret(&self) -> &Uuid {
        &self.0
    }
}

impl Debug for AuthCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AuthCode").field(&"[REDACTED]").finish()
    }
}

impl Debug for AccessCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("AccessCode").field(&"[REDACTED]").finish()
    }
}

impl Debug for RefreshCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RefreshCode").field(&"[REDACTED]").finish()
    }
}

impl Default for AuthCode {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AccessCode {
    fn default() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RefreshCode {
    fn default() -> Self {
        Self(Uuid::now_v7())
    }
}

/// PKCE challenge (RFC 7636) attached to an authorization code,
/// checked against the `code_verifier` sent at the token endpoint.
///
/// Note: Only the `S256` method is supported;
///       `plain` is rejected at `/authorize` time since it provides
///       no real protection over a bare authorization code.
#[derive(Clone, Debug)]
pub struct PkceChallenge {
    pub challenge: String,
}

impl PkceChallenge {
    /// Returns `true` if `verifier` hashes to this challenge.
    pub fn verify(&self, verifier: &str) -> bool {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let digest = Sha256::digest(verifier.as_bytes());
        let computed = URL_SAFE_NO_PAD.encode(digest);
        computed == self.challenge
    }
}

#[derive(Clone, Debug)]
pub struct AdditionalData {
    created_at: DateTime<Utc>,
    pub client_id: ClientId,
    pub actor: ActorConfig,
    pub pkce: Option<PkceChallenge>,
}

impl AdditionalData {
    fn new(client_id: ClientId, actor: ActorConfig) -> Self {
        Self::with_pkce(client_id, actor, None)
    }

    fn with_pkce(client_id: ClientId, actor: ActorConfig, pkce: Option<PkceChallenge>) -> Self {
        let created_at = Utc::now();
        Self {
            created_at,
            client_id,
            actor,
            pkce,
        }
    }

    /// Whether this entry has outlived `ttl_secs`, counted from issuance.
    fn is_older_than(&self, ttl_secs: u64) -> bool {
        let age = Utc::now().signed_duration_since(self.created_at);
        age.num_seconds() >= ttl_secs as i64
    }

    /// Whether this entry's access token has outlived
    /// [`ACCESS_TOKEN_EXPIRES_IN`], counted from issuance.
    fn is_expired(&self) -> bool {
        self.is_older_than(ACCESS_TOKEN_EXPIRES_IN)
    }
}

#[derive(Clone, Serialize)]
pub struct TokenAnswer {
    pub token_type: String,
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
}

impl Debug for TokenAnswer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenAnswer")
            .field("token_type", &self.token_type)
            .field("access_token", &"[REDACTED]")
            .field("expires_in", &self.expires_in)
            .field("refresh_token", &"[REDACTED]")
            .finish()
    }
}

impl TokenAnswer {
    fn new(access_token: AccessCode, refresh_token: RefreshCode) -> Self {
        let conv = |token: &Uuid| token.as_simple().to_string();
        Self {
            token_type: "Bearer".to_owned(),
            access_token: conv(access_token.expose_secret()),
            expires_in: ACCESS_TOKEN_EXPIRES_IN,
            refresh_token: conv(refresh_token.expose_secret()),
        }
    }
}

impl ClientId {
    pub fn from_config(config: OAuthClientConfig) -> (Self, OAuthClientConfig) {
        let client_id = config.client_id.clone().into();
        (client_id, config)
    }
}

impl McpOAuthStore {
    pub fn new(base_url: Url, clients: McpClientsConfig) -> Self {
        let mut actors = HashMap::new();
        let clients = clients
            .clients
            .into_iter()
            .map(|client| {
                let redirect_uri = client
                    .redirect_uri
                    .as_ref()
                    .map(|uri| uri.to_string())
                    .unwrap_or_default();
                // Authentication uses `client-id` only: `name` and `prefix`
                // only matter for the actor string.
                let (client_id, config) = ClientId::from_config(
                    OAuthClientConfig::new(client.client_id, redirect_uri)
                        .with_client_secret(client.secret.expose_secret())
                        .with_scopes(vec!["MCP".to_string()]),
                );
                actors.insert(
                    client_id.clone(),
                    ActorConfig {
                        name: client.name,
                        prefix: client.prefix,
                        on_behalf_of: client.on_behalf_of,
                    },
                );
                (client_id, config)
            })
            .collect();

        Self {
            base_url,
            clients: Arc::new(RwLock::new(clients)),
            actors: Arc::new(RwLock::new(actors)),
            auth_sessions: Arc::new(RwLock::new(HashMap::new())),
            access_tokens: Arc::new(RwLock::new(HashMap::new())),
            refresh_tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Actor-string components for `client_id`, per its
    /// `--mcp-clients-file` entry, or the read-only default if `client_id`
    /// isn't registered at all (shouldn't happen for a client that already
    /// made it past authentication).
    async fn actor_config(&self, client_id: &ClientId) -> ActorConfig {
        self.actors
            .read()
            .await
            .get(client_id)
            .cloned()
            .unwrap_or_else(|| ActorConfig {
                name: "unknown".to_owned(),
                prefix: None,
                on_behalf_of: None,
            })
    }

    /// This server's own base URL without a trailing slash,
    /// e.g. `https://mcp.example.com`.
    pub fn base_url(&self) -> String {
        self.base_url.origin().ascii_serialization()
    }

    pub async fn client_registered(
        &self,
        client_id: impl Into<ClientId>,
    ) -> Option<OAuthClientConfig> {
        let clients = self.clients.read().await;
        clients.get(&client_id.into()).map(|c| c.to_owned())
    }

    pub async fn gen_auth_code(
        &self,
        client_id: impl Into<ClientId>,
        pkce: Option<PkceChallenge>,
    ) -> AuthCode {
        let client_id = client_id.into();
        let actor = self.actor_config(&client_id).await;
        let data = AdditionalData::with_pkce(client_id, actor, pkce);
        let code = AuthCode::default();

        let mut auth_sessions = self.auth_sessions.write().await;
        auth_sessions.insert(code.clone(), data);

        code
    }

    /// Validates and consumes an authorization code: it's removed from the
    /// store either way, since a code is single-use per RFC 6749 §4.1.2 and
    /// must not be redeemable again after an expired or failed attempt.
    pub async fn validate_auth_code(&self, code: &str) -> Option<AdditionalData> {
        let Ok(code) = code.parse::<AuthCode>() else {
            tracing::warn!("invalid authorization code: failed to parse");
            return None;
        };

        let mut auth_sessions = self.auth_sessions.write().await;
        let data = auth_sessions.remove(&code)?;

        if data.is_older_than(AUTH_CODE_EXPIRES_IN) {
            tracing::debug!("authorization code expired");
            return None;
        }

        Some(data)
    }

    pub async fn gen_access_token(&self, client_id: impl Into<ClientId>) -> TokenAnswer {
        let client_id = client_id.into();
        let actor = self.actor_config(&client_id).await;
        let data = AdditionalData::new(client_id, actor);
        let access_token = AccessCode::default();
        let refresh_token = RefreshCode::default();

        let (mut access_tokens, mut refresh_tokens) =
            tokio::join!(self.access_tokens.write(), self.refresh_tokens.write());
        access_tokens.insert(access_token.clone(), data.clone());
        refresh_tokens.insert(refresh_token.clone(), data);

        TokenAnswer::new(access_token, refresh_token)
    }

    pub async fn validate_access_token(&self, token: &str) -> Option<AdditionalData> {
        let Ok(token) = token.parse::<AccessCode>() else {
            tracing::warn!("invalid access token: failed to parse");
            return None;
        };

        let mut access_tokens = self.access_tokens.write().await;
        let data = access_tokens.get(&token)?;

        if data.is_expired() {
            tracing::debug!("access token expired");
            access_tokens.remove(&token);
            return None;
        }

        Some(data.to_owned())
    }

    /// Removes access tokens older than [`ACCESS_TOKEN_EXPIRES_IN`].
    ///
    /// Refresh tokens are unaffected — they're long-lived by design.
    async fn cleanup_expired_access_tokens(&self) {
        let mut access_tokens = self.access_tokens.write().await;
        let before = access_tokens.len();
        access_tokens.retain(|_, data| !data.is_expired());
        let removed = before - access_tokens.len();
        if removed > 0 {
            tracing::debug!("cleaned up {removed} expired access token(s)");
        }
    }

    /// Removes authorization codes older than [`AUTH_CODE_EXPIRES_IN`].
    ///
    /// Codes that *are* redeemed are already removed by
    /// [`Self::validate_auth_code`] on use; this only catches codes a client
    /// never came back for, so the map doesn't grow unbounded.
    async fn cleanup_expired_auth_codes(&self) {
        let mut auth_sessions = self.auth_sessions.write().await;
        let before = auth_sessions.len();
        auth_sessions.retain(|_, data| !data.is_older_than(AUTH_CODE_EXPIRES_IN));
        let removed = before - auth_sessions.len();
        if removed > 0 {
            tracing::debug!("cleaned up {removed} expired authorization code(s)");
        }
    }

    /// Removes refresh tokens older than [`REFRESH_TOKEN_EXPIRES_IN`].
    ///
    /// Expired refresh tokens are already rejected on use by
    /// [`Self::validate_refresh_token`]; this only catches tokens whose
    /// client never came back to use them, so the map doesn't grow
    /// unbounded.
    async fn cleanup_expired_refresh_tokens(&self) {
        let mut refresh_tokens = self.refresh_tokens.write().await;
        let before = refresh_tokens.len();
        refresh_tokens.retain(|_, data| !data.is_older_than(REFRESH_TOKEN_EXPIRES_IN));
        let removed = before - refresh_tokens.len();
        if removed > 0 {
            tracing::debug!("cleaned up {removed} expired refresh token(s)");
        }
    }

    /// Periodically sweeps expired access tokens, authorization codes, and
    /// refresh tokens until `shutdown` fires.
    ///
    /// This is a housekeeping task, not a security boundary: expired
    /// entries are already rejected on use by [`Self::validate_access_token`],
    /// [`Self::validate_auth_code`], and [`Self::validate_refresh_token`]. It
    /// just keeps the maps from growing unbounded between requests.
    pub async fn background_cleanup(&self, shutdown: CancellationToken) {
        let mut interval = time::interval(time::Duration::from_secs(AUTH_CODE_EXPIRES_IN));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.cleanup_expired_access_tokens().await;
                    self.cleanup_expired_auth_codes().await;
                    self.cleanup_expired_refresh_tokens().await;
                }
                () = shutdown.cancelled() => {
                    tracing::debug!("stopping MCP OAuth cleanup task");
                    break;
                }
            }
        }
    }

    /// Validates a refresh token, rejecting (and removing) it if expired.
    ///
    /// Unlike authorization codes, a refresh token is *not* consumed here —
    /// it stays valid for repeated use until it expires, per the existing
    /// `refresh_token` grant behavior.
    pub async fn validate_refresh_token(&self, token: &str) -> Option<AdditionalData> {
        let Ok(token) = token.parse::<RefreshCode>() else {
            tracing::warn!("invalid refresh token: failed to parse");
            return None;
        };

        let mut refresh_tokens = self.refresh_tokens.write().await;
        let data = refresh_tokens.get(&token)?;

        if data.is_older_than(REFRESH_TOKEN_EXPIRES_IN) {
            tracing::debug!("refresh token expired");
            refresh_tokens.remove(&token);
            return None;
        }

        Some(data.to_owned())
    }

    /// Builds a 401 response carrying a `WWW-Authenticate` header that points
    /// clients at this resource's protected-resource metadata (RFC 9728 §5.1),
    /// so they know where to discover the authorization server to use.
    pub fn unauthorized(&self) -> Response {
        let metadata_url = format!(
            "{}/.well-known/oauth-protected-resource/mcp",
            self.base_url()
        );
        (
            StatusCode::UNAUTHORIZED,
            [(
                axum::http::header::WWW_AUTHENTICATE,
                format!(r#"Bearer resource_metadata="{metadata_url}""#),
            )],
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> McpOAuthStore {
        let toml = r#"
            [[client]]
            client-id = "example-id"
            name = "example"
            secret = "s"
            on-behalf-of = "Jane"
        "#;
        let clients: McpClientsConfig = toml::from_str(toml).unwrap();
        McpOAuthStore::new("http://localhost".parse().unwrap(), clients)
    }

    #[tokio::test]
    async fn client_authenticates_by_client_id_only() {
        let store = store();
        assert!(store.client_registered("example-id").await.is_some());
        assert!(store.client_registered("example").await.is_none());
    }

    #[tokio::test]
    async fn issued_token_carries_name_not_client_id() {
        let store = store();
        let token = store.gen_access_token("example-id").await;
        let data = store
            .validate_access_token(&token.access_token)
            .await
            .unwrap();
        assert_eq!(data.actor.name, "example");
    }
}

