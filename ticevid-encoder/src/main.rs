#![feature(exact_size_is_empty)]

use std::{
    collections::HashMap,
    io::Cursor,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use clap::Parser;
use image::{DynamicImage, ImageFormat, ImageReader};
use log::{debug, info, warn};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::encode::{FrameEncoder, QoiEncoder};

pub mod encode;
pub mod frame;

pub const LCD_WIDTH: u16 = 320;
pub const LCD_HEIGHT: u16 = 240;
pub const BLOCK_SIZE: u16 = 512;
pub const BLOCKS_PER_HEADER: u8 = 4;
pub const HEADER_SIZE: u16 = BLOCK_SIZE * BLOCKS_PER_HEADER as u16;
pub const BLOCKS_PER_CHUNK: u8 = 16;
pub const CHUNK_SIZE: u16 = BLOCK_SIZE * BLOCKS_PER_CHUNK as u16;
pub const CHUNK_HEADER_SIZE: u16 = 1;
pub const CHUNK_PAYLOAD_SIZE: u16 = CHUNK_SIZE - CHUNK_HEADER_SIZE;
pub const SCHEMA_VERSION: u8 = 0;

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
        /// The index of the caption track
        index: u8,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SectorId {
    Start,
    Header,
    HeaderVideoTable,
    HeaderVideoTableData(u8),
    HeaderVideoTableStrings(u8),
    HeaderCaptionTable,
    HeaderCaptionTableData(u8),
    HeaderCaptionTableStrings(u8),
    HeaderFontTable,
    HeaderFontTableData(u8),
    HeaderEnd,
    ChunksStart,
    ChunkFirst(u8),
    ChunkStart(u8, u16),
    ChunkData(u8, u16),
    ChunkEnd(u8, u16),
    ChunkLast(u8),
}

#[repr(u8)]
pub enum FrameType {
    VideoKey = 0,
    Caption = 1,
}

impl From<FrameType> for u8 {
    fn from(value: FrameType) -> Self {
        value as u8
    }
}

type SerialBuilder = serseg::prelude::SerialBuilder<SectorId>;
type SectorBuilder = serseg::prelude::SerialSectorBuilder<SectorId>;

async fn serialize_video(
    videos: &[(u32, PathBuf, VideoDefinition)],
    mut output_buffer: impl tokio::io::AsyncWrite + tokio::io::AsyncSeek + Unpin,
) -> anyhow::Result<()> {
    let mut builder = SerialBuilder::default()
        .sector_default(SectorId::Start)
        .sector(
            SectorId::Header,
            SectorBuilder::default()
                .u8(SCHEMA_VERSION)
                // Title
                .dynamic_u24(SectorId::Start, SectorId::HeaderVideoTableStrings(0), 0)
                // Video table length
                .u8(videos.len() as u8)
                .dynamic_u24(SectorId::Start, SectorId::HeaderVideoTable, 0)
                // Caption track length
                .u8(1)
                .u24(u24::u24::from_le_bytes([0, 0, 0]))
                // Caption font table
                .u8(1)
                .dynamic_u24(SectorId::Start, SectorId::HeaderFontTable, 0),
        )
        .sector(
            SectorId::HeaderFontTable,
            SectorBuilder::default().dynamic_u24(
                SectorId::Start,
                SectorId::HeaderFontTableData(0),
                0,
            ),
        )
        .sector(SectorId::HeaderFontTableData(0), SectorBuilder::default());

    // Create header video table
    let mut header_video_table_sector = SectorBuilder::default();

    for video_index in 0..videos.len() {
        header_video_table_sector = header_video_table_sector.dynamic_u24(
            SectorId::Start,
            SectorId::HeaderVideoTableData(video_index as u8),
            video_index,
        );
    }

    builder = builder.sector(SectorId::HeaderVideoTable, header_video_table_sector);

    // Create each video's metadata
    for ((frames, _, definition), video_index) in videos.iter().zip(0..=u8::MAX) {
        builder = builder
            .sector(
                SectorId::HeaderVideoTableData(video_index),
                SectorBuilder::default()
                    // Title
                    .dynamic_u24(
                        SectorId::Start,
                        SectorId::HeaderVideoTableStrings(video_index),
                        0,
                    )
                    // First chunk
                    .dynamic_u24_chunk(
                        SectorId::ChunksStart,
                        SectorId::ChunkStart(video_index, 0),
                        0,
                        CHUNK_SIZE as usize,
                    )
                    // Number of chunks
                    .dynamic_u24_chunk(
                        SectorId::ChunkFirst(video_index),
                        SectorId::ChunkLast(video_index),
                        0,
                        CHUNK_SIZE as usize,
                    )
                    // Icon
                    .u24(u24::u24::from_le_bytes([0, 0, 0]))
                    // Total frames
                    .u24(u24::u24::checked_from_u32(*frames).with_context(|| {
                        format!(
                            "There can only be u24::MAX frames: {} > {}",
                            frames,
                            u24::u24::MAX
                        )
                    })?)
                    // FPS
                    .u8(definition.video.fps)
                    // Video height
                    .u8(180),
            )
            .sector(
                SectorId::HeaderVideoTableStrings(video_index),
                SectorBuilder::default().string(definition.video.title.clone()),
            );
    }

    // End of header
    builder = builder
        .sector(
            SectorId::HeaderEnd,
            SectorBuilder::default().fill(SectorId::Start, HEADER_SIZE.into()),
        )
        .sector_default(SectorId::ChunksStart);

    // Chunks
    for ((frames, frames_path, _), video_index) in videos.iter().zip(0..=u8::MAX) {
        let mut chunk_index = 0;
        let mut frames_in_chunk = 0;
        // Subtract by one because of header size
        let mut chunk_size_left = CHUNK_SIZE as usize - 1;
        let mut chunk_table = SectorBuilder::default();
        let mut chunk_data = SectorBuilder::default();
        builder = builder
            .sector_default(SectorId::ChunkFirst(video_index))
            .sector_default(SectorId::ChunkStart(video_index, chunk_index));

        for frame in 0..*frames {
            let frame_path = frames_path.join(format!("{}.video.bin", frame + 1));
            let frame_size = tokio::fs::metadata(&frame_path)
                .await
                .with_context(|| format!("Frame file can't be found: {frame_path:?}"))?
                .len() as usize;

            // Subtract by 2 for smallest header size
            if frame_size > MAX_FRAME_SIZE as usize {
                bail!(
                    "Frame {frame} of video {video_index} is too big: {frame_size} bytes > {MAX_FRAME_SIZE} bytes"
                );
            }

            let checked_size_left = chunk_size_left.checked_sub(frame_size + 4);

            if let Some(size_left) = checked_size_left {
                chunk_size_left = size_left;

                chunk_table = chunk_table.u8(FrameType::VideoKey).dynamic_u24(
                    SectorId::ChunkStart(video_index, chunk_index),
                    SectorId::ChunkData(video_index, chunk_index),
                    frames_in_chunk as usize,
                );
                chunk_data = chunk_data.external(frame_path, frame_size);

                frames_in_chunk += 1;
            }

            let is_last_frame = frame + 1 == *frames;

            // If end of chunk
            if checked_size_left.is_none() || is_last_frame {
                // Chunk is full
                // Finish chunk
                builder = builder
                    .sector(
                        SectorId::ChunkHeader(video_index, chunk_index),
                        SectorBuilder::default().u8(frames_in_chunk),
                    )
                    .sector(SectorId::ChunkTable(video_index, chunk_index), chunk_table)
                    .sector(SectorId::ChunkData(video_index, chunk_index), chunk_data)
                    .sector(
                        SectorId::ChunkEnd(video_index, chunk_index),
                        SectorBuilder::default().fill(
                            SectorId::ChunkStart(video_index, chunk_index),
                            CHUNK_SIZE as usize,
                        ),
                    );

                if is_last_frame {
                    builder = builder.sector_default(SectorId::ChunkLast(video_index));
                    break;
                }

                // Advance to next chunk
                chunk_index += 1;
                frames_in_chunk = 0;
                chunk_size_left = CHUNK_SIZE as usize - 1;
                chunk_table = SectorBuilder::default();
                chunk_data = SectorBuilder::default();

                builder = builder.sector_default(SectorId::ChunkStart(video_index, chunk_index));
            }
        }
    }

    builder.build(&mut output_buffer).await
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::try_parse()?;

    if let Err(error) = rlimit::increase_nofile_limit(MAX_FILES_OPEN) {
        warn!("Failed to increase max files open to {MAX_FILES_OPEN}:\n{error}");
    } else {
        debug!("Increased max files open to {MAX_FILES_OPEN}.");
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

            debug!(
                "Compressed frame {frame_index:>frame_count_digits$}: {} bytes => {} bytes, {:>6.2}%",
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
    let mut frames = 0u32;

    while let Some(join) = set.join_next().await {
        sum += join?? as f32;
        frames += 1;

        if frames.is_multiple_of(24) || frames == frame_count {
            info!(
                "Encoding frames: {:>frame_count_digits$}/{} {:>6.2}%",
                frames,
                frame_count,
                (frames as f32 / frame_count as f32) * 100.0
            );
        }
    }

    let time = encoding_start.elapsed().as_secs_f32() * 1_000.0;
    info!("Encoding took {:.2} MS.", time);
    info!("Average size {:.0} bytes.", sum / frames as f32);

    let output_buffer = tokio::io::BufWriter::with_capacity(
        BLOCK_SIZE as usize * 64,
        tokio::fs::File::create(args.out).await?,
    );

    serialize_video(&[(frame_count, frames_folder, definition)], output_buffer).await?;

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
