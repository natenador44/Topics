use std::{
    fs::OpenOptions,
    path::{Path, PathBuf},
    sync::{
        Arc, LazyLock,
        mpsc::{Receiver, Sender},
    },
};

use error_stack::{Result, ResultExt};
use serde::{Deserialize, Serialize};
use tokio::{
    runtime::{Handle, Runtime},
    sync::RwLock,
};
use tracing::{debug, error, info, instrument, span, Level, Metadata, Span};
use tracing::field::{debug, display, ValueSet};
use crate::app::models::TopicId;
use crate::app::repository::TopicRepoError;
use crate::app::{
    models::Topic,
    repository::{IdentifierRepository, Repository, SetRepository, TopicFilter, TopicRepository},
};

static APP_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let app_dir = PathBuf::from(
        std::env::var("TOPICS_APP_DIR").expect("TOPICS_APP_DIR environment variable must be set"),
    );

    info!("TOPICS_APP_DIR: {}", app_dir.display());
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).expect("TOPICS_APP_DIR could not be created");
    }
    app_dir
});

static TOPICS_FILE: LazyLock<PathBuf> = LazyLock::new(|| APP_DIR.join("topics.json"));

#[derive(Debug)]
enum TopicUpdateType {
    Create, Update, Delete
}
#[derive(Debug)]
struct TopicUpdate {
    id: TopicId,
    ty: TopicUpdateType,
}

#[derive(Debug, Clone)]
pub struct FileRepo {
    topics: Arc<RwLock<Vec<Topic>>>,
    /// Used to send notifications to the update thread, which gets a read lock of `topics` and
    /// saves them to `TOPICS_FILE`
    topic_updates: Sender<TopicUpdate>,
}

impl FileRepo {
    // consider making load/save data async. this would require loading the entire file into memory first.
    pub fn init(runtime: Handle) -> Result<Self, LoadError> {
        let (tx, rc) = std::sync::mpsc::channel();
        let topics = Arc::new(RwLock::new(load_data(&TOPICS_FILE)?));
        let tc = Arc::clone(&topics);

        runtime.spawn_blocking(move || handle_topic_updates(tc, rc));

        Ok(Self {
            topics,
            topic_updates: tx,
        })
    }
}

#[instrument(skip_all)]
fn handle_topic_updates(topics: Arc<RwLock<Vec<Topic>>>, rc: Receiver<TopicUpdate>) {
    loop {
        match rc.recv() {
            Ok(update) => {
                let span = span!(Level::INFO, "topic_update", id = display(update.id), ty = debug(update.ty));
                let _guard = span.enter();
                info!("topic update received");
                let topics = topics.blocking_read();
                if let Err(e) = save_data(&TOPICS_FILE, &*topics) {
                    error!("failed to apply topic updates: {e:?}");
                }
            }
            Err(e) => {
                error!("recv error: {e}.");
                error!("topic update thread exiting");
            }
        }
    }
}

impl Repository for FileRepo {
    type TopicRepo = FileTopicRepo;

    type IdentifierRepo = FileIdentifierRepo;

    type SetRepo = FileSetRepo;

    fn topics(&self) -> Self::TopicRepo {
        FileTopicRepo {
            topics: Arc::clone(&self.topics),
            topic_updates: self.topic_updates.clone(),
        }
    }

    fn identifiers(&self) -> Self::IdentifierRepo {
        FileIdentifierRepo
    }

    fn sets(&self) -> Self::SetRepo {
        FileSetRepo
    }
}

pub struct FileTopicRepo {
    topics: Arc<RwLock<Vec<Topic>>>,
    topic_updates: Sender<TopicUpdate>,
}

impl FileTopicRepo {
    #[instrument(skip_all)]
    fn send_update(&self, topic_id: TopicId, update_type: TopicUpdateType) {
        if let Err(e) = self.topic_updates.send(TopicUpdate { id: topic_id, ty: update_type }) {
            error!("failed to send topic update: {e:?}");
        }
    }
}

impl TopicRepository for FileTopicRepo {
    #[instrument(skip_all, ret(level = "debug"), name = "repo#search")]
    async fn search(
        &self,
        page: usize,
        page_size: usize,
        filters: Vec<TopicFilter>,
    ) -> Result<Vec<Topic>, TopicRepoError> {
        let topics = self.topics.read().await;

        let page = paginate_list(&topics, page, page_size);

        let Some(page) = page else {
            return Ok(vec![]);
        };

        let filtered = page
            .into_iter()
            .filter(|t| filter_topic(t, &filters))
            .cloned()
            .collect();

        Ok(filtered)
    }

    #[instrument(skip_all, ret(level = "debug"), name = "repo#get_by_id")]
    async fn get(&self, topic_id: TopicId) -> Result<Option<Topic>, TopicRepoError> {
        let topics = self.topics.read().await;
        Ok(topics.iter().find(|t| t.id == topic_id).cloned())
    }

    #[instrument(skip_all, ret(level = "debug"), name = "repo#create")]
    async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<TopicId, TopicRepoError> {
        let new_id = TopicId::now_v7();
        let new_topic = Topic::new(new_id, name, description);
        let mut topics = self.topics.write().await;
        topics.push(new_topic);
        self.send_update(new_id, TopicUpdateType::Create);

        Ok(new_id)
    }

    #[instrument(skip_all, ret(level = "debug"), name = "repo#delete")]
    async fn delete(&self, topic_id: TopicId) -> Result<(), TopicRepoError> {
        let mut topics = self.topics.write().await;
        topics.retain(|t| t.id != topic_id);
        self.send_update(topic_id, TopicUpdateType::Delete);
        Ok(())
    }
}

fn filter_topic(topic: &Topic, filters: &[TopicFilter]) -> bool {
    if filters.is_empty() {
        return true;
    }

    filters.iter().any(|f| match f {
        TopicFilter::Name(n) => topic.name.contains(n),
        TopicFilter::Description(d) => topic
            .description
            .as_ref()
            .map_or(false, |desc| desc.contains(d)),
    })
}

#[instrument(skip_all, fields(list_type = std::any::type_name::<T>()))]
fn paginate_list<T>(list: &[T], page: usize, page_size: usize) -> Option<&[T]> {
    let start = page.saturating_sub(1) * page_size;
    let end = (start + page_size).min(list.len());

    if start < list.len() {
        debug!("page found. start: {start}, end: {end}");
        Some(&list[start..end])
    } else {
        debug!("page not found");
        None
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to open file for reading")]
    Io,
    #[error("failed to parse file contents as JSON")]
    Json,
    #[error("failed to initialize file")]
    FileInit,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("failed to open file for writing")]
    Io,
    #[error("failed to parse file contents to JSON")]
    Json,
}

#[instrument(fields(data_type = std::any::type_name::<T>()))]
fn load_data<T>(path: &'static Path) -> Result<T, LoadError>
where
    for<'a> T: Deserialize<'a> + Serialize + Default,
{
    debug!("loading data...");

    let data = if path.exists() {
        debug!("file already exists, reading and parsing");
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .change_context(LoadError::Io)
            .attach_printable_lazy(|| path.display())?;

        serde_json::from_reader(file)
            .change_context(LoadError::Json)
            .attach_printable_lazy(|| path.display())?
    } else {
        debug!("file does not exist, creating and populating with default");
        let data = T::default();
        save_data(path, &data).change_context(LoadError::FileInit)?;
        data
    };

    debug!("loading data complete");

    Ok(data)
}

#[instrument(skip(data), fields(data_type = std::any::type_name::<T>()))]
fn save_data<T>(path: &'static Path, data: &T) -> Result<(), SaveError>
where
    T: Serialize,
{
    debug!("writing data...");
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .change_context(SaveError::Io)
        .attach_printable_lazy(|| path.display())?;

    serde_json::to_writer(file, data).change_context(SaveError::Json)?;
    debug!("writing data complete");

    Ok(())
}

pub struct FileIdentifierRepo;
impl IdentifierRepository for FileIdentifierRepo {}
pub struct FileSetRepo;
impl SetRepository for FileSetRepo {}
