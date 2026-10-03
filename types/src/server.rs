use crate::Task;
use crate::Uuid;

use ahash::{HashMap, RandomState};
use indexmap::{IndexMap, IndexSet};
use miette::Diagnostic;
use thiserror::Error;

use std::collections::VecDeque;
use std::env;
use std::fs::{self, File};
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct TaskCache {
    dir: PathBuf,
    tasks: DataMap,
    dirty: ChangeSet,
}

#[derive(Debug)]
pub struct TaskMutGuard<'a> {
    id: Uuid,
    task: &'a mut Task,
    dirty: &'a mut ChangeSet,
    actor: String,
}

pub type TaskLoadResult<T> = Result<T, LoadErrors>;
pub type TaskFlushResult<T> = Result<T, FlushErrors>;

#[derive(Error, Diagnostic, Debug)]
#[error("loading of {failed}/{all} tasks failed")]
pub struct LoadErrors {
    failed: usize,
    all: usize,
    #[related]
    errors: VecDeque<LoadError>,
}

#[derive(Error, Diagnostic, Debug)]
pub enum LoadError {
    #[error("failed to access file system")]
    Fs(#[from] FsError),
    #[error("failed to load task")]
    Task(#[from] TaskError),
}

#[derive(Error, Diagnostic, Debug)]
#[error("flushing of {failed}/{all} tasks failed")]
pub struct FlushErrors {
    failed: usize,
    all: usize,
    #[related]
    fs_errors: VecDeque<FsError>,
    #[related]
    task_errors: VecDeque<TaskError>,
}

impl FlushErrors {
    pub fn failed(&self) -> usize {
        self.failed
    }
}

pub type FsResult<T> = Result<T, FsError>;
pub type TaskResult<T> = Result<T, TaskError>;

#[derive(Error, Diagnostic, Debug)]
pub enum FsError {
    #[error("failed to create directory recursively: {0}")]
    CreateDir(PathBuf, #[source] io::Error),
    #[error("failed to read directory: {0}")]
    ReadDir(PathBuf, #[source] io::Error),
    #[error("failed to read entry of directory {0}")]
    ReadDirEntry(PathBuf, #[source] io::Error),
}

#[derive(Error, Diagnostic, Debug)]
pub enum TaskError {
    #[cfg(any(feature = "ssr-test-storagefail", feature = "ssr-test-rand"))]
    #[error("task {0}: something failed (for testing purpose)")]
    Test(Uuid),
    #[error("task {id}: failed to open file {path}")]
    OpenFile {
        id: Uuid,
        path: PathBuf,
        #[source]
        error: io::Error,
    },
    #[error("task {id}: failed to create file {path}")]
    CreateFile {
        id: Uuid,
        path: PathBuf,
        #[source]
        error: io::Error,
    },
    #[error("task {id}: failed to load task from JSON file: {path}")]
    ReadJsonFile {
        id: Uuid,
        path: PathBuf,
        #[source]
        error: serde_json::Error,
    },
    #[error("task {id}: failed to write task to JSON file: {path}")]
    WriteJsonFile {
        id: Uuid,
        path: PathBuf,
        #[source]
        error: serde_json::Error,
    },
    #[error("task {id}: failed to remove file: {path}")]
    RemoveFile {
        id: Uuid,
        path: PathBuf,
        #[source]
        error: io::Error,
    },
    #[error("task {id}: failed to save task file (by renaming):\nfrom: {path_from}\nto: {path_to}")]
    RenameFile {
        id: Uuid,
        path_from: PathBuf,
        path_to: PathBuf,
        #[source]
        error: io::Error,
    },
    #[error("task {id}: failed to write task file: {path}")]
    WriteFile {
        id: Uuid,
        path: PathBuf,
        #[source]
        error: Box<TaskError>,
    },
}

type DataMap = IndexMap<Uuid, Task, RandomState>;
type ChangeSet = IndexSet<Uuid, RandomState>;

impl TaskCache {
    fn has_valid_id(id: &Uuid) -> bool {
        id.get_version_num() == 7
    }

    fn create_id() -> Uuid {
        let id = Uuid::now_v7();
        assert_eq!(id.get_version_num(), 7, "invalid UUID");
        id
    }

    fn with_capacity(capacity: usize) -> Self {
        let path = env::current_dir().expect("CWD available");
        Self {
            dir: path,
            tasks: DataMap::with_capacity_and_hasher(capacity, RandomState::new()),
            dirty: ChangeSet::with_hasher(RandomState::new()),
        }
    }

    pub fn with_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir = dir.into();
        assert!(self.dir.is_dir(), "invalid directory");
        self
    }

    /// Clear all tasks and point at a new directory.
    ///
    /// **Caller must flush before calling this** if there are
    /// unsaved changes — reset discards dirty state silently.
    pub fn reset(&mut self, dir: impl Into<PathBuf>) {
        self.dir = dir.into();
        assert!(self.dir.is_dir(), "invalid directory: {}", self.dir.display());
        self.tasks.clear();
        self.dirty.clear();
    }

    pub fn remove(&mut self, id: &Uuid) -> bool {
        assert_eq!(id.get_version_num(), 7, "invalid UUID");
        self.dirty.insert(*id);
        self.tasks.shift_remove(id).is_some()
    }

    pub fn add(&mut self, mut task: Task, actor: impl Into<String>) -> Uuid {
        task.add_author(actor.into());
        let id = TaskCache::create_id();
        self.dirty.insert(id);
        self.tasks.insert(id, task);
        id
    }

    pub fn add_with_id(&mut self, id: Uuid, mut task: Task, actor: impl Into<String>) {
        assert!(Self::has_valid_id(&id), "id must be UUID v7");
        task.add_author(actor.into());
        self.dirty.insert(id);
        self.tasks.insert(id, task);
    }

    /*
     * Note: get() is accessable via Deref trait
     */

    pub fn get_mut(&mut self, id: &Uuid, actor: impl Into<String>) -> Option<TaskMutGuard<'_>> {
        let dirty = &mut self.dirty;
        let actor = actor.into();
        self.tasks.get_mut(id).map(|task| TaskMutGuard {
            id: *id,
            dirty,
            task,
            actor,
        })
    }

    pub async fn load(&mut self) -> TaskLoadResult<(usize, usize)> {
        let mut num_loaded = 0;
        let mut num_to_migrate = 0;
        let mut errors = VecDeque::new();
        let dir = fs::read_dir(self.dir.as_path())
            .map_err(|e| FsError::ReadDir(self.dir.clone(), e))
            .map_err(|e| LoadErrors {
                failed: 0,
                all: 0,
                errors: [e.into()].into(),
            })?;
        for entry in dir {
            let Ok(entry) = entry.map_err(|e| {
                let e = FsError::ReadDirEntry(self.dir.clone(), e);
                errors.push_back(e.into());
            }) else {
                continue;
            };

            let entry = entry.path();
            let Some(id) = Self::is_task_file(&entry) else {
                // no task file
                continue;
            };

            let Ok((task, needs_migration)) = Self::read_task_file(&id, &entry).await.map_err(|e| {
                errors.push_back(e.into());
            }) else {
                continue;
            };
            
            if needs_migration {             
                self.dirty.insert(id);
                num_to_migrate += 1;
            }

            self.tasks.insert(id, task);
            num_loaded += 1;
        }

        if errors.is_empty() {
            Ok((num_loaded, num_to_migrate))
        } else {
            Err(LoadErrors {
                failed: errors.len(),
                all: num_loaded + errors.len(),
                errors,
            })
        }
    }

    pub async fn flush(&mut self) -> TaskFlushResult<usize> {
        if !self.dir.exists() {
            fs::create_dir_all(&self.dir)
                .map_err(|e| FsError::CreateDir(self.dir.clone(), e))
                .map_err(|e| FlushErrors {
                    failed: 0,
                    all: 0,
                    fs_errors: [e].into(),
                    task_errors: [].into(),
                })?;
        }
        assert!(self.dir.exists());
        assert!(self.dir.is_dir());

        let num = self.dirty.len();
        let mut errors = HashMap::default();

        for id in &self.dirty {
            if let Some(task) = self.tasks.get(id) {
                // Task found -> write file
                if let Err(e) = self.write_task_file(id, task).await {
                    errors.insert(*id, e);
                }
            } else {
                // Task deleted -> delete file
                if let Err(e) = self.delete_task_file(id).await {
                    errors.insert(*id, e);
                }
            }
        }

        if errors.is_empty() {
            self.dirty.clear();
            Ok(num)
        } else {
            // remove ids for successfully written files
            for id in self.dirty.clone().iter() {
                if !errors.contains_key(id) {
                    self.dirty.swap_remove(id);
                }
            }
            Err(FlushErrors {
                failed: self.dirty.len(),
                all: num,
                fs_errors: [].into(),
                task_errors: errors.into_values().collect(),
            })
        }
    }

    fn is_task_file<P: AsRef<Path>>(file: P) -> Option<Uuid> {
        let file = file.as_ref();
        if !file.is_file() {
            return None;
        }

        let filename = file.file_prefix().unwrap().display().to_string();
        let id = filename.strip_prefix("task-")?;

        // we have found a task file
        let Ok(id) = Uuid::try_parse(id) else {
            tracing::warn!("file with invalid UUID detected: {}", file.display());
            return None;
        };
        if !Self::has_valid_id(&id) {
            tracing::warn!("file with invalid UUIDv7 detected: {}", file.display());
            return None;
        }
        Some(id)
    }

    async fn read_task_file<P: AsRef<Path>>(id: &Uuid, path: P) -> TaskResult<(Task, bool)> {
        assert!(Self::has_valid_id(id));
        let path = path.as_ref();
        assert!(path.is_file());
        let raw = fs::read_to_string(path).map_err(|e| TaskError::OpenFile {
            id: *id,
            path: path.to_path_buf(),
            error: e,
        })?;
        let needs_migration = Self::detect_legacy_format(id, &raw);
        let task = serde_json::from_str(&raw).map_err(|e| TaskError::ReadJsonFile {
            id: *id,
            path: path.to_path_buf(),
            error: e,
        })?;
        Ok((task, needs_migration))
    }

    fn detect_legacy_format(id: &Uuid, json_str: &str) -> bool {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(json_str) else {
            return false;
        };

        // Legacy status: plain string instead of {"ToDo":{"since":"..."}}
        let legacy_status = v.get("status").map(|s| s.is_string()).unwrap_or(false);

        // Legacy time_estimate: {"Guess":"..."} or {"Precise":{"secs":...}}
        // New format is a plain string e.g. "Min30"
        // Guess is mapped to Day2 — warn so the lossy rounding is visible.
        let legacy_time_estimate = v
            .get("time_estimate")
            .map(|t| t.is_object())
            .unwrap_or(false);
        if let Some(guess) = v.get("time_estimate").and_then(|t| t.get("Guess")).and_then(|g| g.as_str()) {
            tracing::warn!(
                "task {id}: migrating legacy Guess time_estimate \"{guess}\" — best-effort parse; unrecognised values fall back to Day2"
            );
        }

        // Legacy date fields: {"Precise":"..."} or {"Guess":"..."}
        // Guess values are parsed best-effort (RFC3339 / date-only → soft=true); unparseable values are dropped.
        for field in ["due_date", "start_date"] {
            if let Some(guess) = v.get(field).and_then(|d| d.get("Guess")).and_then(|g| g.as_str()) {
                tracing::warn!(
                    "task {id}: migrating legacy Guess {field} \"{guess}\" — best-effort parse, soft=true; unparseable values are dropped"
                );
            }
        }

        let legacy_date = ["due_date", "start_date"]
            .iter()
            .any(|f| v.get(f).map(|d| d.get("Precise").is_some() || d.get("Guess").is_some()).unwrap_or(false));

        // Legacy field name: "context" was renamed to "category"
        let legacy_context_field = v.get("context").is_some();
        if legacy_context_field {
            tracing::info!("task {id}: migrating legacy \"context\" field to \"category\"");
        }

        // Legacy context prefix: contexts used to start with '@', now '#'
        let legacy_context_prefix = v
            .get("contexts")
            .and_then(|c| c.as_array())
            .map(|arr| arr.iter().any(|c| c.as_str().is_some_and(|s| s.starts_with('@'))))
            .unwrap_or(false);
        if legacy_context_prefix {
            tracing::info!("task {id}: migrating legacy '@'-prefixed contexts to '#'");
        }

        legacy_status
            || legacy_time_estimate
            || legacy_date
            || legacy_context_field
            || legacy_context_prefix
    }

    async fn write_task_file(&self, id: &Uuid, task: &Task) -> TaskResult<()> {
        let path = Self::filename(&self.dir, id);
        let write_file = || -> TaskResult<()> {
            let temp_path = path.with_extension("json.tmp");
            let file = File::create(&temp_path).map_err(|e| TaskError::CreateFile {
                id: *id,
                path: temp_path.clone(),
                error: e,
            })?;
            serde_json::to_writer(file, task).map_err(|e| TaskError::WriteJsonFile {
                id: *id,
                path: temp_path.clone(),
                error: e,
            })?;
            fs::rename(&temp_path, &path).map_err(|e| TaskError::RenameFile {
                id: *id,
                path_from: temp_path,
                path_to: path.clone(),
                error: e,
            })?;
            Ok(())
        };

        write_file().map_err(|e| TaskError::WriteFile {
            id: *id,
            path,
            error: Box::new(e),
        })?;

        cfg_if::cfg_if! {
            if #[cfg(feature = "ssr-test-storagefail")] {
                Self::return_error(id)
            } else if #[cfg(feature = "ssr-test-rand")] {
                Self::should_return_error(id)
            } else {
                Ok(())
            }
        }
    }

    #[cfg(feature = "ssr-test-storagefail")]
    #[allow(dead_code)]
    fn return_error(id: &Uuid) -> TaskResult<()> {
        Err(TaskError::Test(*id))
    }

    #[cfg(feature = "ssr-test-rand")]
    #[allow(dead_code)]
    fn should_return_error(id: &Uuid) -> TaskResult<()> {
        use rand::Rng;
        let mut rng = rand::rng();
        if rng.random::<bool>() {
            Ok(())
        } else {
            Err(TaskError::Test(*id))
        }
    }

    async fn delete_task_file(&self, id: &Uuid) -> TaskResult<()> {
        let file = Self::filename(&self.dir, id);
        fs::remove_file(&file).map_err(|e| TaskError::RemoveFile {
            id: *id,
            path: file,
            error: e,
        })?;
        Ok(())
    }

    fn filename<P: AsRef<Path>>(dir: P, id: &Uuid) -> PathBuf {
        let id = id.as_simple();
        dir.as_ref().join(format!("task-{id}.json"))
    }
}

impl Default for TaskCache {
    fn default() -> Self {
        Self::with_capacity(10)
    }
}

impl Deref for TaskCache {
    type Target = DataMap;
    fn deref(&self) -> &Self::Target {
        &self.tasks
    }
}

impl<'a> Deref for TaskMutGuard<'a> {
    type Target = Task;
    fn deref(&self) -> &Self::Target {
        self.task
    }
}

impl<'a> DerefMut for TaskMutGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.task
    }
}

impl<'a> Drop for TaskMutGuard<'a> {
    fn drop(&mut self) {
        self.task.add_author(&self.actor);
        self.dirty.insert(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_fs::TempDir;
    use miette::{IntoDiagnostic, MietteHandlerOpts, Result};
    use std::path::PathBuf;
    use tracing_test::traced_test;
    use uuid::uuid;

    #[test]
    fn filenames() {
        // empty dir
        assert_eq!(
            TaskCache::filename(
                &PathBuf::new(),
                &uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04")
            ),
            PathBuf::from("task-019c500fe59875a19bd9286e3d82cd04.json")
        );
        // absolute dir
        assert_eq!(
            TaskCache::filename(
                &PathBuf::from("/tasks"),
                &uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04")
            ),
            PathBuf::from("/tasks/task-019c500fe59875a19bd9286e3d82cd04.json")
        );
        // relative dir
        assert_eq!(
            TaskCache::filename(
                &PathBuf::from("tasks"),
                &uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04")
            ),
            PathBuf::from("tasks/task-019c500fe59875a19bd9286e3d82cd04.json")
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn flush() -> Result<()> {
        init_miette_report();
        let dir = TempDir::with_prefix("kid-").into_diagnostic()?;
        tracing::debug!("dir: {}", dir.path().display());
        assert_files(&dir, 0)?;
        let mut cache = TaskCache::default().with_dir(dir.path());
        assert_files(&dir, 0)?;
        cache.flush().await?;
        assert_files(&dir, cache.len())?;
        dir.close().into_diagnostic()?;
        Ok(())
    }

    // DETECT LEGACY FORMAT

    const DETECT_ID: Uuid = uuid!("019c500f-e598-75a1-9bd9-286e3d82cd04");

    #[test]
    fn detect_legacy_format_no_time_estimate() {
        let json = r#"{"summary":"A","status":{"ToDo":{"since":"2026-01-01T00:00:00Z"}}}"#;
        assert!(!TaskCache::detect_legacy_format(&DETECT_ID, json));
    }

    #[test]
    fn detect_legacy_format_status_string() {
        let json = r#"{"summary":"A","status":"ToDo"}"#;
        assert!(TaskCache::detect_legacy_format(&DETECT_ID, json));
    }

    #[test]
    fn detect_legacy_format_time_estimate_guess() {
        let json = r#"{"summary":"A","status":{"ToDo":{"since":"2026-01-01T00:00:00Z"}},"time_estimate":{"Guess":"a weekend"}}"#;
        assert!(TaskCache::detect_legacy_format(&DETECT_ID, json));
    }

    #[test]
    fn detect_legacy_format_time_estimate_precise() {
        let json = r#"{"summary":"A","status":{"ToDo":{"since":"2026-01-01T00:00:00Z"}},"time_estimate":{"Precise":{"secs":3600,"nanos":0}}}"#;
        assert!(TaskCache::detect_legacy_format(&DETECT_ID, json));
    }

    #[test]
    fn detect_legacy_format_time_estimate_variant() {
        let json = r#"{"summary":"A","status":{"ToDo":{"since":"2026-01-01T00:00:00Z"}},"time_estimate":"Hours1"}"#;
        assert!(!TaskCache::detect_legacy_format(&DETECT_ID, json));
    }

    #[test]
    fn detect_legacy_format_context_field() {
        let json = r#"{"summary":"A","status":{"ToDo":{"since":"2026-01-01T00:00:00Z"}},"context":"Children"}"#;
        assert!(TaskCache::detect_legacy_format(&DETECT_ID, json));
    }

    #[test]
    fn detect_legacy_format_category_field() {
        let json = r#"{"summary":"A","status":{"ToDo":{"since":"2026-01-01T00:00:00Z"}},"category":"Children"}"#;
        assert!(!TaskCache::detect_legacy_format(&DETECT_ID, json));
    }

    #[test]
    fn detect_legacy_format_context_prefix() {
        let json = r#"{"summary":"A","status":{"ToDo":{"since":"2026-01-01T00:00:00Z"}},"contexts":["@home"]}"#;
        assert!(TaskCache::detect_legacy_format(&DETECT_ID, json));
    }

    #[test]
    fn detect_legacy_format_context_prefix_migrated() {
        let json = r##"{"summary":"A","status":{"ToDo":{"since":"2026-01-01T00:00:00Z"}},"contexts":["#home"]}"##;
        assert!(!TaskCache::detect_legacy_format(&DETECT_ID, json));
    }

    fn assert_files(dir: &Path, num: usize) -> Result<()> {
        let entries = fs::read_dir(dir).into_diagnostic()?;
        assert_eq!(entries.count(), num);
        Ok(())
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
}
