use crate::error::Error;
use crate::models::{AudioFormat, MediaType, TaskResponse};
use crate::queue::QueueJob;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

mod wrapper;
pub(crate) mod ytdl;

type CollectResult = Result<FetchCollection, Error>;
type DownloadResult = Result<DownloadResponse, Error>;

pub trait FetchServiceEvents: Send {
    fn on_start(&mut self);
    fn on_end(&mut self);
    fn on_complete(&mut self);
    fn on_error(&mut self);
}

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
    media_type: MediaType,
    audio_format: Option<AudioFormat>,
    observers: Vec<Arc<Mutex<dyn FetchServiceEvents>>>,
}

impl<T> FetchServiceHandler<T>
where
    T: FetchService,
{
    pub fn new(
        url: &str,
        service: T,
        media_type: MediaType,
        audio_format: Option<AudioFormat>,
    ) -> Self {
        FetchServiceHandler {
            url: url.to_string(),
            service: Box::new(service),
            media_type,
            audio_format,
            observers: vec![],
        }
    }

    pub fn add_observer(&mut self, observer: Arc<Mutex<dyn FetchServiceEvents>>) {
        self.observers.push(observer);
    }

    fn fire_start_event(&mut self) {
        for wrapped_observer in self.observers.clone() {
            let mut observer = wrapped_observer.lock().unwrap();
            observer.on_start();
        }
    }

    fn fire_end_event(&mut self) {
        for wrapped_observer in self.observers.clone() {
            let mut observer = wrapped_observer.lock().unwrap();
            observer.on_end();
        }
    }

    fn fire_complete_event(&mut self) {
        for wrapped_observer in self.observers.clone() {
            let mut observer = wrapped_observer.lock().unwrap();
            observer.on_complete();
        }
    }

    fn fire_error_event(&mut self) {
        for wrapped_observer in self.observers.clone() {
            let mut observer = wrapped_observer.lock().unwrap();
            observer.on_error();
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
        self.fire_start_event();
        match self.service.download(self.media_type, self.audio_format) {
            Ok(_) => {
                self.fire_complete_event();
            }
            Err(_) => {
                self.fire_error_event();
            }
        };
        self.fire_end_event();
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
