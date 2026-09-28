use rmcp::transport::auth::AuthorizationMetadata;
use secrecy::{ExposeSecret, SecretString};

use axum::Json;
use axum::body::{self, Body};
use axum::extract::{Form, Query, State};
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect, Response};

use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

use super::store::{ClientName, McpOAuthStore, OnBehalfOf, PkceChallenge, Prefix};

/// Query parameters of /authorize API call
#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    #[allow(dead_code)]
    response_type: String,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
}

/// Authorization HTML
#[derive(Template)]
#[template(path = "mcp_oauth_authorize.html")]
struct AuthorizeTemplate {
    client_id: String,
    redirect_uri: String,
    scope: String,
    state: String,
    scopes: String,
    code_challenge: String,
    code_challenge_method: String,
}

/// Query parameters of /approve API call
#[derive(Debug, Deserialize)]
pub struct ApprovalForm {
    client_id: String,
    redirect_uri: String,
    state: String,
    approved: String,
    #[serde(default)]
    code_challenge: String,
    // Round-tripped through the hidden form field for completeness,
    // but not read: `authorize` already rejected anything other than `S256`
    // (or no method at all) before this form could ever be rendered.
    #[allow(dead_code)]
    #[serde(default)]
    code_challenge_method: String,
}

#[derive(Deserialize)]
struct TokenRequest {
    grant_type: String,
    #[serde(default)]
    code: SecretString,
    #[allow(dead_code)]
    #[serde(default)]
    client_id: String,
    #[allow(dead_code)]
    #[serde(default)]
    client_secret: SecretString,
    #[allow(dead_code)]
    #[serde(default)]
    redirect_uri: String,
    #[serde(default)]
    code_verifier: Option<SecretString>,
    #[serde(default)]
    refresh_token: SecretString,
}

// Manual Debug: keeps non-secret fields visible for debugging while never
// touching the secret values.
impl std::fmt::Debug for TokenRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenRequest")
            .field("grant_type", &self.grant_type)
            .field("code", &"[REDACTED]")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("redirect_uri", &self.redirect_uri)
            .field(
                "code_verifier",
                &self.code_verifier.as_ref().map(|_| "[REDACTED]"),
            )
            .field("refresh_token", &"[REDACTED]")
            .finish()
    }
}

/// OAuth 2.0 Protected Resource Metadata (RFC 9728),
/// served under `/.well-known/oauth-protected-resource*`
/// so MCP clients can discover which authorization server(s)
/// to use for this resource.
#[derive(Debug, Serialize)]
struct ProtectedResourceMetadata {
    resource: String,
    authorization_servers: Vec<String>,
    bearer_methods_supported: Vec<String>,
    scopes_supported: Vec<String>,
}

pub async fn auth_server(State(state): State<Arc<McpOAuthStore>>) -> impl IntoResponse {
    tracing::debug!("client fetches metadata of authentication server");
    let base_url = state.base_url();
    let mut metadata = AuthorizationMetadata::default();
    metadata.registration_endpoint = Some(format!("{base_url}/register"));
    metadata.authorization_endpoint = format!("{base_url}/authorize");
    metadata.token_endpoint = format!("{base_url}/token");
    metadata.scopes_supported = Some(vec!["MCP".to_owned()]);
    metadata.response_types_supported = Some(vec!["code".to_owned()]);
    metadata.code_challenge_methods_supported = Some(vec!["S256".to_owned()]);
    metadata.issuer = Some(base_url);
    metadata.additional_fields = HashMap::from([(
        "grant_types_supported".into(),
        json!(["authorization_code", "client_credentials", "refresh_token"]),
    )]);
    tracing::debug!("metadata: {:?}", metadata);
    (StatusCode::OK, Json(metadata))
}

pub async fn protected_resource(State(state): State<Arc<McpOAuthStore>>) -> impl IntoResponse {
    tracing::debug!("client fetches protected-resource metadata");

    let base_url = state.base_url();
    let metadata = ProtectedResourceMetadata {
        resource: format!("{base_url}/mcp"),
        authorization_servers: vec![base_url],
        bearer_methods_supported: vec!["header".to_owned()],
        scopes_supported: vec!["MCP".to_owned()],
    };

    (StatusCode::OK, Json(metadata))
}

pub async fn authorize(
    Query(params): Query<AuthorizeQuery>,
    State(state): State<Arc<McpOAuthStore>>,
) -> impl IntoResponse {
    tracing::debug!("client asks for user authorization");

    // check if client is registered
    let client = &params.client_id;
    let Some(client) = state.client_registered(client).await else {
        tracing::warn!("client {client} not registered, skipping authorize rendering");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "unregistered client id"
            })),
        )
            .into_response();
    };

    // compare redirect uris
    if client.redirect_uri != params.redirect_uri {
        tracing::warn!(
            "client {client} registered with different redirect uri, skipping authorize rendering",
            client = client.client_id
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "unregistered redirect uri"
            })),
        )
            .into_response();
    }

    // check response type
    if params.response_type != "code" {
        tracing::warn!(
            "client {client} wants to use unsupported response type {response}, skipping authorize rendering",
            client = client.client_id,
            response = params.response_type
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "unsupported response type"
            })),
        )
            .into_response();
    }

    // reject `plain` PKCE (and the implicit "plain" default when no method is
    // given at all): it offers no protection over a bare authorization code,
    // so only S256 is accepted.
    if params.code_challenge.is_some() && params.code_challenge_method.as_deref() != Some("S256") {
        tracing::warn!(
            "client {client} used unsupported code_challenge_method {method:?}, rejecting",
            client = client.client_id,
            method = params.code_challenge_method
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_request",
                "error_description": "only S256 code_challenge_method is supported"
            })),
        )
            .into_response();
    }

    // render HTML
    let template = AuthorizeTemplate {
        client_id: params.client_id,
        redirect_uri: params.redirect_uri,
        scope: params.scope.clone().unwrap_or_default(),
        state: params.state.clone().unwrap_or_default(),
        scopes: params
            .scope
            .clone()
            .unwrap_or_else(|| "(no scope)".to_string()),
        code_challenge: params.code_challenge.clone().unwrap_or_default(),
        code_challenge_method: params.code_challenge_method.clone().unwrap_or_default(),
    };
    Html(template.render().unwrap()).into_response()
}

pub async fn approve(
    State(state): State<Arc<McpOAuthStore>>,
    Form(form): Form<ApprovalForm>,
) -> impl IntoResponse {
    let mut redirect_url = if form.approved == "true" {
        tracing::info!("user approved the authorization request");
        // `code_challenge_method` was already validated as `S256` (or absent)
        // in `authorize`; the hidden form field just carries it through the
        // redirect to here, so it's not re-checked.
        let pkce = if form.code_challenge.is_empty() {
            None
        } else {
            Some(PkceChallenge {
                challenge: form.code_challenge.clone(),
            })
        };
        let auth_code = state.gen_auth_code(form.client_id, pkce).await;
        format!(
            "{uri}?code={code}",
            uri = form.redirect_uri,
            code = auth_code.expose_secret().as_simple()
        )
    } else {
        tracing::warn!("user rejected the authorization request");
        format!("{uri}?error=access_denied", uri = form.redirect_uri)
    };
    if !form.state.is_empty() {
        redirect_url.push_str("&state=");
        redirect_url.push_str(&form.state);
    }
    tracing::debug!("redirecting to: {}", redirect_url);
    Redirect::to(&redirect_url).into_response()
}

pub async fn gen_access_token(
    State(state): State<Arc<McpOAuthStore>>,
    request: Request<Body>,
) -> impl IntoResponse {
    tracing::debug!("client requests an access token");
    let request = match body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => {
            tracing::debug!("request body received ({} bytes)", bytes.len());
            bytes
        }
        Err(e) => {
            tracing::error!("cannot read request body: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": "can't read request body"
                })),
            )
                .into_response();
        }
    };

    let request = match serde_urlencoded::from_bytes::<TokenRequest>(&request) {
        Ok(form) => {
            tracing::debug!("successfully parsed form data: {:?}", form);
            form
        }
        Err(e) => {
            tracing::error!("cannot parse form data: {}", e);
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({
                    "error": "invalid_request",
                    "error_description": format!("can't parse form data: {}", e)
                })),
            )
                .into_response();
        }
    };

    if request.client_id.is_empty() {
        tracing::error!("empty client id detected");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_client",
                "error_description": "invalid client id"
            })),
        )
            .into_response();
    }

    let Some(client) = state.client_registered(&request.client_id).await else {
        tracing::warn!("invalid client id: {}", request.client_id);
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid_client",
                "error_description": "unregistered client id"
            })),
        )
            .into_response();
    };

    if let Some(client_secret) = client.client_secret.as_deref() {
        if request.client_secret.expose_secret() != client_secret {
            tracing::warn!("invalid secret for client {}", client.client_id);
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_client",
                    "error_description": "invalid secret"
                })),
            )
                .into_response();
        }
    } else {
        tracing::warn!("skipping secret comparison for client {}", client.client_id);
    }

    match request.grant_type.as_str() {
        "authorization_code" => {
            let auth_code = request.code.expose_secret();
            let Some(data) = state.validate_auth_code(auth_code).await else {
                tracing::warn!("invalid authorization code (client {})", request.client_id);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_grant",
                        "error_description": "invalid authorization code"
                    })),
                )
                    .into_response();
            };

            if let Some(pkce) = &data.pkce {
                let verified = request
                    .code_verifier
                    .as_ref()
                    .is_some_and(|verifier| pkce.verify(verifier.expose_secret()));
                if !verified {
                    tracing::warn!("PKCE verification failed for client {}", data.client_id);
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "error": "invalid_grant",
                            "error_description": "invalid code_verifier"
                        })),
                    )
                        .into_response();
                }
            }

            let token = state.gen_access_token(data.client_id).await;
            match serde_json::to_value(token) {
                Ok(token) => {
                    tracing::info!("successfully created access token");
                    (StatusCode::OK, Json(token)).into_response()
                }
                Err(e) => {
                    tracing::error!("failed to create access token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "server_error",
                            "error_description": format!("failed to create access token: {}", e)
                        })),
                    )
                        .into_response()
                }
            }
        }
        "client_credentials" => {
            // No authorization code or user approval involved - the client
            // secret comparison above *is* the authentication for this
            // grant, per RFC 6749 §4.4. Unlike authorization_code (where
            // PKCE can stand in for a public client without a secret), a
            // secretless client here would be authenticated by nothing at
            // all, so require one explicitly instead of relying on the
            // generic check above, which silently skips clients without a
            // configured secret.
            if client.client_secret.is_none() {
                tracing::warn!(
                    "client {} has no secret configured; client_credentials requires one",
                    client.client_id
                );
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_client",
                        "error_description": "client_credentials requires a client secret"
                    })),
                )
                    .into_response();
            }

            let token = state.gen_access_token(client.client_id).await;
            match serde_json::to_value(token) {
                Ok(token) => {
                    tracing::info!("successfully created access token via client_credentials");
                    (StatusCode::OK, Json(token)).into_response()
                }
                Err(e) => {
                    tracing::error!("failed to create access token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "server_error",
                            "error_description": format!("failed to create access token: {}", e)
                        })),
                    )
                        .into_response()
                }
            }
        }
        "refresh_token" => {
            let refresh_token = request.refresh_token.expose_secret();
            let Some(data) = state.validate_refresh_token(refresh_token).await else {
                tracing::warn!("invalid refresh token (client {})", request.client_id);
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "invalid_grant",
                        "error_description": "invalid refresh token"
                    })),
                )
                    .into_response();
            };

            let token = state.gen_access_token(data.client_id).await;
            match serde_json::to_value(token) {
                Ok(token) => {
                    tracing::info!("successfully recreated access token");
                    (StatusCode::OK, Json(token)).into_response()
                }
                Err(e) => {
                    tracing::error!("failed to recreate access token: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "server_error",
                            "error_description": format!("failed to recreate access token: {}", e)
                        })),
                    )
                        .into_response()
                }
            }
        }
        _ => {
            tracing::warn!("unsupported grant type: {}", request.grant_type);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "unsupported_grant_type",
                    "error_description": "only authorization_code, client_credentials and refresh_token are supported"
                })),
            )
                .into_response()
        }
    }
}

pub async fn validate_access_token(
    State(token_store): State<Arc<McpOAuthStore>>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    tracing::debug!("validate_token_middleware");
    let auth_header = request.headers().get("Authorization");
    let token = match auth_header {
        Some(header) => {
            let header_str = header.to_str().unwrap_or("");
            if let Some(stripped) = header_str.strip_prefix("Bearer ") {
                stripped.to_string()
            } else {
                tracing::warn!("incomplete auth header");
                return token_store.unauthorized();
            }
        }
        None => {
            tracing::warn!("missing auth header");
            return token_store.unauthorized();
        }
    };
    match token_store.validate_access_token(&token).await {
        Some(data) => {
            tracing::info!("valid access token (client {})", data.client_id);
            // Lets handlers (e.g. the MCP tool/resource router) derive the
            // acting client's name from the authenticated client's
            // configured entry instead of the self-reported (and thus unverified) MCP
            // `clientInfo.name` from the initialize handshake.
            request.extensions_mut().insert(ClientName::from(data.actor.name));
            if let Some(prefix) = data.actor.prefix {
                request.extensions_mut().insert(Prefix(prefix));
            }
            if let Some(on_behalf_of) = data.actor.on_behalf_of {
                request
                    .extensions_mut()
                    .insert(OnBehalfOf::from(on_behalf_of));
            }
            next.run(request).await
        }
        None => {
            tracing::warn!("invalid access token");
            token_store.unauthorized()
        }
    }
}
