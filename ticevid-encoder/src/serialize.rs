use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use anyhow::Context;
use image::{DynamicImage, ImageReader};
use log::debug;
use tokio::io::AsyncWriteExt;
use u24::u24;

use crate::{
    BLOCK_SIZE, FRAME_FORMAT, FRAME_FORMAT_EXTENSION, HEADER_SIZE, LCD_HEIGHT, LCD_WIDTH,
    definition::title::TitleDefinition,
    encode::{FrameEncoder, QoiEncoder},
};

pub const VERSION: (u16, u8, u8) = (0, 1, 0);

async fn open_frame(path: PathBuf) -> anyhow::Result<DynamicImage> {
    let buffer = tokio::fs::read(path).await?;
    Ok(ImageReader::with_format(Cursor::new(buffer), FRAME_FORMAT).decode()?)
}

fn compress_color_space(rgb: [u8; 3]) -> u8 {
    let [red, green, blue] = rgb;
    let red = (red / 32) << 5;
    let green = green / 32;
    let blue = (blue / 64) << 3;
    red | green | blue
}

async fn encode_frame(frame: DynamicImage) -> anyhow::Result<Vec<u8>> {
    Ok(frame
        .as_rgb8()
        .context("Image wasn't 8-bit color.")?
        .pixels()
        .map(|pixel| compress_color_space(pixel.0))
        .collect())
}

fn picture_chunk_path(frame_index: usize, frames_folder: &Path) -> PathBuf {
    frames_folder.join(format!("{frame_index}.picture.bin"))
}

pub async fn serialize_frame(
    frames_folder: &Path,
    frame_index: u32,
    frame_count_digits: usize,
) -> anyhow::Result<usize> {
    let frame_name = frames_folder.join(format!("{frame_index}.{FRAME_FORMAT_EXTENSION}"));
    let frame = open_frame(frame_name).await.map(encode_frame)?.await?;

    let mut output_buffer = vec![0; LCD_WIDTH as usize * LCD_HEIGHT as usize];
    let compressed_bytes = QoiEncoder::default().encode(&frame, &mut output_buffer)?;

    debug!(
        "Compressed frame {frame_index:>frame_count_digits$}: {} bytes => {} bytes, {:>6.2}%",
        frame.len(),
        compressed_bytes,
        (compressed_bytes as f32 / frame.len() as f32) * 100.0,
    );

    let path = picture_chunk_path(frame_index as usize, frames_folder);
    tokio::fs::File::create(&path)
        .await
        .with_context(|| format!("Failed to open chunk at: {}", path.display()))?
        .write_all(&output_buffer[..compressed_bytes])
        .await
        .with_context(|| format!("Failed to write chunk at: {}", path.display()))?;

    Ok(compressed_bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PictureChunkId {
    title_index: u8,
    frame_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SectorId {
    Header,
    TitleTable,
    Title { title_index: u8 },
    TitleName { title_index: u8 },
    HeaderEnd,
    Chunks,
    PictureChunkTable { title_index: u8 },
    PictureChunk(PictureChunkId),
    PictureChunkImage(PictureChunkId),
    PictureChunkEnd(PictureChunkId),
    PictureChunkPadding(PictureChunkId),
}

type SerialBuilder = serseg::prelude::SerialBuilder<SectorId>;
type SectorBuilder = serseg::prelude::SerialSectorBuilder<SectorId>;

fn try_into_u24(value: impl TryInto<u32>) -> Option<u24> {
    value.try_into().ok().and_then(u24::checked_from_u32)
}

pub async fn serialize_container(
    titles: Vec<(Vec<usize>, PathBuf, TitleDefinition)>,
    mut output_buffer: impl tokio::io::AsyncWrite + tokio::io::AsyncSeek + Unpin,
) -> anyhow::Result<()> {
    let title_len = titles.len();
    let title_count = u8::try_from(title_len)
        .with_context(|| format!("Title count over maximum; {title_len} > {}", u8::MAX))?;

    // Title table

    let mut title_table_builder = SectorBuilder::default();

    for title_index in 0..title_count {
        title_table_builder =
            title_table_builder.dynamic_u24(SectorId::Header, SectorId::Title { title_index }, 0);
    }

    // Header

    let mut builder = SerialBuilder::default()
        .sector(
            SectorId::Header,
            SectorBuilder::default()
                .u16(VERSION.0)
                .u8(VERSION.1)
                .u8(VERSION.2)
                .dynamic_u16(SectorId::Header, SectorId::HeaderEnd, 0)
                .u8(title_count)
                .dynamic_u24(SectorId::Header, SectorId::TitleTable, 0)
                // Font pack
                .null_24()
                // UI font index
                .null_8(),
        )
        .sector(SectorId::TitleTable, title_table_builder);

    // Title header

    for (title_index, (frame_sizes, _, title)) in (0..title_count).zip(&titles) {
        let frame_count = frame_sizes.len();
        let frame_count = try_into_u24(frame_count).with_context(|| {
            format!("Frame count exceeded maximum; {frame_count} > {}", u24::MAX)
        })?;

        let mut title_builder = SectorBuilder::default();

        title_builder = if title.name.is_empty() {
            title_builder.null_24()
        } else {
            title_builder.dynamic_u24(SectorId::Header, SectorId::TitleName { title_index }, 0)
        };

        builder = builder.sector(
            SectorId::Title { title_index },
            title_builder
                // Color palette count
                .u8(0)
                // Color palette
                .null_24()
                // Icon
                .null_24()
                .u8(title.height)
                .u24(frame_count)
                .u8(title.fps)
                // Caption track count
                .null_8()
                // Caption tracks
                .null_24()
                // Caption foreground
                .u8(0xFF)
                // Caption background
                .null_8()
                // Caption transparent
                // TODO: Add bools to serseg
                .u8(1)
                // Chapter count
                .null_8()
                // Chapter table
                .null_24()
                .dynamic_u24_chunk(
                    SectorId::Chunks,
                    SectorId::PictureChunkTable { title_index },
                    0,
                    BLOCK_SIZE as usize,
                ),
        );

        if !title.name.is_empty() {
            builder = builder.sector(
                SectorId::TitleName { title_index },
                SectorBuilder::default().string(title.name.clone()),
            );
        }
    }

    // End of header

    builder = builder
        .sector(
            SectorId::HeaderEnd,
            SectorBuilder::default().fill(SectorId::Header, HEADER_SIZE as usize),
        )
        .sector_default(SectorId::Chunks);

    // Picture chunk tables
    for (title_index, (frame_count, _, _)) in (0..title_count).zip(&titles) {
        let mut picture_chunk_table_builder = SectorBuilder::default();

        for frame_index in 0..frame_count.len() {
            let chunk_id = PictureChunkId {
                title_index,
                frame_index,
            };

            picture_chunk_table_builder = picture_chunk_table_builder
                // Block count
                .dynamic_u16_chunk(
                    SectorId::PictureChunk(chunk_id),
                    SectorId::PictureChunkEnd(chunk_id),
                    0,
                    BLOCK_SIZE as usize,
                )
                // Block index
                .dynamic_u24_chunk(
                    SectorId::Header,
                    SectorId::PictureChunk(chunk_id),
                    0,
                    BLOCK_SIZE as usize,
                )
        }

        builder = builder.sector(
            SectorId::PictureChunkTable { title_index },
            picture_chunk_table_builder,
        );
    }

    // Picture chunks
    for (title_index, (frame_count, frames_directory, _)) in (0..title_count).zip(&titles) {
        for (frame_index, frame_size) in frame_count.iter().enumerate() {
            let chunk_id = PictureChunkId {
                title_index,
                frame_index,
            };

            let frame_path = picture_chunk_path(frame_index + 1, frames_directory);

            builder = builder
                .sector(
                    SectorId::PictureChunk(chunk_id),
                    SectorBuilder::default().dynamic_u16(
                        SectorId::PictureChunkImage(chunk_id),
                        SectorId::PictureChunkEnd(chunk_id),
                        0,
                    ),
                )
                .sector(
                    SectorId::PictureChunkImage(chunk_id),
                    SectorBuilder::default().external(frame_path, *frame_size),
                )
                .sector_default(SectorId::PictureChunkEnd(chunk_id))
                .sector(
                    SectorId::PictureChunkPadding(chunk_id),
                    SectorBuilder::default()
                        .fill(SectorId::PictureChunkEnd(chunk_id), BLOCK_SIZE as usize),
                );
        }
    }

    builder.build(&mut output_buffer).await
}
