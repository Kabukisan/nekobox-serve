use crate::database::set_task_response;
use crate::error::Error;
use crate::models::{AudioFormat, MediaType, TaskResponse, TaskStatus};
use crate::queue::QueueJob;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod wrapper;
pub(crate) mod ytdl;

type CollectResult = Result<FetchCollection, Error>;
type DownloadResult = Result<DownloadResponse, Error>;

pub trait FetchService {
    fn prepare(&mut self, task: &TaskResponse, url: &str) -> Result<(), Error>;
    fn collect(&mut self) -> CollectResult;
    fn download(
        &mut self,
        media_type: MediaType,
        audio_format: Option<AudioFormat>,
    ) -> DownloadResult;
}

pub struct FetchServiceHandler<T>
where
    T: FetchService + ?Sized,
{
    url: String,
    service: Box<T>,
    task: TaskResponse,
    media_type: MediaType,
    audio_format: Option<AudioFormat>,
}

impl<T> FetchServiceHandler<T>
where
    T: FetchService,
{
    pub fn new(
        url: &str,
        service: T,
        task: TaskResponse,
        media_type: MediaType,
        audio_format: Option<AudioFormat>,
    ) -> Self {
        FetchServiceHandler {
            url: url.to_string(),
            service: Box::new(service),
            task,
            media_type,
            audio_format,
        }
    }

    pub fn prepare(&mut self, task: &TaskResponse, url: &str) -> Result<(), Error> {
        self.service.prepare(task, url)
    }

    pub fn collect(&mut self) -> Result<FetchCollection, Error> {
        self.service.collect()
    }

    fn download(
        &mut self,
        media_type: MediaType,
        audio_format: Option<AudioFormat>,
    ) -> DownloadResult {
        self.service.download(media_type, audio_format)
    }
}

impl<T> QueueJob for FetchServiceHandler<T>
where
    T: FetchService + Send + 'static,
{
    fn run(&mut self) {
        set_task_response(&self.task).expect("Failed to change task status");
        match self.service.download(self.media_type, self.audio_format) {
            Ok(_) => {
                self.task.status = TaskStatus::Complete;
                self.task.percentage = 1.0;
            }
            Err(_) => {
                self.task.status = TaskStatus::Error;
            }
        };
        set_task_response(&self.task).expect("Failed to change task status");
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadResponse {
    pub status: ResponseStatus,
    pub path: Option<PathBuf>,
}

impl DownloadResponse {
    pub fn new(status: ResponseStatus, path: Option<PathBuf>) -> Self {
        DownloadResponse { status, path }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ResponseStatus {
    Complete,
    Pending,
    Failed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchCollection {
    pub title: Option<String>,
    pub description: Option<String>,
    pub duration: Option<usize>,
    pub channel: Option<String>,
    pub views: Option<usize>,
    pub uploader: Option<String>,
}

impl FetchCollection {
    pub fn builder() -> Self {
        FetchCollection {
            title: None,
            description: None,
            duration: None,
            channel: None,
            views: None,
            uploader: None,
        }
    }

    pub fn title<S: Into<String>>(&mut self, title: S) -> &mut Self {
        self.title = Some(title.into());
        self
    }

    pub fn description<S: Into<String>>(&mut self, description: S) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    pub fn channel<S: Into<String>>(&mut self, channel: S) -> &mut Self {
        self.channel = Some(channel.into());
        self
    }

    pub fn duration(&mut self, duration: usize) -> &mut Self {
        self.duration = Some(duration);
        self
    }

    pub fn views(&mut self, views: usize) -> &mut Self {
        self.views = Some(views);
        self
    }

    pub fn uploader<S: Into<String>>(&mut self, uploader: S) -> &mut Self {
        self.uploader = Some(uploader.into());
        self
    }
}
