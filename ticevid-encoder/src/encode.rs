use std::iter::Peekable;

const QOI_TAG_LITERAL: u8 = 0xFF;
const QOI_TAG_DIFF: u8 = 0b0000_0000;
const QOI_TAG_INDEX: u8 = 0b1000_0000;
const QOI_TAG_RUN: u8 = 0b1100_0000;

pub trait FrameEncoder {
    fn encode(&mut self, frame: &[u8], output_buffer: &mut [u8]) -> anyhow::Result<usize>;
}

type Lzss = lzss::Lzss<10, 4, 0x00, { 1 << 10 }, { 2 << 10 }>;

pub struct LzssEncoder;

impl FrameEncoder for LzssEncoder {
    fn encode(&mut self, frame: &[u8], output_buffer: &mut [u8]) -> anyhow::Result<usize> {
        let compressed_bytes = Lzss::compress_stack(
            lzss::SliceReader::new(frame),
            lzss::SliceWriter::new(output_buffer),
        )?;

        Ok(compressed_bytes)
    }
}

/// Modified QOI algorithm
/// Based on <https://qoiformat.org/qoi-specification.pdf/>
///
/// The major difference is there's only one color channel.
/// Diff and luma have been replaced with a 7-bit diff
pub struct QoiEncoder {
    output_index: usize,
    index_table: [u8; 64],
    previous_pixel: u8,
}

impl QoiEncoder {
    fn index_insert(&mut self, value: u8) {
        let index = Self::index_hash(value);
        self.index_table[index as usize] = value;
    }

    fn index_has(&self, value: u8) -> bool {
        let index = Self::index_hash(value);
        self.index_table[index as usize] == value
    }

    fn index_hash(value: u8) -> u8 {
        value % 64
    }

    fn write(&mut self, value: u8, output_buffer: &mut [u8]) {
        output_buffer[self.output_index] = value;
        self.output_index += 1;
    }

    fn write_run(&mut self, value: u8, output_buffer: &mut [u8]) {
        assert!((0..63).contains(&value));
        self.write(QOI_TAG_RUN | value, output_buffer);
    }

    fn write_literal(&mut self, value: u8, output_buffer: &mut [u8]) {
        self.write(QOI_TAG_LITERAL, output_buffer);
        self.write(value, output_buffer);
    }

    fn write_index(&mut self, value: u8, output_buffer: &mut [u8]) {
        self.write(QOI_TAG_INDEX | Self::index_hash(value), output_buffer);
    }

    fn write_diff(&mut self, value: i8, output_buffer: &mut [u8]) {
        let diff = match value {
            i8::MIN..-64 | 0 | 65..=i8::MAX => panic!("Invalid diff chunk value of {value}"),
            -64..0 => 127u8.strict_add_signed(value + 1),
            1..=64 => value.cast_unsigned() - 1,
        };

        self.write(QOI_TAG_DIFF | diff, output_buffer);
    }

    fn create_run<I: Iterator<Item = u8>>(
        &mut self,
        pixels: &mut Peekable<I>,
        output_buffer: &mut [u8],
    ) -> QoiControl {
        for run_index in 0..63 {
            if let Some(&pixel) = pixels.peek() {
                if pixel == self.previous_pixel {
                    pixels.next();
                    if let Some(&next_pixel) = pixels.peek() {
                        if next_pixel != self.previous_pixel || run_index == 62 {
                            self.write_run(run_index, output_buffer);
                            // Write run
                            return QoiControl::Wrote;
                        }
                    } else {
                        self.write_run(run_index, output_buffer);
                        // Write run and no more data to encode
                        return QoiControl::Done;
                    }
                } else {
                    // Pixel can't run
                    return QoiControl::Invalid;
                }
            } else {
                // No more data to encode
                return QoiControl::Done;
            }
        }

        // Pixel can't run
        QoiControl::Invalid
    }

    fn create_index<I: Iterator<Item = u8>>(
        &mut self,
        pixels: &mut Peekable<I>,
        output_buffer: &mut [u8],
    ) -> QoiControl {
        if let Some(&pixel) = pixels.peek() {
            if self.index_has(pixel) {
                pixels.next();
                self.write_index(pixel, output_buffer);
                self.previous_pixel = pixel;
                QoiControl::Wrote
            } else {
                QoiControl::Invalid
            }
        } else {
            QoiControl::Done
        }
    }

    fn create_difference<I: Iterator<Item = u8>>(
        &mut self,
        pixels: &mut Peekable<I>,
        output_buffer: &mut [u8],
    ) -> QoiControl {
        if let Some(&pixel) = pixels.peek() {
            let diff = pixel.wrapping_sub(self.previous_pixel).cast_signed();

            match diff {
                i8::MIN..-64 | 0 | 65..=i8::MAX => QoiControl::Invalid,
                -64..0 | 1..=64 => {
                    pixels.next();
                    self.index_insert(pixel);
                    self.write_diff(diff, output_buffer);
                    self.previous_pixel = pixel;
                    QoiControl::Wrote
                }
            }
        } else {
            QoiControl::Done
        }
    }
}

enum QoiControl {
    /// Chunk has been written; start over chunk search
    Wrote,
    /// Another chunk needs to be tried
    Invalid,
    /// No more data to process
    Done,
}

impl Default for QoiEncoder {
    fn default() -> Self {
        Self {
            index_table: [0; 64],
            previous_pixel: 0,
            output_index: 0,
        }
    }
}

impl FrameEncoder for QoiEncoder {
    fn encode(&mut self, frame: &[u8], output_buffer: &mut [u8]) -> anyhow::Result<usize> {
        let mut pixels = frame.iter().copied().peekable();

        loop {
            match self.create_run(&mut pixels, output_buffer) {
                QoiControl::Wrote => continue,
                QoiControl::Invalid => (),
                QoiControl::Done => break,
            }

            match self.create_index(&mut pixels, output_buffer) {
                QoiControl::Wrote => continue,
                QoiControl::Invalid => (),
                QoiControl::Done => break,
            }

            match self.create_difference(&mut pixels, output_buffer) {
                QoiControl::Wrote => continue,
                QoiControl::Invalid => (),
                QoiControl::Done => break,
            }

            if let Some(pixel) = pixels.next() {
                self.write_literal(pixel, output_buffer);
                self.index_insert(pixel);
                self.previous_pixel = pixel;
            } else {
                break;
            }
        }

        Ok(self.output_index)
    }
}
