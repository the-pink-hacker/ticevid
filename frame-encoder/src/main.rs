use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    video_sequence: PathBuf,
    frames: usize,
    out: PathBuf,
}

fn compress_color_space(rgb: [u8; 3]) -> String {
    let (red, green, blue) = (rgb[0], rgb[1], rgb[2]);
    let red = (red / 32) << 5;
    let green = green / 32;
    let blue = (blue / 64) << 3;
    let pixel = red | green | blue;
    format!("${:x}", pixel)
}

fn encode_frame(frame: PathBuf, output: &PathBuf) -> anyhow::Result<()> {
    let frame_png = image::io::Reader::open(frame)?.decode()?;

    let pixels = frame_png
        .as_rgb8()
        .with_context(|| format!("Image wasn't 8-bit color."))?
        .pixels()
        .map(|pixel| compress_color_space(pixel.0))
        .collect::<Vec<_>>()
        .chunks_exact(frame_png.width() as usize)
        .map(|f| {
            let mut line = "\n.db ".to_string();
            line.push_str(&f.join(","));
            line
        })
        .collect::<String>();

    println!("{}", pixels);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::try_parse()?;

    for i in 1..=args.frames {
        let frame = args.video_sequence.join(format!("video{}.png", i));
        encode_frame(frame, &args.out)?;
    }

    Ok(())
}
