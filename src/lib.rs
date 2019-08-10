use std::{
    convert::TryFrom,
    error,
    io::{self, Error, Read, Write},
    str::FromStr,
};

/**
 * Read a "big-endian" u8 from the specified bit source. Since big-endian and
 * little-endian refer to byte order, not bit order, there is no difference
 * between the big-endian and little-endian encodings of a single byte.
 * Nevertheless, both `read_u8` and `read_u8_le` are provided for the sake of
 * uniformity.
 */
pub fn read_u8(src: &mut dyn Read) -> io::Result<u8> {
    let mut buf = [0; 1];
    src.read_exact(&mut buf)?;
    Ok(u8::from_be_bytes(buf))
}

/**
 * Read a "big-endian" u8 from the specified bit source. Since big-endian and
 * little-endian refer to byte order, not bit order, there is no difference
 * between the big-endian and little-endian encodings of a single byte.
 * Nevertheless, both `read_u8` and `read_u8_le` are provided for the sake of
 * uniformity.
 */
pub fn read_u8_le(src: &mut dyn Read) -> io::Result<u8> {
    let mut buf = [0; 1];
    src.read_exact(&mut buf)?;
    Ok(u8::from_le_bytes(buf))
}

/// Read a big-endian u16 from the specified bit source.
pub fn read_u16(src: &mut dyn Read) -> io::Result<u16> {
    let mut buf = [0; 2];
    src.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

/// Read a little-endian u16 from the specified bit source.
pub fn read_u16_le(src: &mut dyn Read) -> io::Result<u16> {
    let mut buf = [0; 2];
    src.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

/// Read a big-endian u32 from the specified bit source.
pub fn read_u32(src: &mut dyn Read) -> io::Result<u32> {
    let mut buf = [0; 4];
    src.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

/// Read a little-endian u32 from the specified bit source.
pub fn read_u32_le(src: &mut dyn Read) -> io::Result<u32> {
    let mut buf = [0; 4];
    src.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

/// Read a big-endian u64 from the specified bit source.
pub fn read_u64(src: &mut dyn Read) -> io::Result<u64> {
    let mut buf = [0; 8];
    src.read_exact(&mut buf)?;
    Ok(u64::from_be_bytes(buf))
}

/// Read a little-endian u64 from the specified bit source.
pub fn read_u64_le(src: &mut dyn Read) -> io::Result<u64> {
    let mut buf = [0; 8];
    src.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

/// Read a big-endian u128 from the specified bit source.
pub fn read_u128(src: &mut dyn Read) -> io::Result<u128> {
    let mut buf = [0; 16];
    src.read_exact(&mut buf)?;
    Ok(u128::from_be_bytes(buf))
}

/// Read a little-endian u128 from the specified bit source.
pub fn read_u128_le(src: &mut dyn Read) -> io::Result<u128> {
    let mut buf = [0; 16];
    src.read_exact(&mut buf)?;
    Ok(u128::from_le_bytes(buf))
}

/// Read a big-endian f32 from the specified bit source.
pub fn read_f32(src: &mut dyn Read) -> io::Result<f32> {
    read_u32(src).map(f32::from_bits)
}

/// Read a little-endian f32 from the specified bit source.
pub fn read_f32_le(src: &mut dyn Read) -> io::Result<f32> {
    read_u32_le(src).map(f32::from_bits)
}

/// Read a big-endian f64 from the specified bit source.
pub fn read_f64(src: &mut dyn Read) -> io::Result<f64> {
    read_u64(src).map(f64::from_bits)
}

/// Read a little-endian f64 from the specified bit source.
pub fn read_f64_le(src: &mut dyn Read) -> io::Result<f64> {
    read_u64_le(src).map(f64::from_bits)
}

/// Read `length` bytes into a new `Vec` from the specified bit source.
pub fn read_bytes(src: &mut dyn Read, length: u64) -> io::Result<Vec<u8>> {
    let mut handle = src.take(length);
    let length_usize = usize::try_from(length).unwrap_or(usize::max_value());
    let mut buf = Vec::with_capacity(length_usize);
    if let Err(e) = handle.read_to_end(&mut buf) {
        let msg = format!(
            "Expected {} bytes, but ran into an error instead: {:?}",
            length, e
        );
        Err(Error::new(e.kind(), msg))
    } else {
        Ok(buf)
    }
}

/**
 * Write a "big-endian" u8 to the specified bit sink. Since big-endian and
 * little-endian refer to byte order, not bit order, there is no difference
 * between the big-endian and little-endian encodings of a single byte.
 * Nevertheless, both `write_u8` and `write_u8_le` are provided for the sake of
 * uniformity.
 */
pub fn write_u8(out: &mut dyn Write, val: u8) -> io::Result<()> {
    out.write_all(&val.to_be_bytes())
}

/**
 * Write a "little-endian" u8 to the specified bit sink. Since big-endian and
 * little-endian refer to byte order, not bit order, there is no difference
 * between the big-endian and little-endian encodings of a single byte.
 * Nevertheless, both `write_u8` and `write_u8_le` are provided for the sake of
 * uniformity.
 */
pub fn write_u8_le(out: &mut dyn Write, val: u8) -> io::Result<()> {
    out.write_all(&val.to_le_bytes())
}

/// Write a big-endian u16 to the specified bit sink.
pub fn write_u16(out: &mut dyn Write, val: u16) -> io::Result<()> {
    out.write_all(&val.to_be_bytes())
}

/// Write a little-endian u16 to the specified bit sink.
pub fn write_u16_le(out: &mut dyn Write, val: u16) -> io::Result<()> {
    out.write_all(&val.to_le_bytes())
}

/// Write a big-endian u32 to the specified bit sink.
pub fn write_u32(out: &mut dyn Write, val: u32) -> io::Result<()> {
    out.write_all(&val.to_be_bytes())
}

/// Write a little-endian u32 to the specified bit sink.
pub fn write_u32_le(out: &mut dyn Write, val: u32) -> io::Result<()> {
    out.write_all(&val.to_le_bytes())
}

/// Write a big-endian u64 to the specified bit sink.
pub fn write_u64(out: &mut dyn Write, val: u64) -> io::Result<()> {
    out.write_all(&val.to_be_bytes())
}

/// Write a little-endian u64 to the specified bit sink.
pub fn write_u64_le(out: &mut dyn Write, val: u64) -> io::Result<()> {
    out.write_all(&val.to_le_bytes())
}

/// Write a big-endian u128 to the specified bit sink.
pub fn write_u128(out: &mut dyn Write, val: u128) -> io::Result<()> {
    out.write_all(&val.to_be_bytes())
}

/// Write a little-endian u128 to the specified bit sink.
pub fn write_u128_le(out: &mut dyn Write, val: u128) -> io::Result<()> {
    out.write_all(&val.to_le_bytes())
}

/// Write a big-endian f32 to the specified bit sink.
pub fn write_f32(out: &mut dyn Write, val: f32) -> io::Result<()> {
    write_u32(out, val.to_bits())
}

/// Write a little-endian f32 to the specified bit sink.
pub fn write_f32_le(out: &mut dyn Write, val: f32) -> io::Result<()> {
    write_u32_le(out, val.to_bits())
}

/// Write a big-endian f64 to the specified bit sink.
pub fn write_f64(out: &mut dyn Write, val: f64) -> io::Result<()> {
    write_u64(out, val.to_bits())
}

/// Write a little-endian f64 to the specified bit sink.
pub fn write_f64_le(out: &mut dyn Write, val: f64) -> io::Result<()> {
    write_u64_le(out, val.to_bits())
}

/// Write the specified `Vec` of bytes to the specified bit sink.
#[deprecated(since = "1.2.0", note = "replaced by write_byte_slice, which \
                                      doesn't require that the bytes be housed \
                                      in a Vec")]
pub fn write_bytes(out: &mut dyn Write, vals: Vec<u8>) -> io::Result<()> {
    write_byte_slice(out, &vals[..])
}

/// Write the specified slice of bytes to the specified bit sink.
pub fn write_byte_slice(out: &mut dyn Write, vals: &[u8]) -> io::Result<()> {
    out.write_all(vals)
}

pub fn prompt<T, E>(p: &str) -> io::Result<T>
  where
    T: FromStr<Err = E>,
    E: Into<Box<dyn error::Error + Send + Sync>>,
{
    let mut stdout = io::stdout();
    stdout.write_all(&p.as_bytes()[..])?;
    stdout.flush()?;
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf)?;
    match <T as FromStr>::from_str(&buf) {
        Ok(x) => Ok(x),
        Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io::{Cursor, Error, ErrorKind};

    #[test]
    fn writes_8u8() -> io::Result<()> {
        let v = 8u8;
        let mut c = Cursor::new(Vec::with_capacity(1));
        write_u8(&mut c, v)?;
        let buf = c.into_inner();
        match buf.get(0) {
            Some(8u8) => Ok(()),
            Some(x) => Err(Error::new(
                ErrorKind::Other,
                format!("Expected first byte in buffer to be {}, found {}", v, x),
            )),
            None => Err(Error::new(ErrorKind::Other, "Write failed")),
        }
    }

    #[test]
    fn writes_8u8_le() -> io::Result<()> {
        let v = 8u8;
        let mut c = Cursor::new(Vec::with_capacity(1));
        write_u8_le(&mut c, v)?;
        let buf = c.into_inner();
        match buf.get(0) {
            Some(8u8) => Ok(()),
            Some(x) => Err(Error::new(
                ErrorKind::Other,
                format!("Expected first byte in buffer to be {}, found {}", v, x),
            )),
            None => Err(Error::new(ErrorKind::Other, "Write failed")),
        }
    }

    #[test]
    fn writes_4660u16() -> io::Result<()> {
        let v = 0x1234u16;
        let high_bits = 0x12u8;
        let low_bits = 0x34u8;
        let mut c = Cursor::new(Vec::with_capacity(2));
        write_u16(&mut c, v)?;
        let buf = c.into_inner();
        match &buf[..2] {
            &[0x12u8, 0x34u8] => Ok(()),
            &[x, y] => {
                let msg = format!(
                    "Expected buffer contents to be [{}, {}], found [{}, {}]",
                    high_bits, low_bits, x, y);
                Err(Error::new(ErrorKind::Other, msg))
            },
            slice => {
                let msg = format!(
                    "Expected buffer contents to be [{}, {}], found {:?}",
                    high_bits, low_bits, slice);
                Err(Error::new(ErrorKind::Other, msg))
            },
        }
    }

    #[test]
    fn writes_4660u16_le() -> io::Result<()> {
        let v = 0x1234u16;
        let high_bits = 0x12u8;
        let low_bits = 0x34u8;
        let mut c = Cursor::new(Vec::with_capacity(2));
        write_u16_le(&mut c, v)?;
        let buf = c.into_inner();
        match &buf[..2] {
            &[0x34u8, 0x12u8] => Ok(()),
            &[y, x] => {
                let msg = format!(
                    "Expected buffer contents to be [{}, {}], found [{}, {}]",
                    low_bits, high_bits, y, x);
                Err(Error::new(ErrorKind::Other, msg))
            },
            slice => {
                let msg = format!(
                    "Expected buffer contents to be [{}, {}], found {:?}",
                    low_bits, high_bits, slice);
                Err(Error::new(ErrorKind::Other, msg))
            },
        }
    }
}
