use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TitleDefinition {
    /// The name of the title that is displayed to the user.
    #[serde(default)]
    pub name: String,
    /// A path to a 16x16 icon.
    #[serde(default)]
    pub icon: Option<PathBuf>,
    /// The path to the title's video file.
    pub video: PathBuf,
    /// If set, encodes from the durration.
    #[serde(default)]
    pub start: Option<TitleDuration>,
    /// If set, encodes the durration's worth of frames.
    #[serde(default)]
    pub durration: Option<TitleDuration>,
    /// The desired frame rate of the playback.
    pub fps: u8,
    /// Which captions are available for the title.
    #[serde(default)]
    pub captions: HashMap<String, CaptionSource>,
    // TODO: Make optional.
    /// The height of the video.
    pub height: u8,
}

#[derive(Debug, Deserialize)]
pub struct TitleMetadata {
    pub title: String,
    pub source: PathBuf,
    #[serde(default)]
    pub start: Option<TitleDuration>,
    #[serde(default)]
    pub durration: Option<TitleDuration>,
    pub fps: u8,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct TitleDuration {
    pub milliseconds: u32,
    pub seconds: u64,
    pub minutes: u64,
    pub hours: u64,
}

impl From<TitleDuration> for rust_ffmpeg::Duration {
    fn from(value: TitleDuration) -> Self {
        std::time::Duration::new(
            value.seconds + (value.minutes * 60) + (value.hours * 60 * 60),
            value.milliseconds * 1_000,
        )
        .into()
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CaptionSource {
    /// From a subtitle file
    External {
        /// An ass or srt file
        source: PathBuf,
    },
    /// From the video file
    Internal {
        /// The index of the caption track
        index: u8,
    },
}
