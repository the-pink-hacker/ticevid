#![feature(exact_size_is_empty)]
#![feature(iter_next_chunk)]
#![feature(iter_array_chunks)]
// Allowing this for pedantic
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cast_precision_loss)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use clap::Parser;
use futures_util::{StreamExt, TryStreamExt, stream};
use image::ImageFormat;
use log::{debug, info, warn};
use u24::u24;

use crate::definition::{container::ContainerDefinition, title::TitleDefinition};

pub mod definition;
pub mod encode;
pub mod frame;
pub mod serialize;

pub const LCD_WIDTH: u16 = 320;
pub const LCD_HEIGHT: u16 = 240;
pub const BLOCK_SIZE: u16 = 512;
pub const BLOCKS_PER_HEADER: u8 = 16;
pub const HEADER_SIZE: u16 = BLOCK_SIZE * BLOCKS_PER_HEADER as u16;
pub const BLOCKS_PER_CHUNK: u8 = 16;
pub const CHUNK_SIZE: u16 = BLOCK_SIZE * BLOCKS_PER_CHUNK as u16;
pub const SCHEMA_VERSION: u24 = u24::checked_from_u32(0).unwrap();

pub const PICTURE_START_IMAGE_SIZE: usize = CHUNK_SIZE as usize - 8;
pub const PICTURE_IMAGE_SIZE: usize = CHUNK_SIZE as usize - 2;

pub const FRAME_FORMAT: ImageFormat = ImageFormat::Qoi;
pub const FRAME_FORMAT_EXTENSION: &str = "qoi";

pub const MAX_FILES_OPEN: u64 = 1024 * 1024;

#[derive(Debug, Parser)]
pub struct Args {
    /// A toml file defining a title container.
    container: PathBuf,
    /// The output file of the collection.
    out: PathBuf,
    /// The max amount of threads used for jobs. Defaults to the number of logical CPU cores.
    #[clap(short = 'j')]
    threads: Option<usize>,
}

fn get_container_directory(container: &Path) -> anyhow::Result<&Path> {
    container
        .parent()
        .with_context(|| format!("Failed to get container folder: {}", container.display()))
}

fn get_output_directory(output: &Path) -> anyhow::Result<&Path> {
    output.parent().with_context(|| {
        format!(
            "Failed to get output file's parent directory: {}",
            output.display()
        )
    })
}

async fn encode_title(
    title: TitleDefinition,
    title_directory: &Path,
    output_directory: &Path,
    threads: usize,
) -> anyhow::Result<(Vec<Vec<usize>>, PathBuf, TitleDefinition)> {
    let frames_folder = Arc::new(title.frames_folder(output_directory)?);

    let frame_count = title.create_frames(title_directory, &frames_folder).await?;
    let frame_count_digits = (frame_count.checked_ilog10().unwrap_or_default() + 1) as usize;

    let encoding_start = tokio::time::Instant::now();

    let mut frame_stream = stream::iter(1..=frame_count)
        .map(|frame_index| {
            let frames_folder = Arc::clone(&frames_folder);
            tokio::spawn(async move {
                Box::pin(serialize::serialize_frame(
                    &frames_folder,
                    frame_index,
                    frame_count_digits,
                ))
                .await
            })
        })
        .buffered(threads);

    let mut sum = 0.0;
    let mut frames = 0u32;

    let mut frame_chunk_sizes = Vec::with_capacity(frame_count as usize);

    while let Some(join) = frame_stream.next().await {
        let (bytes, chunk_sizes) = join??;
        sum += bytes as f32;
        frames += 1;

        frame_chunk_sizes.push(chunk_sizes);

        if frames.is_multiple_of(title.fps.into()) || frames == frame_count {
            info!(
                "Encoding frames: {:>frame_count_digits$}/{} {:>6.2}%",
                frames,
                frame_count,
                (frames as f32 / frame_count as f32) * 100.0
            );
        }
    }

    let time = encoding_start.elapsed().as_secs_f32() * 1_000.0;
    info!("Encoding took {time:.2} MS.");
    info!("Average size {:.0} bytes.", sum / frames as f32);

    Ok((frame_chunk_sizes, frames_folder.to_path_buf(), title))
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

    let threads = args.threads.unwrap_or_else(num_cpus::get);

    let container_directory = get_container_directory(&args.container)?;
    let output_directory = get_output_directory(&args.out)?;

    let container = ContainerDefinition::load(&args.container).await?;

    let encoded_titles = stream::iter(container.titles)
        .then(|title| encode_title(title, container_directory, output_directory, threads))
        .try_collect()
        .await?;

    let output_buffer = tokio::io::BufWriter::with_capacity(
        BLOCK_SIZE as usize * 64,
        tokio::fs::File::create(&args.out)
            .await
            .with_context(|| format!("Failed to open output: {}", args.out.display()))?,
    );

    serialize::serialize_container(encoded_titles, output_buffer).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::encode::{FrameEncoder, QoiEncoder};

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
