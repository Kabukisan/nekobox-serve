use crate::models::{Format, MediaType};
use std::fs::File;

pub trait Delivery {
    fn title(&mut self) -> String;
    fn download(&mut self, media_type: MediaType, format: Option<Format>) -> String;
}

pub trait Description {
    fn description(&mut self) -> Option<String>;
}

pub trait Thumbnail {
    fn thumbnail(&mut self) -> Option<File>;
}
