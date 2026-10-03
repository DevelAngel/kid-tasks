//! MCP (Model Context Protocol) server exposing task operations to AI assistants.
//! See `docs/adr/rmcp-mcp-server.md` for the design decisions
//! behind the tool/resource split, the assistant-identity handling,
//! and the separate-port transport choice.

use crate::oauth::{self, McpOAuthStore};
use crate::{SharedTaskCache, SharedTimeOffset};

use kid_app::DeadlineGroup;
use kid_app::server::{
    QuickWinsGroups, RecentChange, UpcomingBacklog, UpcomingGroups, group_quick_wins,
    group_recently_changed_since, group_upcoming,
};
use kid_types::task::Details as TaskDetails;
use kid_types::task::DetailsPatch as TaskDetailsPatch;
use kid_types::{Task, TaskAssignee, TaskCategory, TaskContext, TaskInfos, TaskPriority, TaskSummary, Uuid};

use chrono::{Datelike, Weekday};

use axum::Router;
use axum::middleware;
use axum::routing::{get, post};

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{
    CacheScope, CallToolResult, ContentBlock, ErrorData as McpError, Implementation,
    ListResourcesResult, PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResponse,
    ReadResourceResult, Resource, ResourceContents, ResultType, ServerCapabilities, ServerInfo,
};
use rmcp::schemars::{self, JsonSchema};
use rmcp::service::RequestContext;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{RoleServer, ServerHandler, tool, tool_handler, tool_router};

use indexmap::IndexSet;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use url::Url;

use std::collections::BTreeSet;
use std::sync::Arc;

/// Standalone MCP server, kept on its own port.
/// See `docs/adr/rmcp-mcp-server.md` — "Transport and port".
pub struct McpServer;

/// MCP server exposing task tools and read-only reference resources.
#[derive(Clone)]
pub struct McpService {
    task_cache: SharedTaskCache,
    time_offset: SharedTimeOffset,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ListInput {
    /// Fuzzy search query, matched against summary/category/contexts
    #[serde(default)]
    search: Option<String>,
    /// Filter by status (default: open)
    #[serde(default)]
    status: StatusFilter,
}

#[derive(Debug, Serialize, JsonSchema)]
struct ListOutput {
    entries: Vec<TaskEntry>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum StatusFilter {
    #[default]
    Open,
    Done,
    All,
}

#[derive(Debug, Serialize, JsonSchema)]
struct TaskEntry {
    id: Uuid,
    task: Task,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AddInput {
    /// Short task title
    summary: String,
    /// Life-domain category, e.g. "Household", "Finance" — see kid://categories
    category: String,
    /// GTD-style contexts, each starting with '#' — see kid://contexts
    #[serde(default)]
    contexts: Vec<String>,
    /// Optional task details (due_date, start_date, time_estimate, notes, availability)
    #[serde(default)]
    details: Option<TaskDetails>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RenameInput {
    id: Uuid,
    /// New summary — this is the task's identity, use `update` for other changes
    summary: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ReplaceInput {
    id: Uuid,
    /// Full replacement details — omitted fields become unset
    details: TaskDetails,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpdateInput {
    id: Uuid,
    /// Patch — omitted fields stay unchanged, explicit null clears a field
    details: TaskDetailsPatch,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct CompleteInput {
    id: Uuid,
    /// Reopen the task instead of completing it
    #[serde(default)]
    reopen: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct RecategorizeInput {
    id: Uuid,
    category: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ContextsInput {
    id: Uuid,
    /// Contexts, each starting with '#'
    #[serde(default)]
    contexts: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SetPriorityInput {
    id: Uuid,
    /// "A", "B", or "C" — omit or null to clear
    #[serde(default)]
    priority: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SetAssigneeInput {
    id: Uuid,
    /// Mention-style, e.g. "@mia" — omit or null to clear
    #[serde(default)]
    assignee: Option<String>,
}

impl McpServer {
    pub async fn serve(
        listener: TcpListener,
        shutdown: CancellationToken,
        task_cache: SharedTaskCache,
        time_offset: SharedTimeOffset,
        base_url: Url,
        allowed_origins: Vec<Url>,
        clients: crate::oauth::McpClientsConfig,
    ) -> Result<()> {
        tracing::info!(
            "MCP server listening on: http://{}",
            listener.local_addr().unwrap()
        );

        // The server's own base URL must be allowed too, since it issues
        // requests to itself (e.g. OAuth metadata, self-registration).
        let all_origins: Vec<Url> = std::iter::once(base_url.clone())
            .chain(allowed_origins)
            .collect();
        let allowed_hosts: Vec<String> = all_origins.iter().filter_map(host_header).collect();
        let allowed_origins: Vec<String> = all_origins
            .iter()
            .map(|url| url.origin().ascii_serialization())
            .collect();

        let mcp_service = StreamableHttpService::new(
            move || Ok(McpService::new(task_cache.clone(), time_offset.clone())),
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default()
                .with_allowed_origins(allowed_origins)
                .with_allowed_hosts(allowed_hosts)
                .with_cancellation_token(shutdown.child_token()),
        );

        let oauth_store = Arc::new(McpOAuthStore::new(base_url, clients));
        tokio::spawn({
            let oauth_store = oauth_store.clone();
            let shutdown = shutdown.child_token();
            async move { oauth_store.background_cleanup(shutdown).await }
        });

        let protected_mcp_router =
            Router::new()
                .nest_service("/mcp", mcp_service)
                .layer(middleware::from_fn_with_state(
                    oauth_store.clone(),
                    oauth::validate_access_token,
                ));

        let oauth_server_router = Router::new()
            .route(
                "/.well-known/oauth-authorization-server",
                get(oauth::auth_server).options(oauth::auth_server),
            )
            .route(
                "/.well-known/oauth-protected-resource",
                get(oauth::protected_resource).options(oauth::protected_resource),
            )
            .route(
                "/.well-known/oauth-protected-resource/mcp",
                get(oauth::protected_resource).options(oauth::protected_resource),
            )
            .route("/authorize", get(oauth::authorize))
            .route("/oauth/approve", post(oauth::approve))
            .route(
                "/token",
                post(oauth::gen_access_token).options(oauth::gen_access_token),
            )
            .with_state(oauth_store.clone());

        // In deployment, static assets live under LEPTOS_SITE_ROOT; for
        // local dev (where that's unset) public/ is a sibling of server/
        // (this crate's manifest dir) in the workspace.
        let favicons_dir = std::env::var("LEPTOS_SITE_ROOT")
            .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../public").to_owned());
        let favicons = ServeDir::new(favicons_dir);

        let app = Router::new()
            .merge(protected_mcp_router)
            .merge(oauth_server_router)
            .fallback_service(favicons)
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                shutdown.cancelled().await;
                tracing::info!("MCP server shutting down");
            })
            .await
            .into_diagnostic()?;
        Ok(())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for McpService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("kid-mcp", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "Family task manager. Every task needs a category and should \
             have at least one context (GTD-style, starting with '#'). \
             Fetch kid://categories and kid://contexts to reuse existing \
             values before inventing new ones.",
        )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        use indoc::indoc;
        const TTL_MS: u64 = 10 * 60 * 1000;

        let mut resources = vec![
            Resource::new(self::categories::URI, self::categories::NAME)
                .with_description("Categories currently in use")
                .with_mime_type("application/json"),
            Resource::new(self::contexts::URI, self::contexts::NAME)
                .with_description("Contexts currently in use")
                .with_mime_type("application/json"),
            Resource::new(self::weekly_report::URI, self::weekly_report::NAME)
                .with_description(indoc! {"
                    Narrative weekly review, reflecting on the last 7 days of task activity without scores, streaks, or comparisons.
                    Falls back to a plain list of recent changes if sampling is unavailable.
                "})
                .with_mime_type("text/markdown"),
            Resource::new(self::daily_report::URI, self::daily_report::NAME)
                .with_description(
                    "Markdown summary of open tasks for today: grouped into \
                     Overdue, Today, Tomorrow, This Week, Next Week, Later, and \
                     Ready to Start. Use this to tell a human what's on their \
                     plate.",
                )
                .with_mime_type("text/markdown"),
            Resource::new(self::backlog::URI, self::backlog::NAME)
                .with_description(
                    "Markdown list of open tasks that have no due date and \
                     aren't ready to start yet.",
                )
                .with_mime_type("text/markdown"),
            Resource::new(self::quick_wins::URI, self::quick_wins::NAME)
                .with_description(
                    "Markdown list of open tasks that have a time estimate, \
                     grouped from shortest to longest. Use this to suggest \
                     something to knock out with whatever time is available.",
                )
                .with_mime_type("text/markdown"),
        ];

        // Per-assignee report variants, one per assignee currently in use,
        // plus an "unassigned" variant, always listed regardless of
        // whether any task currently has no assignee. Assignees aren't
        // known ahead of time, so they're enumerated from the cache, same
        // as categories/contexts above.
        let assignees: BTreeSet<TaskAssignee> = {
            let cache = self.task_cache.read().await;
            cache
                .iter()
                .filter_map(|(_, task)| task.info().assignee().cloned())
                .collect()
        };
        for assignee in assignees {
            AssigneeFilter::Assignee(assignee).push_resources(&mut resources);
        }
        AssigneeFilter::Unassigned.push_resources(&mut resources);


        Ok(ListResourcesResult {
            resources,
            next_cursor: None,
            meta: None,
            // since 2026-07-28 spec
            result_type: Some(ResultType::COMPLETE),
            ttl_ms: Some(TTL_MS),
            cache_scope: Some(CacheScope::Private),
        })
    }



    async fn read_resource(
        &self,
        ReadResourceRequestParams { uri, .. }: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let result = match uri.as_str() {
            self::categories::URI => {
                let cache = self.task_cache.read().await;
                let categories: BTreeSet<TaskCategory> = cache
                    .iter()
                    .map(|(_, task)| task.category().parse::<TaskCategory>().unwrap())
                    .collect();
                let json = serde_json::to_string(&categories).unwrap_or_default();
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(json, &uri).with_mime_type("application/json"),
                ]))
            }
            self::contexts::URI => {
                let cache = self.task_cache.read().await;
                let contexts: BTreeSet<TaskContext> = cache
                    .iter()
                    .flat_map(|(_, task)| task.info().contexts().iter().cloned())
                    .collect();
                let json = serde_json::to_string(&contexts).unwrap_or_default();
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(json, &uri).with_mime_type("application/json"),
                ]))
            }
            self::weekly_report::URI => {
                let markdown = {
                    const DAYS: u32 = 8;
                    let cache = self.task_cache.read().await;
                    let today = kid_app::time::today_at_offset(self.time_offset.get());
                    let changes = group_recently_changed_since(cache.iter(), today, DAYS);
                    render_recent_changes(changes, DAYS)
                };
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            self::daily_report::URI => {
                let markdown = {
                    let cache = self.task_cache.read().await;
                    let today = kid_app::time::today_at_offset(self.time_offset.get());
                    let (groups, _backlog) = group_upcoming(cache.iter(), today);
                    render_daily_report(groups)
                };
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            self::backlog::URI => {
                let markdown = {
                    let cache = self.task_cache.read().await;
                    let today = kid_app::time::today_at_offset(self.time_offset.get());
                    let (_groups, backlog) = group_upcoming(cache.iter(), today);
                    render_backlog(backlog)
                };
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            self::quick_wins::URI => {
                let markdown = {
                    let cache = self.task_cache.read().await;
                    let groups = group_quick_wins(cache.iter());
                    render_quick_wins(groups)
                };
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            uri_str if uri_str.starts_with(&format!("{}/", self::daily_report::URI)) => {
                let slug = &uri_str[self::daily_report::URI.len() + 1..];
                let filter: AssigneeFilter = slug.parse()?;
                let markdown = {
                    let cache = self.task_cache.read().await;
                    let today = kid_app::time::today_at_offset(self.time_offset.get());
                    let filtered = cache
                        .iter()
                        .filter(|(_, task)| filter.matches(task.info().assignee()));
                    let (groups, _backlog) = group_upcoming(filtered, today);
                    render_daily_report(groups)
                };
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            uri_str if uri_str.starts_with(&format!("{}/", self::backlog::URI)) => {
                let slug = &uri_str[self::backlog::URI.len() + 1..];
                let filter: AssigneeFilter = slug.parse()?;
                let markdown = {
                    let cache = self.task_cache.read().await;
                    let today = kid_app::time::today_at_offset(self.time_offset.get());
                    let filtered = cache
                        .iter()
                        .filter(|(_, task)| filter.matches(task.info().assignee()));
                    let (_groups, backlog) = group_upcoming(filtered, today);
                    render_backlog(backlog)
                };
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            uri_str if uri_str.starts_with(&format!("{}/", self::quick_wins::URI)) => {
                let slug = &uri_str[self::quick_wins::URI.len() + 1..];
                let filter: AssigneeFilter = slug.parse()?;
                let markdown = {
                    let cache = self.task_cache.read().await;
                    let filtered = cache
                        .iter()
                        .filter(|(_, task)| filter.matches(task.info().assignee()));
                    let groups = group_quick_wins(filtered);
                    render_quick_wins(groups)
                };
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }
            uri_str if uri_str.starts_with(&format!("{}/", self::weekly_report::URI)) => {
                let slug = &uri_str[self::weekly_report::URI.len() + 1..];
                let filter: AssigneeFilter = slug.parse()?;
                let markdown = {
                    const DAYS: u32 = 8;
                    let cache = self.task_cache.read().await;
                    let today = kid_app::time::today_at_offset(self.time_offset.get());
                    let filtered = cache
                        .iter()
                        .filter(|(_, task)| filter.matches(task.info().assignee()));
                    let changes = group_recently_changed_since(filtered, today, DAYS);
                    render_recent_changes(changes, DAYS)
                };
                Ok(ReadResourceResult::new(vec![
                    ResourceContents::text(markdown, &uri).with_mime_type("text/markdown"),
                ]))
            }


            _ => Err(McpError::resource_not_found(
                format!("unknown resource: {uri}"),
                None,
            )),
        }?;
        Ok(ReadResourceResponse::Complete(result))
    }
}

#[tool_router]
impl McpService {
    #[tool(
        description = "List tasks, optionally filtered by status and/or fuzzy search",
        annotations(title = "List Tasks", read_only_hint = true, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    async fn list(
        &self,
        Parameters(ListInput { search, status }): Parameters<ListInput>,
    ) -> Json<ListOutput> {
        let cache = self.task_cache.read().await;
        let entries: Vec<_> = cache
            .iter()
            .filter(|(_, task)| match status {
                StatusFilter::All => true,
                StatusFilter::Open => !task.is_done(),
                StatusFilter::Done => task.is_done(),
            })
            .collect();

        let entries: Vec<_> = match search.as_deref().map(str::trim).filter(|q| !q.is_empty()) {
            Some(query) => {
                let words: Vec<&str> = query.split_whitespace().collect();
                entries
                    .into_iter()
                    .filter(|(_, task)| {
                        let haystack = format!(
                            "{} {} {}",
                            task.summary(),
                            task.category(),
                            task.contexts()
                                .iter()
                                .map(|c| c.to_string())
                                .collect::<Vec<_>>()
                                .join(" "),
                        );
                        words
                            .iter()
                            .all(|w| sublime_fuzzy::best_match(w, &haystack).is_some())
                    })
                    .collect()
            }
            None => entries,
        };

        let entries = entries
            .into_iter()
            .map(|(id, task)| TaskEntry {
                id: *id,
                task: task.clone(),
            })
            .collect();

        Json(ListOutput { entries })
    }

    #[tool(
        description = "Add a new task. Returns the new task's id",
        annotations(title = "Add Task", read_only_hint = false, destructive_hint = false, idempotent_hint = false, open_world_hint = false)
    )]
    async fn add(
        &self,
        Parameters(AddInput {
            summary,
            category,
            contexts,
            details,
        }): Parameters<AddInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let summary: TaskSummary = summary.parse().map_err(parse_err)?;
        let category: TaskCategory = category.parse().map_err(parse_err)?;
        let contexts: IndexSet<TaskContext> = contexts
            .iter()
            .map(|s| s.parse().map_err(parse_err))
            .collect::<Result<_, _>>()?;
        let mut task = Task::new(summary)
            .with_category(category)
            .with_contexts(contexts);
        if let Some(details) = details {
            task = task.with_details(details);
        }
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        let id = cache.add(task, actor);
        Ok(CallToolResult::structured(serde_json::json!({ "id": id })))
    }

    #[tool(
        description = "Rename a task's summary — its identity. Use `update` for other changes",
        annotations(title = "Rename Task", read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    async fn rename(
        &self,
        Parameters(RenameInput { id, summary }): Parameters<RenameInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let summary: TaskSummary = summary.parse().map_err(parse_err)?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.rename(summary);
                Ok(CallToolResult::success(vec![ContentBlock::text("renamed")]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(
        description = "Replace all task details (PUT semantics — omitted fields cleared)",
        annotations(title = "Replace Task Details", read_only_hint = false, destructive_hint = true, idempotent_hint = true, open_world_hint = false)
    )]
    async fn replace(
        &self,
        Parameters(ReplaceInput { id, details }): Parameters<ReplaceInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.set_details(details);
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "replaced",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(
        description = "Patch task details (PATCH semantics — omitted fields unchanged)",
        annotations(title = "Patch Task Details", read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    async fn update(
        &self,
        Parameters(UpdateInput { id, details }): Parameters<UpdateInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.patch_details(details);
                Ok(CallToolResult::success(vec![ContentBlock::text("updated")]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(
        description = "Complete a task, or reopen it with reopen=true",
        annotations(title = "Complete/Reopen Task", read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    async fn complete(
        &self,
        Parameters(CompleteInput { id, reopen }): Parameters<CompleteInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                if reopen {
                    task.mark_todo();
                } else {
                    task.mark_done();
                }
                let msg = if reopen { "reopened" } else { "completed" };
                Ok(CallToolResult::success(vec![ContentBlock::text(msg)]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(
        description = "Change a task's category",
        annotations(title = "Change Task Category", read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    async fn recategorize(
        &self,
        Parameters(RecategorizeInput { id, category }): Parameters<RecategorizeInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let category: TaskCategory = category.parse().map_err(parse_err)?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.set_category(category);
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "recategorized",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(
        description = "Add contexts to a task, keeping existing ones",
        annotations(title = "Add Task Contexts", read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    async fn add_contexts(
        &self,
        Parameters(ContextsInput { id, contexts }): Parameters<ContextsInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let contexts: IndexSet<TaskContext> = contexts
            .iter()
            .map(|s| s.parse().map_err(parse_err))
            .collect::<Result<_, _>>()?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.extend_contexts(contexts);
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "contexts added",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(
        description = "Replace all of a task's contexts (empty list clears them)",
        annotations(title = "Replace Task Contexts", read_only_hint = false, destructive_hint = true, idempotent_hint = true, open_world_hint = false)
    )]
    async fn replace_contexts(
        &self,
        Parameters(ContextsInput { id, contexts }): Parameters<ContextsInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let contexts: IndexSet<TaskContext> = contexts
            .iter()
            .map(|s| s.parse().map_err(parse_err))
            .collect::<Result<_, _>>()?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                task.set_contexts(contexts);
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "contexts replaced",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(
        description = "Set or clear a task's priority",
        annotations(title = "Set Task Priority", read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    async fn set_priority(
        &self,
        Parameters(SetPriorityInput { id, priority }): Parameters<SetPriorityInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let priority: Option<TaskPriority> =
            priority.map(|s| s.parse().map_err(parse_err)).transpose()?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                match priority {
                    Some(p) => task.set_priority(p),
                    None => task.clear_priority(),
                }
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "priority set",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }

    #[tool(
        description = "Set or clear a task's assignee",
        annotations(title = "Set Task Assignee", read_only_hint = false, destructive_hint = false, idempotent_hint = true, open_world_hint = false)
    )]
    async fn set_assignee(
        &self,
        Parameters(SetAssigneeInput { id, assignee }): Parameters<SetAssigneeInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let assignee: Option<TaskAssignee> =
            assignee.map(|s| s.parse().map_err(parse_err)).transpose()?;
        let actor = Self::actor(&context)?;
        let mut cache = self.task_cache.write().await;
        match cache.get_mut(&id, actor) {
            Some(mut task) => {
                match assignee {
                    Some(a) => task.set_assignee(a),
                    None => task.clear_assignee(),
                }
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "assignee set",
                )]))
            }
            None => Ok(Self::not_found(id)),
        }
    }
}

impl McpService {
    pub fn new(task_cache: SharedTaskCache, time_offset: SharedTimeOffset) -> Self {
        Self {
            task_cache,
            time_offset,
            tool_router: Self::tool_router(),
        }
    }

    /// Derives the actor string `<name>:<on_behalf_of>` entirely from
    /// the `--mcp-clients-file` entry of the OAuth client that authenticated
    /// this request - neither piece is claimed by the client itself.
    ///
    /// The name is not the MCP handshake's self-reported
    /// `clientInfo.name`, since that's freely claimed by the client and
    /// thus unverified. It carries whatever prefix distinguishes it (e.g.
    /// `ai:claude-desktop` vs. `matrix-relay`), since not every client is an
    /// AI assistant. Likewise, `on_behalf_of` is fixed per client in that
    /// same file, not a tool parameter - a client claiming this for itself
    /// couldn't be trusted any more than it claiming its own identity.
    ///
    /// A client whose entry omits `on-behalf-of` is read-only: there's no
    /// human to attribute a change to, so this returns an error instead of
    /// making one up.
    fn actor(context: &RequestContext<RoleServer>) -> Result<String, McpError> {
        let extensions = context
            .extensions
            .get::<axum::http::request::Parts>()
            .map(|parts| &parts.extensions);
        let name = extensions
            .and_then(|ext| ext.get::<oauth::ClientName>())
            .map(|name| name.to_string())
            .unwrap_or_else(|| {
                tracing::warn!("no authenticated client name in request extensions");
                "unknown".to_string()
            });
        let prefix = extensions
            .and_then(|ext| ext.get::<oauth::Prefix>())
            .map(|prefix| format!("{prefix}:"))
            .unwrap_or_default();
        let Some(on_behalf_of) = extensions.and_then(|ext| ext.get::<oauth::OnBehalfOf>()) else {
            return Err(McpError::invalid_request(
                format!("client {prefix}{name} is read-only (no on-behalf-of configured)"),
                None,
            ));
        };
        Ok(format!("{prefix}{name}:{on_behalf_of}"))
    }

    fn not_found(id: Uuid) -> CallToolResult {
        CallToolResult::error(vec![ContentBlock::text(format!("task {id} not found"))])
    }
}

fn parse_err(e: &str) -> McpError {
    McpError::invalid_params(e.to_string(), None)
}

/// Selects tasks by assignee for the filtered report resources: either a
/// specific assignee, or explicitly unassigned tasks.
enum AssigneeFilter {
    Assignee(TaskAssignee),
    Unassigned,
}

impl AssigneeFilter {
    /// Reserved URI slug for the "no assignee" variant. Doesn't collide
    /// with real assignee slugs, which are validated to start with `@`
    /// before the sigil is stripped.
    const UNASSIGNED_SLUG: &'static str = "unassigned";

    fn matches(&self, assignee: Option<&TaskAssignee>) -> bool {
        match self {
            Self::Assignee(a) => assignee == Some(a),
            Self::Unassigned => assignee.is_none(),
        }
    }

    /// URI path slug: the assignee without its `@` sigil, or the
    /// reserved `unassigned` slug.
    fn slug(&self) -> String {
        match self {
            Self::Assignee(a) => a.to_string().trim_start_matches('@').to_string(),
            Self::Unassigned => Self::UNASSIGNED_SLUG.to_string(),
        }
    }

    fn name_suffix(&self) -> String {
        match self {
            Self::Assignee(a) => a.to_string(),
            Self::Unassigned => "Unassigned".to_string(),
        }
    }

    fn description_suffix(&self) -> String {
        match self {
            Self::Assignee(a) => format!("assigned to {a}"),
            Self::Unassigned => "with no assignee".to_string(),
        }
    }

    /// Pushes the four per-filter report resource entries (daily,
    /// backlog, quick wins, weekly) for this filter.
    fn push_resources(&self, resources: &mut Vec<Resource>) {
        let slug = self.slug();
        let name_suffix = self.name_suffix();
        let description_suffix = self.description_suffix();
        resources.push(
            Resource::new(
                format!("{}/{slug}", daily_report::URI),
                format!("Daily Report — {name_suffix}"),
            )
            .with_description(format!("Daily Report filtered to tasks {description_suffix}."))
            .with_mime_type("text/markdown"),
        );
        resources.push(
            Resource::new(
                format!("{}/{slug}", backlog::URI),
                format!("Backlog — {name_suffix}"),
            )
            .with_description(format!("Backlog filtered to tasks {description_suffix}."))
            .with_mime_type("text/markdown"),
        );
        resources.push(
            Resource::new(
                format!("{}/{slug}", quick_wins::URI),
                format!("Quick Wins — {name_suffix}"),
            )
            .with_description(format!("Quick Wins filtered to tasks {description_suffix}."))
            .with_mime_type("text/markdown"),
        );
        resources.push(
            Resource::new(
                format!("{}/{slug}", weekly_report::URI),
                format!("Weekly Review — {name_suffix}"),
            )
            .with_description(format!("Weekly Review filtered to tasks {description_suffix}."))
            .with_mime_type("text/markdown"),
        );
    }
}

impl std::str::FromStr for AssigneeFilter {
    type Err = McpError;

    /// Parses a URI path slug (e.g. `alice`, no `@` sigil, or the
    /// reserved `unassigned`) into an [`AssigneeFilter`].
    fn from_str(slug: &str) -> Result<Self, Self::Err> {
        match slug {
            Self::UNASSIGNED_SLUG => Ok(Self::Unassigned),
            slug => format!("@{slug}")
                .parse()
                .map(Self::Assignee)
                .map_err(parse_err),
        }
    }
}



pub(super) mod categories {
    pub const NAME: &str = "Categories";
    pub const URI: &str = "kid://categories";
}
pub(super) mod contexts {
    pub const NAME: &str = "Contexts";
    pub const URI: &str = "kid://contexts";
}
pub(super) mod weekly_report {
    pub const NAME: &str = "Weekly Review";
    pub const URI: &str = "kid://report/weekly";
}
pub(super) mod daily_report {
    pub const NAME: &str = "Daily Report";
    pub const URI: &str = "kid://report/daily";
}
pub(super) mod backlog {
    pub const NAME: &str = "Backlog";
    pub const URI: &str = "kid://report/backlog";
}
pub(super) mod quick_wins {
    pub const NAME: &str = "Quick Wins";
    pub const URI: &str = "kid://report/quick_wins";
}


/// Renders `changes` ([`group_recently_changed`]'s result) as Markdown,
/// grouped by day, most recent first. Fallback body for the weekly
/// review resource when sampling is unavailable.
fn render_recent_changes(changes: Vec<RecentChange>, days: u32) -> String {
    if changes.is_empty() {
        return format!("No task activity in the last {days} days.\n");
    }

    let mut markdown = String::with_capacity(500);
    let mut current_day = None;
    for change in &changes {
        let day = change.last_changed.date_naive();
        if current_day != Some(day) {
            markdown.push_str(&format!("\n**{}**\n\n", day.format("%A, %-d %B")));
            current_day = Some(day);
        }
        markdown.push_str(&task_line(&change.info));
    }
    markdown
}

/// Renders `groups` (the dated-or-ready part of [`group_upcoming`]'s result)
/// as Markdown: one heading per [`DeadlineGroup`], with the same "This/Next
/// Weekend" relabeling and weekday/weekend divider as the Upcoming view when
/// every task in a `ThisWeek`/`NextWeek` group falls on a weekend.
fn render_daily_report(groups: UpcomingGroups) -> String {
    if groups.is_empty() {
        return "No open tasks due, or ready to start, today.\n".to_owned();
    }

    let mut markdown = String::with_capacity(500);
    for (group, tasks) in &groups {
        let all_weekend = matches!(group, DeadlineGroup::ThisWeek | DeadlineGroup::NextWeek)
            && tasks
                .iter()
                .all(|(_, _, _, date)| matches!(date.weekday(), Weekday::Sat | Weekday::Sun));
        let separator_idx = if matches!(group, DeadlineGroup::ThisWeek | DeadlineGroup::NextWeek) {
            tasks
                .iter()
                .position(|(_, _, _, date)| matches!(date.weekday(), Weekday::Sat | Weekday::Sun))
                .filter(|&idx| idx > 0)
        } else {
            None
        };

        markdown.push_str(&format!("\n**{}**\n\n", group.display_label(all_weekend)));
        for (idx, (_, info, ..)) in tasks.iter().enumerate() {
            if separator_idx == Some(idx) {
                markdown.push_str("---\n\n");
            }
            markdown.push_str(&task_line(info));
        }
    }

    markdown
}

/// Renders `backlog` (the dateless part of [`group_upcoming`]'s result) as
/// Markdown.
fn render_backlog(backlog: UpcomingBacklog) -> String {
    if backlog.is_empty() {
        return "No open tasks without a date.\n".to_owned();
    }

    let mut markdown = String::with_capacity(500);
    for (_, info) in &backlog {
        markdown.push_str(&task_line(info));
    }

    markdown
}

/// Renders `groups` ([`group_quick_wins`]'s result) as Markdown: one
/// heading per time estimate, shortest first.
fn render_quick_wins(groups: QuickWinsGroups) -> String {
    if groups.is_empty() {
        return "No open tasks have a time estimate.\n".to_owned();
    };

    let mut markdown = String::with_capacity(500);
    for (estimate, tasks) in groups {
        markdown.push_str(&format!("\n**{estimate}**\n\n"));
        for (_, ref task) in tasks {
            markdown.push_str(&task_line(task));
        }
    }

    markdown
}

/// Renders one task as a Markdown checkbox line, e.g.
/// `- [ ] Buy groceries _Household_`. Always unchecked: `group_upcoming`
/// only ever returns open tasks.
fn task_line<'a>(info: &'a impl TaskInfos<'a>) -> String {
    let summary = info.summary();
    let category = info.category();
    let category = if category.is_empty() {
        "⚠ no category"
    } else {
        category
    };
    if info.is_done() {
        format!("- ☑ {summary} _{category}_\n")
    } else {
        format!("- ☐ {summary} _{category}_\n")
    }
}

/// Derives a `Host` header value (`host` or `host:port`) from a configured
/// allowed origin, for `StreamableHttpServerConfig::with_allowed_hosts`.
fn host_header(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    Some(match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_owned(),
    })
}
