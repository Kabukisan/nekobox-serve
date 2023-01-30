use crate::environment::provide_cache_subdir;
use crate::error::Error;
use crate::models::{AudioFormat, MediaType, TaskResponse};
use crate::service::wrapper::YoutubeDlWrapper;
use crate::service::{
    DownloadResponse, DownloadResult, FetchCollection, FetchService, ResponseStatus,
};
use serde_json::Value::Null;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};

pub struct YoutubeDl {
    wrapper: Option<YoutubeDlWrapper>,
    working_directory: Option<PathBuf>,
    task: Option<TaskResponse>,
    url: Option<String>,
}

impl YoutubeDl {
    pub fn service() -> Self {
        YoutubeDl {
            wrapper: None,
            working_directory: None,
            task: None,
            url: None,
        }
    }

    fn prepare_command(&mut self) -> Result<(YoutubeDlWrapper, PathBuf), Error> {
        let working_directory = provide_cache_subdir(&self.task.as_ref().unwrap().status_id)
            .ok_or(Error::InternalError)?;
        let mut wrapper = YoutubeDlWrapper::new(Path::new("youtube-dl"));
        wrapper.current_dir(&working_directory);
        wrapper.arg(self.url.as_ref().unwrap());

        Ok((wrapper, working_directory))
    }

    fn handle_stdout(&mut self, child: &mut Child) {
        todo!()
    }
}

impl FetchService for Box<YoutubeDl> {
    fn prepare(&mut self, task: &TaskResponse, url: &str) -> Result<(), Error> {
        self.task = Some(task.clone());
        self.url = Some(url.to_string());

        let (wrapper, working_directory) = self.prepare_command()?;

        self.wrapper = Some(wrapper);
        self.working_directory = Some(working_directory);
        Ok(())
    }
    fn collect(&mut self) -> Result<FetchCollection, Error> {
        let wrapper = self.wrapper.as_mut().unwrap();

        let output = wrapper
            .write_info_json()
            .write_thumbnail()
            .skip_download()
            .output("0.%(ext)s")
            .execute_command();

        if let Err(_) = output {
            return Err(Error::InternalError);
        }

        let working_directory = self.working_directory.as_mut().unwrap();

        let json_file = fs::read_to_string(&working_directory.join("0.info.json"))?;
        let json: serde_json::Value = serde_json::from_str(&json_file)?;

        let mut collection = FetchCollection::builder();

        if json["title"] != Null {
            collection.title(json["title"].to_string());
        }

        if json["description"] != Null {
            collection.description(json["description"].to_string());
        }

        if json["duration"] != Null {
            let duration = json["duration"].to_string().parse::<usize>();

            if let Ok(value) = duration {
                collection.duration(value);
            }
        }

        if json["channel"] != Null {
            collection.channel(json["channel"].to_string());
        }

        if json["view_count"] != Null {
            let view_count = json["view_count"].to_string().parse::<usize>();

            if let Ok(value) = view_count {
                collection.views(value);
            }
        }

        if json["uploader"] != Null {
            collection.uploader(json["uploader"].to_string());
        }

        Ok(collection)
    }

    fn download(
        &mut self,
        media_type: MediaType,
        audio_format: Option<AudioFormat>,
    ) -> DownloadResult {
        let (mut wrapper, _) = self.prepare_command()?;

        if media_type == MediaType::Audio {
            wrapper.extract_audio();
        }

        if let Some(audio_format) = audio_format {
            wrapper.audio_format(&audio_format.to_string());
        }

        let mut output = wrapper
            .no_playlist()
            .newline()
            .stdout(Stdio::piped())
            .spawn_command()?;

        // Progress information is taken from the command data stream
        // and transmitted to the websocket clients
        // self.handle_stdout(&mut output);

        let (response_status, file_path) = match output.wait() {
            Ok(_) => {
                let file_path = self.working_directory.clone();
                (ResponseStatus::Complete, file_path)
            }
            Err(_) => (ResponseStatus::Failed, None),
        };

        let response = DownloadResponse::new(response_status, file_path);
        Ok(response)
    }
}
