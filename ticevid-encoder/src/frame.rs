use std::path::{Path, PathBuf};

use anyhow::Context;
use log::{debug, info};
use rust_ffmpeg::{FFmpegBuilder, PixelFormat, VideoFilter};

use crate::{FRAME_FORMAT_EXTENSION, LCD_WIDTH, definition::title::TitleDefinition};

impl TitleDefinition {
    pub fn frames_folder(&self, output_directory: &Path) -> anyhow::Result<PathBuf> {
        let video_name = self
            .video
            .file_stem()
            .with_context(|| format!("Failed to get source file name: {}", self.video.display()))?
            .to_os_string();

        let mut frames_folder = output_directory.join(video_name);
        frames_folder
            .as_mut_os_string()
            .push(format!("-{}-frames", self.name));

        Ok(frames_folder)
    }

    /// Returns the number of frames generated
    pub async fn create_frames(
        &self,
        title_directory: &Path,
        frames_directory: &Path,
    ) -> anyhow::Result<u32> {
        if tokio::fs::try_exists(&frames_directory)
            .await
            .unwrap_or_default()
        {
            tokio::fs::remove_dir_all(&frames_directory).await?;
        }
        tokio::fs::create_dir_all(&frames_directory)
            .await
            .with_context(|| {
                format!(
                    "Failed to create frame directory: {}",
                    frames_directory.display()
                )
            })?;

        let video_path = title_directory.join(&self.video);

        let mut input = rust_ffmpeg::Input::new(video_path)
            .option("probesize", "100M")
            .option("analyzeduration", "100M");

        if let Some(start) = self.start.clone() {
            input = input.seek(start.into());
        }

        if let Some(duration) = self.durration.clone() {
            input = input.duration(duration.into());
        }

        let builder = FFmpegBuilder::new()?
            .input(input)
            .output(
                rust_ffmpeg::Output::new(
                    frames_directory.join(format!("%d.{FRAME_FORMAT_EXTENSION}")),
                )
                .no_audio()
                .no_subtitles()
                .option("pix_fmt", PixelFormat::rgb24().to_string())
                .option("r", self.fps.to_string()),
            )
            .video_filter(VideoFilter::scale_aspect(LCD_WIDTH.into()))
            .overwrite()
            .on_progress(|progress| {
                info!(
                    "Gnerating Frames: completed {} frames",
                    progress.frame.unwrap_or_default()
                );
            });

        debug!("FFmpeg Command: {}", builder.command()?);
        info!("Generating image sequence with FFmpeg.");

        builder.spawn().await?.wait().await?;

        let frames_folder = frames_directory.to_path_buf();
        let frames = tokio::runtime::Handle::current()
            .spawn_blocking(move || std::fs::read_dir(frames_folder).map(std::fs::ReadDir::count))
            .await??;

        debug!("Output {frames} frames");

        u32::try_from(frames).with_context(|| format!("Frames exceeded maximum amount: {frames}"))
    }
}
