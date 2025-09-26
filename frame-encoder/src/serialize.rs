use std::{collections::HashMap, hash::Hash, io::SeekFrom, path::PathBuf};

use anyhow::{bail, Context};
use indexmap::IndexMap;
use log::debug;
use tokio::io::{AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt};
use u24::u24;

#[derive(Debug, Clone)]
pub struct SerialBuilder<S: Hash + Eq + Clone + std::fmt::Debug> {
    sectors: IndexMap<S, SerialSectorBuilder<S>>,
}

// Default macro requires S to implement default
// We don't want that
impl<S: Hash + Eq + Clone + std::fmt::Debug> Default for SerialBuilder<S> {
    fn default() -> Self {
        Self {
            sectors: IndexMap::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct SerialTracker<S: Hash + Eq> {
    sector_offsets: HashMap<S, usize>,
}

#[derive(Debug, Clone)]
pub struct SerialSectorBuilder<S: Hash + Eq> {
    fields: Vec<SerialField<S>>,
}

// Default macro requires S to implement default
// We don't want that
impl<S: Hash + Eq + std::fmt::Debug> Default for SerialSectorBuilder<S> {
    fn default() -> Self {
        Self {
            fields: Vec::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum SerialField<S: Hash + Eq> {
    /// Refences data that isn't know yet
    Dynamic {
        origin: S,
        sector: S,
        /// Index from begining of first sector
        index: usize,
    },
    /// File to be loaded on build
    External {
        path: PathBuf,
        /// Is checked on build
        size: usize,
    },
    U8(u8),
    U16(u16),
    U24(u24),
    U32(u32),
    U64(u64),
    /// Variable width null terminated string
    String(String),
    /// Fills data up to offset from origin
    /// Errors if past origin
    Fill {
        origin: S,
        fill: usize,
    },
}

impl<S: Hash + Eq + Clone + std::fmt::Debug> SerialBuilder<S> {
    pub fn sector(mut self, key: S, builder: SerialSectorBuilder<S>) -> Self {
        self.sectors.insert(key, builder);
        self
    }

    pub fn sector_default(self, key: S) -> Self {
        self.sector(key, SerialSectorBuilder::<S>::default())
    }

    pub async fn build(
        self,
        buffer: &mut (impl AsyncWrite + Unpin + AsyncSeek),
    ) -> anyhow::Result<()> {
        let tracker = SerialTracker::new(&self.sectors).await?;

        for (_, sector) in &self.sectors {
            sector.build(buffer, &self.sectors, &tracker).await?;
        }

        buffer.flush().await?;

        Ok(())
    }
}

impl<S: Hash + Eq + Clone + std::fmt::Debug> SerialTracker<S> {
    fn offset_field_from_sector(
        &self,
        from_sector: &S,
        to_sector: &S,
        to_index: usize,
        sectors: &IndexMap<S, SerialSectorBuilder<S>>,
        tracker: &SerialTracker<S>,
    ) -> anyhow::Result<usize> {
        let from_offset = self
            .sector_offsets
            .get(from_sector)
            .cloned()
            .with_context(|| format!("Sector does not exist: {from_sector:#?}"))?;
        let to_offset = self
            .sector_offsets
            .get(to_sector)
            .cloned()
            .with_context(|| format!("Sector does not exist: {to_sector:#?}"))?;
        let mut offset = to_offset.checked_sub(from_offset).with_context(|| {
            format!("From sector was ahead of to sector: {from_offset} > {to_offset}")
        })?;

        let fields = &sectors
            .get(to_sector)
            .with_context(|| format!("Sector does not exist: {to_sector:#?}"))?
            .fields;

        if fields.len() <= to_index {
            bail!(
                "Can't index into sector; not enough fields. Sector: {:#?}, Length: {}, Index: {}",
                to_sector,
                fields.len(),
                to_index
            );
        }

        // Adds the sizes of all fields up to the index
        for (field, _) in fields.iter().zip(0..to_index) {
            offset += field.calculate_size(offset, tracker)?;
        }

        Ok(offset)
    }

    /// Caches all sector starting and ending offsets
    async fn new(sectors: &IndexMap<S, SerialSectorBuilder<S>>) -> anyhow::Result<Self> {
        let mut tracker = Self {
            sector_offsets: HashMap::with_capacity(sectors.len()),
        };

        let mut offset = 0;

        for (sector_id, sector) in sectors {
            let start = offset;

            for field in &sector.fields {
                offset += field.calculate_size(offset, &tracker)?;
            }

            let old_value = tracker.sector_offsets.insert(sector_id.clone(), start);

            if let Some(start) = old_value {
                bail!(
                    "Sector offsets was already populated; key: {:#?}, start: {start}",
                    sector_id
                );
            }
        }

        debug!("Tracked all sectors");

        Ok(tracker)
    }

    fn offset_from_origin(&self, origin_sector: &S) -> anyhow::Result<usize> {
        self.sector_offsets
            .get(origin_sector)
            .with_context(|| {
                format!("Failed to find origin; was likely in front or missing: {origin_sector:#?}")
            })
            .cloned()
    }
}

impl<S: Hash + Eq + Clone + std::fmt::Debug> SerialSectorBuilder<S> {
    fn field(mut self, field: SerialField<S>) -> Self {
        self.fields.push(field);
        self
    }

    pub fn string(self, value: impl Into<String>) -> Self {
        self.field(SerialField::String(value.into()))
    }

    pub fn u8(self, value: impl Into<u8>) -> Self {
        self.field(SerialField::U8(value.into()))
    }

    pub fn i8(self, value: impl Into<i8>) -> Self {
        self.field(SerialField::U8(value.into() as u8))
    }

    pub fn u16(self, value: impl Into<u16>) -> Self {
        self.field(SerialField::U16(value.into()))
    }

    pub fn i16(self, value: impl Into<i16>) -> Self {
        self.field(SerialField::U16(value.into() as u16))
    }

    pub fn u24(self, value: impl Into<u24>) -> Self {
        self.field(SerialField::U24(value.into()))
    }

    pub fn u32(self, value: impl Into<u32>) -> Self {
        self.field(SerialField::U32(value.into()))
    }

    pub fn i32(self, value: impl Into<i32>) -> Self {
        self.field(SerialField::U32(value.into() as u32))
    }

    pub fn u64(self, value: impl Into<u64>) -> Self {
        self.field(SerialField::U64(value.into()))
    }

    pub fn i64(self, value: impl Into<i64>) -> Self {
        self.field(SerialField::U64(value.into() as u64))
    }

    pub fn dynamic(self, origin: S, sector: S, index: usize) -> Self {
        self.field(SerialField::Dynamic {
            origin,
            sector,
            index,
        })
    }

    pub fn fill(self, origin: S, fill: usize) -> Self {
        self.field(SerialField::Fill { origin, fill })
    }

    pub async fn file(self, path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let path = path.into();
        let size = tokio::fs::metadata(&path).await?.len() as usize;

        Ok(self.field(SerialField::External { path, size }))
    }

    async fn build(
        &self,
        buffer: &mut (impl AsyncWrite + Unpin + AsyncSeek),
        sectors: &IndexMap<S, SerialSectorBuilder<S>>,
        tracker: &SerialTracker<S>,
    ) -> anyhow::Result<()> {
        for field in &self.fields {
            field.build(buffer, sectors, tracker).await?;
        }

        Ok(())
    }
}

impl<S: Hash + Eq + Clone + std::fmt::Debug> SerialField<S> {
    fn calculate_size(&self, offset: usize, tracker: &SerialTracker<S>) -> anyhow::Result<usize> {
        match self {
            // Add one for null terminator
            Self::String(value) => Ok(value.len() + 1),
            Self::Dynamic {
                sector: _,
                index: _,
                origin: _,
            }
            | Self::U24(_) => Ok(3),
            Self::U8(_) => Ok(1),
            Self::U16(_) => Ok(2),
            Self::U32(_) => Ok(4),
            Self::U64(_) => Ok(8),
            Self::External { path: _, size } => Ok(*size),
            Self::Fill { origin, fill } => {
                let origin_position = tracker.offset_from_origin(origin)?;
                Self::fill_size(offset, origin_position, *fill)
            }
        }
    }

    async fn build(
        &self,
        buffer: &mut (impl AsyncWrite + Unpin + AsyncSeek),
        sectors: &IndexMap<S, SerialSectorBuilder<S>>,
        tracker: &SerialTracker<S>,
    ) -> anyhow::Result<()> {
        match self {
            Self::String(value) => {
                buffer.write_all(value.as_bytes()).await?;
                buffer.write_u8(0).await?;
            }
            Self::Dynamic {
                sector,
                index,
                origin,
            } => {
                let pointer =
                    tracker.offset_field_from_sector(origin, sector, *index, sectors, tracker)?;
                let pointer = u24::checked_from_u32(pointer as u32).with_context(|| {
                    format!(
                        "Pointer exceeds 24-bit limit: {} bytes > {} bytes",
                        pointer,
                        u24::MAX
                    )
                })?;
                buffer.write_all(&pointer.to_le_bytes()).await?;
            }
            Self::U8(value) => {
                buffer.write_u8(*value).await?;
            }
            Self::U16(value) => {
                buffer.write_u16(*value).await?;
            }
            Self::U24(value) => {
                buffer.write_all(&value.to_le_bytes()).await?;
            }
            Self::U32(value) => {
                buffer.write_u32(*value).await?;
            }
            Self::U64(value) => {
                buffer.write_u64(*value).await?;
            }
            Self::Fill { origin, fill } => {
                let offset = buffer.stream_position().await? as usize;
                let origin_position = tracker.offset_from_origin(origin)?;
                let fill_amount = Self::fill_size(offset, origin_position, *fill)?;
                buffer.seek(SeekFrom::Current(fill_amount as i64)).await?;
            }
            Self::External { path, size } => {
                let data = tokio::fs::read(path).await?;
                let read = buffer.write(&data).await?;

                if read != *size {
                    bail!(
                        "External file has incorrect file size:\n\
                         Expected: {size} bytes, Found: {read} bytes\n\
                         Path: {path:?}"
                    );
                }
            }
        }

        Ok(())
    }

    fn fill_size(offset: usize, origin_position: usize, fill: usize) -> anyhow::Result<usize> {
        let fill_start = offset.checked_sub(origin_position).with_context(|| format!("Failed to serialize; current position is before fill origin: {offset} < {origin_position}"))?;
        fill.checked_sub(fill_start).with_context(|| {
            format!("Failed to serialize; fill start is past fill amount: {fill_start} > {fill}")
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    type Builder = SerialBuilder<ExampleSectorKey>;
    type SectorBuilder = SerialSectorBuilder<ExampleSectorKey>;

    #[derive(Debug, Clone, Hash, PartialEq, Eq)]
    enum ExampleSectorKey {
        First,
        Second,
        Third,
    }

    #[tokio::test]
    async fn sector_string() {
        let expected = b"This is a test\x00";
        let mut buffer = Cursor::new(Vec::with_capacity(expected.len()));

        Builder::default()
            .sector(
                ExampleSectorKey::First,
                SectorBuilder::default().string("This is a test"),
            )
            .build(&mut buffer)
            .await
            .unwrap();

        assert_eq!(buffer.into_inner(), expected);
    }

    #[tokio::test]
    async fn sector_u24() {
        let expected = [0x12, 0x34, 0x56];
        let mut buffer = Cursor::new(Vec::with_capacity(expected.len()));

        Builder::default()
            .sector(
                ExampleSectorKey::First,
                SectorBuilder::default().u24(u24::from_le_bytes([0x12, 0x34, 0x56])),
            )
            .build(&mut buffer)
            .await
            .unwrap();

        assert_eq!(buffer.into_inner(), expected);
    }

    #[tokio::test]
    async fn sector_dynamic() {
        let expected = b"\xFF\x06\x00\x00\x13\x00\x00first string\x00second string\x00";
        let mut buffer = Cursor::new(Vec::with_capacity(expected.len()));

        Builder::default()
            .sector(ExampleSectorKey::First, SectorBuilder::default().u8(0xFF))
            .sector(
                ExampleSectorKey::Second,
                SectorBuilder::default()
                    .dynamic(ExampleSectorKey::Second, ExampleSectorKey::Third, 0)
                    .dynamic(ExampleSectorKey::Second, ExampleSectorKey::Third, 1),
            )
            .sector(
                ExampleSectorKey::Third,
                SectorBuilder::default()
                    .string("first string")
                    .string("second string"),
            )
            .build(&mut buffer)
            .await
            .unwrap();

        assert_eq!(buffer.into_inner(), expected);
    }

    #[tokio::test]
    async fn sector_fill() {
        let expected = [
            b'T', b'e', b's', b't', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF,
        ];
        let mut buffer = Cursor::new(Vec::with_capacity(expected.len()));

        Builder::default()
            .sector_default(ExampleSectorKey::First)
            .sector(
                ExampleSectorKey::Second,
                SectorBuilder::default()
                    .string("Test")
                    .fill(ExampleSectorKey::First, 16)
                    .u8(0xFF),
            )
            .build(&mut buffer)
            .await
            .unwrap();

        assert_eq!(buffer.into_inner(), expected);
    }

    #[tokio::test]
    async fn sector_fill_end() {
        let expected = b"Test\x00";
        let mut buffer = Cursor::new(Vec::with_capacity(expected.len()));

        Builder::default()
            .sector_default(ExampleSectorKey::First)
            .sector(
                ExampleSectorKey::Second,
                SectorBuilder::default()
                    .string("Test")
                    .fill(ExampleSectorKey::First, 16),
            )
            .build(&mut buffer)
            .await
            .unwrap();

        assert_eq!(buffer.into_inner(), expected);
    }

    #[tokio::test]
    async fn sector_fill_overflow() {
        let mut buffer = Cursor::new(Vec::new());

        let result = Builder::default()
            .sector_default(ExampleSectorKey::First)
            .sector(
                ExampleSectorKey::Second,
                SectorBuilder::default()
                    .string("Test")
                    .fill(ExampleSectorKey::First, 2),
            )
            .build(&mut buffer)
            .await;

        assert!(result.is_err());
    }
}
