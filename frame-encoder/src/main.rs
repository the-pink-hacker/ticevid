use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    video_sequence: PathBuf,
    frames: u32,
    out: PathBuf,
}

fn compress_color_space(rgb: [u8; 3]) -> u8 {
    let (red, green, blue) = (rgb[0], rgb[1], rgb[2]);
    let red = (red / 32) << 5;
    let green = green / 32;
    let blue = (blue / 64) << 3;
    red | green | blue
}

fn encode_frame(frame: &Path, output: &mut File) -> anyhow::Result<()> {
    let frame_png = image::io::Reader::open(frame)?.decode()?;

    let pixels = frame_png
        .as_rgb8()
        .context("Image wasn't 8-bit color.")?
        .pixels()
        .map(|pixel| compress_color_space(pixel.0))
        .collect::<Vec<_>>();

    output.seek(SeekFrom::End(0))?;
    output.write_all(&pixels)?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::try_parse()?;

    let mut output = File::create(args.out)?;

    for i in 1..=args.frames {
        let frame = args.video_sequence.join(format!("video{}.png", i));
        encode_frame(&frame, &mut output)?;
    }

    Ok(())
}
