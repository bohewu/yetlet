//! Known noninterlaced 8-bit RGB/RGBA browser screenshots, matching png_pixels.
use super::require;
use crate::tooling::{Result, fail};
use std::io::Read;
pub struct Pixels {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub rows: Vec<Vec<u8>>,
}
impl Pixels {
    pub fn rgb(&self, x: usize, y: usize) -> Result<[u8; 3]> {
        require(x < self.width && y < self.height, "PNG point bounds")?;
        let start = x * self.channels;
        Ok(self.rows[y][start..start + 3].try_into().unwrap())
    }
}
pub fn decode(data: &[u8]) -> Result<Pixels> {
    require(data.starts_with(b"\x89PNG\r\n\x1a\n"), "PNG signature")?;
    let mut offset = 8;
    let mut compressed = Vec::new();
    let mut dimensions = None;
    while offset < data.len() {
        let length = data
            .get(offset..offset + 4)
            .ok_or_else(|| fail("PNG truncated length"))?;
        let length = u32::from_be_bytes(length.try_into().unwrap()) as usize;
        let chunk = data
            .get(offset + 4..offset + 12 + length)
            .ok_or_else(|| fail("PNG truncated chunk"))?;
        let kind = &chunk[..4];
        let bytes = &chunk[4..4 + length];
        if kind == b"IHDR" {
            require(bytes.len() == 13, "PNG IHDR size")?;
            let w = u32::from_be_bytes(bytes[..4].try_into().unwrap()) as usize;
            let h = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as usize;
            require(
                bytes[8] == 8 && [2, 6].contains(&bytes[9]) && bytes[10..] == [0, 0, 0],
                "unsupported PNG encoding",
            )?;
            require(
                w > 0 && h > 0 && w <= 16384 && h <= 16384,
                "PNG dimension bound",
            )?;
            dimensions = Some((w, h, if bytes[9] == 2 { 3 } else { 4 }));
        } else if kind == b"IDAT" {
            require(
                compressed.len() + bytes.len() <= 64 * 1024 * 1024,
                "PNG compressed bound",
            )?;
            compressed.extend(bytes);
        }
        offset += length + 12;
    }
    let (width, height, channels) = dimensions.ok_or_else(|| fail("PNG missing IHDR"))?;
    let stride = width * channels;
    require(
        height * (stride + 1) <= 256 * 1024 * 1024,
        "PNG decoded bound",
    )?;
    let expected = height * (stride + 1);
    let mut raw = Vec::new();
    flate2::read::ZlibDecoder::new(&compressed[..])
        .take((expected + 1) as u64)
        .read_to_end(&mut raw)?;
    require(raw.len() == expected, "PNG decompressed size")?;
    let mut previous = vec![0u8; stride];
    let mut rows = Vec::new();
    for encoded in raw.chunks_exact(stride + 1) {
        let mode = encoded[0];
        let mut row = encoded[1..].to_vec();
        for x in 0..stride {
            let left = if x >= channels { row[x - channels] } else { 0 };
            let above = previous[x];
            let corner = if x >= channels {
                previous[x - channels]
            } else {
                0
            };
            let value = match mode {
                0 => 0,
                1 => left,
                2 => above,
                3 => ((left as u16 + above as u16) / 2) as u8,
                4 => {
                    let p = left as i16 + above as i16 - corner as i16;
                    let a = (p - left as i16).abs();
                    let b = (p - above as i16).abs();
                    let c = (p - corner as i16).abs();
                    if a <= b && a <= c {
                        left
                    } else if b <= c {
                        above
                    } else {
                        corner
                    }
                }
                _ => return Err(fail("unsupported PNG filter")),
            };
            row[x] = row[x].wrapping_add(value);
        }
        previous = row.clone();
        rows.push(row);
    }
    Ok(Pixels {
        width,
        height,
        channels,
        rows,
    })
}
