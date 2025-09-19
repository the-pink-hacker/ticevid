#![feature(exact_size_is_empty)]

use std::{
    collections::HashMap,
    io::Cursor,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;
use image::{DynamicImage, ImageFormat, ImageReader};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::encode::{FrameEncoder, QoiEncoder};

pub mod encode;
pub mod frame;

pub const LCD_WIDTH: u16 = 320;
pub const LCD_HEIGHT: u16 = 240;
pub const BLOCK_SIZE: u16 = 512;
pub const BLOCKS_PER_HEADER: u8 = 2;
pub const HEADER_SIZE: u16 = BLOCK_SIZE * BLOCKS_PER_HEADER as u16;

pub const FRAME_FORMAT: ImageFormat = ImageFormat::Qoi;
pub const FRAME_FORMAT_EXTENSION: &str = "qoi";

pub const MAX_FILES_OPEN: u64 = 1024 * 1024;

#[derive(Debug, Deserialize)]
pub struct VideoDefinition {
    pub video: VideoMetadata,
    #[serde(default)]
    pub captions: HashMap<String, CaptionSource>,
}

#[derive(Debug, Deserialize)]
pub struct VideoMetadata {
    pub title: String,
    pub source: PathBuf,
    #[serde(default)]
    pub start: Option<VideoDuration>,
    #[serde(default)]
    pub durration: Option<VideoDuration>,
    pub fps: u8,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct VideoDuration {
    pub milliseconds: u32,
    pub seconds: u64,
    pub minutes: u64,
    pub hours: u64,
}

impl From<VideoDuration> for rust_ffmpeg::Duration {
    fn from(value: VideoDuration) -> Self {
        std::time::Duration::new(
            value.seconds + (value.minutes * 60) + (value.hours * 60 * 60),
            value.milliseconds * 1_000,
        )
        .into()
    }
}

impl VideoDefinition {
    async fn load(video: &Path) -> anyhow::Result<Self> {
        tokio::fs::read_to_string(video)
            .await
            .with_context(|| format!("Failed to load video definition: {video:?}"))
            .and_then(|x| {
                toml::from_str(&x)
                    .with_context(|| format!("Failed to parse video definition: {video:?}"))
            })
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CaptionSource {
    /// From a srt file
    External {
        /// The srt file
        source: PathBuf,
    },
    /// From the video file
    Internal {
        /// The name of the caption track
        name: String,
    },
}

#[derive(Debug, Parser)]
pub struct Args {
    video: PathBuf,
    out: PathBuf,
}

fn compress_color_space(rgb: [u8; 3]) -> u8 {
    let (red, green, blue) = (rgb[0], rgb[1], rgb[2]);
    let red = (red / 32) << 5;
    let green = green / 32;
    let blue = (blue / 64) << 3;
    red | green | blue
}

async fn open_frame(path: PathBuf) -> anyhow::Result<DynamicImage> {
    let buffer = tokio::fs::read(path).await?;
    Ok(ImageReader::with_format(Cursor::new(buffer), FRAME_FORMAT).decode()?)
}

async fn encode_frame(frame: DynamicImage) -> anyhow::Result<Vec<u8>> {
    Ok(frame
        .as_rgb8()
        .context("Image wasn't 8-bit color.")?
        .pixels()
        .map(|pixel| compress_color_space(pixel.0))
        .collect::<Vec<_>>())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::try_parse()?;

    if let Err(error) = rlimit::increase_nofile_limit(MAX_FILES_OPEN) {
        log::warn!("Failed to increase max files open to {MAX_FILES_OPEN}:\n{error}");
    } else {
        log::debug!("Increased max files open to {MAX_FILES_OPEN}.");
    }

    let definition_folder = args
        .video
        .parent()
        .with_context(|| format!("Failed to get definition folder: {:?}", args.video))?;

    let output_folder = args
        .out
        .parent()
        .context("Failed to get out file's parent directory")?;

    let definition = VideoDefinition::load(&args.video).await?;

    let frames_folder = definition.frames_folder(output_folder)?;

    let frame_count = definition
        .create_frames(definition_folder, &frames_folder)
        .await?;

    let frame_count_digits = (frame_count.checked_ilog10().unwrap_or_default() + 1) as usize;

    let mut set = tokio::task::JoinSet::<anyhow::Result<_>>::new();

    let encoding_start = tokio::time::Instant::now();

    for frame_index in 1..=frame_count {
        let frames_folder = frames_folder.clone();
        set.spawn(async move {
            let frame_name = frames_folder.join(format!("{frame_index}.{FRAME_FORMAT_EXTENSION}"));
            let frame = open_frame(frame_name).await.map(encode_frame)?.await?;

            let mut output_buffer = [0; LCD_WIDTH as usize * LCD_HEIGHT as usize];
            let compressed_bytes = QoiEncoder::default().encode(&frame, &mut output_buffer)?;

            //let mut output_buffer = [0; LCD_WIDTH as usize * LCD_HEIGHT as usize];
            //let compressed_bytes = LzssEncoder.encode(&qoi_buffer[..compressed_bytes], &mut output_buffer)?;

            log::debug!(
                "Compressed frame {frame_index:>frame_count_digits$}: {} bytes => {} bytes, {:>5.2}%",
                frame.len(),
                compressed_bytes,
                (compressed_bytes as f32 / frame.len() as f32) * 100.0,
            );

            tokio::fs::File::create(frames_folder.join(format!("{frame_index}.video.bin")))
                .await?
                .write_all(&output_buffer[..compressed_bytes])
                .await?;

            Ok(compressed_bytes)
        });
    }

    let mut sum = 0.0;
    let mut frames = 0usize;

    while let Some(join) = set.join_next().await {
        sum += join?? as f32;
        frames += 1;
        log::info!(
            "Encoding frames: {:>frame_count_digits$}/{} {:>5.2}%",
            frames,
            frame_count,
            (frames as f32 / frame_count as f32) * 100.0
        );
    }

    let time = encoding_start.elapsed().as_secs_f32() * 1_000.0;
    log::info!("Encoding took {:.2} MS.", time);
    log::info!("Average size {:.0} bytes.", sum / frames as f32);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qoi_run_intial() {
        let frame = vec![0, 0, 0, 0];
        let mut output = vec![0];
        let bytes = QoiEncoder::default().encode(&frame, &mut output).unwrap();

        let expected = vec![0b1100_0011];
        assert_eq!(bytes, expected.len());
        assert_eq!(output, expected);
    }

    #[test]
    fn qoi_run_overflow() {
        let frame = vec![0; 64];
        let mut output = vec![0; 2];
        let bytes = QoiEncoder::default().encode(&frame, &mut output).unwrap();

        let expected = vec![0b1111_1110, 0b1100_0000];
        assert_eq!(bytes, expected.len());
        assert_eq!(output, expected);
    }
}
