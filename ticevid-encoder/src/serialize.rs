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
    CHUNK_SIZE, FRAME_FORMAT, FRAME_FORMAT_EXTENSION, HEADER_SIZE, LCD_HEIGHT, LCD_WIDTH,
    PICTURE_IMAGE_SIZE, PICTURE_START_IMAGE_SIZE,
    definition::title::TitleDefinition,
    encode::{FrameEncoder, QoiEncoder},
};

pub const VERSION: (u16, u8, u8) = (0, 1, 0);

const U24_ONE: u24 = u24::checked_from_u32(1).unwrap();

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

fn picture_chunk_path(frame_index: u32, chunk_index: u8, frames_folder: &Path) -> PathBuf {
    frames_folder.join(format!("{frame_index}.picture.{chunk_index}.bin"))
}

pub async fn write_picture_chunk(
    frame_index: u32,
    chunk_index: u8,
    frames_folder: &Path,
    buffer: &[u8],
) -> anyhow::Result<()> {
    let path = picture_chunk_path(frame_index, chunk_index, frames_folder);
    debug!(
        "Writing picture chunk: Frame {frame_index}, Chunk {chunk_index}, Size {} bytes",
        buffer.len()
    );
    tokio::fs::File::create(&path)
        .await
        .with_context(|| format!("Failed to open chunk at: {}", path.display()))?
        .write_all(buffer)
        .await
        .with_context(|| format!("Failed to write chunk at: {}", path.display()))
}

pub async fn serialize_frame(
    frames_folder: &Path,
    frame_index: u32,
    frame_count_digits: usize,
) -> anyhow::Result<(usize, Vec<usize>)> {
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

    let mut output_iter = output_buffer[..compressed_bytes].iter().copied();

    let mut chunk_index = 0;

    let mut chunk_sizes = Vec::new();

    match output_iter.next_chunk::<PICTURE_START_IMAGE_SIZE>() {
        // One or more chunks big
        Ok(chunk) => {
            chunk_sizes.push(chunk.len());
            write_picture_chunk(frame_index, chunk_index, frames_folder, &chunk).await?;
            chunk_index += 1;

            let mut remaining_chunks = output_iter.array_chunks::<PICTURE_IMAGE_SIZE>();

            // Remaining chunks
            for chunk in remaining_chunks.by_ref() {
                chunk_sizes.push(chunk.len());
                write_picture_chunk(frame_index, chunk_index, frames_folder, &chunk).await?;
                chunk_index += 1;
            }

            let last_chunk = remaining_chunks.into_remainder();

            // Last chunk isn't exact
            if !last_chunk.is_empty() {
                chunk_sizes.push(last_chunk.len());
                write_picture_chunk(
                    frame_index,
                    chunk_index,
                    frames_folder,
                    &last_chunk.collect::<Vec<_>>(),
                )
                .await?;
            }
        }
        // Less than one chunk big
        Err(chunk) => {
            chunk_sizes.push(chunk.len());
            write_picture_chunk(frame_index, 0, frames_folder, &chunk.collect::<Vec<_>>()).await?;
        }
    }

    Ok((compressed_bytes, chunk_sizes))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PictureChunkId {
    title_index: u8,
    frame: u24,
    chunk_index: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SectorId {
    Header,
    TitleTable,
    Title { title_index: u8 },
    TitleName { title_index: u8 },
    HeaderEnd,
    Chunks,
    PictureChunk(PictureChunkId),
    PictureChunkEnd(PictureChunkId),
}

type SerialBuilder = serseg::prelude::SerialBuilder<SectorId>;
type SectorBuilder = serseg::prelude::SerialSectorBuilder<SectorId>;

fn try_into_u24(value: impl TryInto<u32>) -> Option<u24> {
    value.try_into().ok().and_then(u24::checked_from_u32)
}

pub async fn serialize_container(
    titles: Vec<(Vec<Vec<usize>>, PathBuf, TitleDefinition)>,
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
                .u8(title_count)
                .dynamic_u24(SectorId::Header, SectorId::TitleTable, 0)
                // Font pack
                .null_24()
                // UI font index
                .u8(0),
        )
        .sector(SectorId::TitleTable, title_table_builder);

    // Title header

    for (title_index, (frame_counts, _, title)) in (0..title_count).zip(&titles) {
        let frame_len = frame_counts.len();
        let frame_count = try_into_u24(frame_len)
            .with_context(|| format!("Frame count exceeded maximum; {frame_len} > {}", u24::MAX))?;

        let first_picture_chunk_id = PictureChunkId {
            title_index,
            frame: u24::default(),
            chunk_index: 0,
        };

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
                .u8(0)
                // Caption tracks
                .null_24()
                // Caption foreground
                .u8(0xFF)
                // Caption background
                .u8(0)
                // Caption transparent
                // TODO: Add bools to serseg
                .u8(1)
                // Chapter count
                .u8(0)
                // Chapter table
                .null_24()
                .dynamic_u16(
                    SectorId::PictureChunk(first_picture_chunk_id),
                    SectorId::PictureChunkEnd(first_picture_chunk_id),
                    0,
                )
                .dynamic_u24_chunk(
                    SectorId::Chunks,
                    SectorId::PictureChunk(first_picture_chunk_id),
                    0,
                    CHUNK_SIZE as usize,
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

    // Start picture chunks
    for (title_index, (frame_sizes, frames_directory, _)) in (0..title_count).zip(&titles) {
        let mut frame_sizes_iter = frame_sizes.iter().enumerate();

        while let Some((frame_index, chunk_sizes)) = frame_sizes_iter.next() {
            let frame = try_into_u24(frame_index).with_context(|| {
                format!("Frame index exceeded maximum; {frame_index} > {}", u24::MAX)
            })?;

            let first_picture_chunk_id = PictureChunkId {
                title_index,
                frame,
                chunk_index: 0,
            };

            let mut picture_sector_builder = SectorBuilder::default();

            picture_sector_builder = if chunk_sizes.len() > 1 {
                let chunk_count = chunk_sizes.len() as u8 - 1;

                let next_chunk_id = PictureChunkId {
                    title_index,
                    frame,
                    chunk_index: 1,
                };

                picture_sector_builder
                    .dynamic_u24_chunk(
                        SectorId::Chunks,
                        SectorId::PictureChunk(next_chunk_id),
                        0,
                        CHUNK_SIZE as usize,
                    )
                    .u8(chunk_count)
                    .dynamic_u16(
                        SectorId::PictureChunk(next_chunk_id),
                        SectorId::PictureChunkEnd(next_chunk_id),
                        0,
                    )
            } else {
                picture_sector_builder.null_24().u8(0).null_16()
            };

            picture_sector_builder = if frame_sizes_iter.is_empty() {
                picture_sector_builder.null_16()
            } else {
                let next_frame_id = PictureChunkId {
                    title_index,
                    frame: frame + U24_ONE,
                    chunk_index: 0,
                };

                picture_sector_builder.dynamic_u16(
                    SectorId::PictureChunk(next_frame_id),
                    SectorId::PictureChunkEnd(next_frame_id),
                    0,
                )
            };

            builder = builder
                .sector(
                    SectorId::PictureChunk(first_picture_chunk_id),
                    picture_sector_builder.external(
                        picture_chunk_path(frame_index as u32 + 1, 0, frames_directory),
                        chunk_sizes[0],
                    ),
                )
                .sector(
                    SectorId::PictureChunkEnd(first_picture_chunk_id),
                    SectorBuilder::default().fill(
                        SectorId::PictureChunk(first_picture_chunk_id),
                        CHUNK_SIZE as usize,
                    ),
                );
        }
    }

    // Picture chunks
    for (title_index, (frame_sizes, frames_directory, _)) in (0..title_count).zip(titles) {
        for (frame_index, chunk_sizes) in frame_sizes.iter().enumerate() {
            if chunk_sizes.len() <= 1 {
                continue;
            }

            let frame = try_into_u24(frame_index).with_context(|| {
                format!("Frame index exceeded maximum; {frame_index} > {}", u24::MAX)
            })?;

            let mut chunk_sizes_iter = chunk_sizes.iter().cloned().enumerate().skip(1);

            while let Some((chunk_index, chunk_size)) = chunk_sizes_iter.next() {
                let picture_id = PictureChunkId {
                    title_index,
                    frame,
                    chunk_index: chunk_index as u8,
                };

                let mut sector_builder = SectorBuilder::default();

                sector_builder = if chunk_sizes_iter.is_empty() {
                    sector_builder.null_16()
                } else {
                    let next_chunk_id = PictureChunkId {
                        title_index,
                        frame,
                        chunk_index: chunk_index as u8 + 1,
                    };

                    sector_builder.dynamic_u16(
                        SectorId::PictureChunk(next_chunk_id),
                        SectorId::PictureChunkEnd(next_chunk_id),
                        0,
                    )
                };

                builder = builder
                    .sector(
                        SectorId::PictureChunk(picture_id),
                        sector_builder.external(
                            picture_chunk_path(
                                frame_index as u32 + 1,
                                chunk_index as u8,
                                &frames_directory,
                            ),
                            chunk_size,
                        ),
                    )
                    .sector(
                        SectorId::PictureChunkEnd(picture_id),
                        SectorBuilder::default()
                            .fill(SectorId::PictureChunk(picture_id), CHUNK_SIZE as usize),
                    );
            }
        }
    }

    builder.build(&mut output_buffer).await
}
