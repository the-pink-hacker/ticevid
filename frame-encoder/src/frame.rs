use std::path::{Path, PathBuf};

use anyhow::Context;
use log::{debug, info};
use rust_ffmpeg::{FFmpegBuilder, PixelFormat, VideoFilter};

use crate::{VideoDefinition, FRAME_FORMAT_EXTENSION, LCD_WIDTH};

impl VideoDefinition {
    pub fn frames_folder(&self, output_folder: &Path) -> anyhow::Result<PathBuf> {
        let video_name = self
            .video
            .source
            .file_stem()
            .with_context(|| format!("Failed to get source file name: {:?}", self.video.source))?
            .to_os_string();

        let mut frames_folder = output_folder.join(video_name);
        frames_folder.as_mut_os_string().push("-frames");

        Ok(frames_folder)
    }

    /// Returns the number of frames generated
    pub async fn create_frames(
        &self,
        definition_folder: &Path,
        frames_folder: &Path,
    ) -> anyhow::Result<usize> {
        if tokio::fs::try_exists(&frames_folder)
            .await
            .unwrap_or_default()
        {
            tokio::fs::remove_dir_all(&frames_folder).await?;
        }
        tokio::fs::create_dir_all(&frames_folder)
            .await
            .with_context(|| format!("Failed to create frame directory: {frames_folder:?}"))?;

        let video_path = definition_folder.join(&self.video.source);

        let mut input = rust_ffmpeg::Input::new(video_path)
            .option("probesize", "100M")
            .option("analyzeduration", "100M");

        if let Some(start) = self.video.start.as_ref().cloned() {
            input = input.seek(start.into());
        }

        if let Some(duration) = self.video.durration.as_ref().cloned() {
            input = input.duration(duration.into());
        }

        let builder = FFmpegBuilder::new()?
            .input(input)
            .output(
                rust_ffmpeg::Output::new(
                    frames_folder.join(format!("%d.{FRAME_FORMAT_EXTENSION}")),
                )
                .no_audio()
                .no_subtitles()
                .option("pix_fmt", PixelFormat::rgb24().to_string())
                .option("r", self.video.fps.to_string()),
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

        let frames_folder = frames_folder.to_path_buf();
        let frames = tokio::runtime::Handle::current()
            .spawn_blocking(move || std::fs::read_dir(frames_folder).map(std::fs::ReadDir::count))
            .await??;

        debug!("Output {frames} frames");

        Ok(frames)
    }
}
