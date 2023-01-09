use std::fs::File;
use crate::models::{MediaFormat, MediaType};

pub trait Delivery {
    fn title(&mut self) -> String;
    fn download<S: Into<String>>(&mut self, url: S, media_type: MediaType, format: Option<MediaFormat>) -> String;
}

pub trait Description {
    fn description(&mut self) -> Option<String>;
}

pub trait Thumbnail {
    fn thumbnail(&mut self) -> Option<File>;
}