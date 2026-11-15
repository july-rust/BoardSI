//! F_002 BSIG v1 byte grammar. No struct transmute, native endian, or float regridding.
use sha2::{Digest, Sha256};

use crate::{NativeError, Result};

pub const MAX_PAYLOAD: usize = 64 * 1024 * 1024;
pub const MAX_REFS: usize = 65_535;
pub const MAX_SEGMENTS: usize = 1_000_000;
pub const ENVELOPE_LEN: usize = 52;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub id: u16,
    pub ty: u16,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryRecord {
    pub kind: u16,
    pub fields: Vec<Field>,
}

fn invalid(msg: &'static str) -> NativeError {
    NativeError::Invalid(msg)
}
fn read_u16(x: &[u8]) -> u16 {
    u16::from_le_bytes([x[0], x[1]])
}
fn read_u32(x: &[u8]) -> u32 {
    u32::from_le_bytes([x[0], x[1], x[2], x[3]])
}
fn read_u64(x: &[u8]) -> u64 {
    u64::from_le_bytes(x.try_into().expect("length checked"))
}
fn slice(data: &[u8], off: &mut usize, len: usize) -> Result<Vec<u8>> {
    let end = off.checked_add(len).ok_or_else(|| invalid("overflow"))?;
    let bytes = data
        .get(*off..end)
        .ok_or_else(|| invalid("truncated field"))?
        .to_vec();
    *off = end;
    Ok(bytes)
}
fn reduced_rational(bytes: &[u8]) -> Result<()> {
    if bytes.len() != 32 {
        return Err(invalid("rational length"));
    }
    let n = i128::from_le_bytes(bytes[0..16].try_into().expect("slice"));
    let d = i128::from_le_bytes(bytes[16..32].try_into().expect("slice"));
    if d <= 0 || n == i128::MIN {
        return Err(invalid("rational sign/overflow"));
    }
    let mut a = n.abs();
    let mut b = d;
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    if a != 1 {
        return Err(invalid("unreduced rational"));
    }
    Ok(())
}
fn point(bytes: &[u8]) -> Result<()> {
    if bytes.len() != 20 {
        return Err(invalid("point length"));
    }
    Ok(())
}
fn boundary(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 4 {
        return Err(invalid("short boundary"));
    }
    let count = read_u32(&bytes[..4]) as usize;
    if count == 0 || count > MAX_SEGMENTS {
        return Err(invalid("boundary count"));
    }
    let mut pos = 4usize;
    for _ in 0..count {
        let tag = *bytes
            .get(pos)
            .ok_or_else(|| invalid("boundary truncated"))?;
        let segment_size = match tag {
            1 => 41,
            2 => 62,
            _ => return Err(invalid("boundary tag")),
        };
        let seg = bytes
            .get(
                pos..pos
                    .checked_add(segment_size)
                    .ok_or_else(|| invalid("overflow"))?,
            )
            .ok_or_else(|| invalid("boundary data"))?;
        match tag {
            1 => {
                point(&seg[1..21])?;
                point(&seg[21..41])?;
            }
            2 => {
                point(&seg[1..21])?;
                point(&seg[21..41])?;
                point(&seg[41..61])?;
                if seg[61] > 1 {
                    return Err(invalid("arc direction"));
                }
            }
            _ => unreachable!(),
        }
        pos += segment_size;
    }
    if pos != bytes.len() {
        return Err(invalid("boundary trailing bytes"));
    }
    Ok(())
}
fn refs(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 4 {
        return Err(invalid("short ref list"));
    }
    let count = read_u32(&bytes[..4]) as usize;
    if count > MAX_REFS
        || 4usize.checked_add(count.checked_mul(4).ok_or_else(|| invalid("overflow"))?)
            != Some(bytes.len())
    {
        return Err(invalid("ref count/size"));
    }
    Ok(())
}
fn padstack(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 4 {
        return Err(invalid("short padstack"));
    }
    let count = read_u32(&bytes[..4]) as usize;
    if count == 0 || count > MAX_REFS {
        return Err(invalid("padstack count"));
    }
    let mut pos = 4usize;
    for _ in 0..count {
        let end = pos.checked_add(12).ok_or_else(|| invalid("overflow"))?;
        let entry = bytes
            .get(pos..end)
            .ok_or_else(|| invalid("short padstack entry"))?;
        let role = entry[4];
        let shape = entry[5];
        let reserved = read_u16(&entry[6..8]);
        let length = read_u32(&entry[8..12]) as usize;
        if role > 2 || shape > 4 || reserved != 0 {
            return Err(invalid("padstack role/shape"));
        }
        let body = bytes
            .get(end..end.checked_add(length).ok_or_else(|| invalid("overflow"))?)
            .ok_or_else(|| invalid("short padstack shape"))?;
        match shape {
            0 if length == 32 => reduced_rational(body)?,
            1 | 2 if length == 64 => {
                reduced_rational(&body[0..32])?;
                reduced_rational(&body[32..64])?;
            }
            3 if length == 103 => {
                if body[0] > 1 {
                    return Err(invalid("thermal bool"));
                }
                for start in [1, 33, 65] {
                    reduced_rational(&body[start..start + 32])?;
                }
                if read_u16(&body[97..99]) >= 360 || read_u32(&body[99..103]) == 0 {
                    return Err(invalid("thermal angle/spokes"));
                }
            }
            4 => boundary(body)?,
            _ => return Err(invalid("padstack shape length")),
        }
        pos = end + length;
    }
    if pos != bytes.len() {
        return Err(invalid("padstack trailing data"));
    }
    Ok(())
}
fn validate_field(f: &Field) -> Result<()> {
    let b = f.bytes.as_slice();
    match f.ty {
        1 if b.len() == 1 && b[0] <= 1 => {}
        2 if b.len() == 1 => {}
        3 if b.len() == 2 => {}
        4 if b.len() == 4 => {}
        5 => reduced_rational(b)?,
        6 => point(b)?,
        7 => {
            if b.len() < 4 {
                return Err(invalid("short point list"));
            }
            let count = read_u32(&b[..4]) as usize;
            if count < 2
                || 4usize.checked_add(count.checked_mul(20).ok_or_else(|| invalid("overflow"))?)
                    != Some(b.len())
            {
                return Err(invalid("point list count/length"));
            }
            for p in b[4..].chunks_exact(20) {
                point(p)?;
            }
        }
        8 => boundary(b)?,
        9 => refs(b)?,
        10 => {
            if b.len() < 4 || b[1] > 1 || b[3] != 0 {
                return Err(invalid("orientation flags"));
            }
            match b[0] {
                0 if b.len() == 4 && b[2] <= 3 => {}
                1 if b.len() == 12
                    && b[2] == 255
                    && f64::from_le_bytes(b[4..12].try_into().expect("slice")).is_finite() => {}
                _ => return Err(invalid("orientation kind/value")),
            }
        }
        11 => padstack(b)?,
        12 if b.len() == 8 && b[0..4] != b[4..8] => {}
        _ => return Err(invalid("field type/value")),
    }
    Ok(())
}
const TRACE: &[(u16, u16)] = &[(1, 7), (2, 5)];
const ARC: &[(u16, u16)] = &[(1, 6), (2, 6), (3, 6), (4, 1), (5, 5)];
const AREA: &[(u16, u16)] = &[(1, 8), (2, 9)];
const CUTOUT: &[(u16, u16)] = &[(1, 8)];
const PADSTACK: &[(u16, u16)] = &[(1, 11)];
const PAD: &[(u16, u16)] = &[(1, 6), (2, 10)];
const VIA: &[(u16, u16)] = &[(1, 6), (2, 2), (3, 12), (4, 9)];
const DRILL: &[(u16, u16)] = &[(1, 5), (2, 1), (3, 12)];
fn expected_fields(kind: u16) -> Result<&'static [(u16, u16)]> {
    match kind {
        1 => Ok(TRACE),
        2 => Ok(ARC),
        3 | 9 => Ok(AREA),
        4 => Ok(CUTOUT),
        5 => Ok(PADSTACK),
        6 => Ok(PAD),
        7 => Ok(VIA),
        8 => Ok(DRILL),
        _ => Err(invalid("unknown geometry kind")),
    }
}
impl GeometryRecord {
    pub fn validate(&self) -> Result<()> {
        let expected = expected_fields(self.kind)?;
        if expected.len() != self.fields.len() || self.fields.len() > 64 {
            return Err(invalid("required field set"));
        }
        for (field, (id, ty)) in self.fields.iter().zip(expected.iter()) {
            if field.id != *id || field.ty != *ty {
                return Err(invalid("field id/type/order"));
            }
            validate_field(field)?;
        }
        if self.kind == 7 && self.fields[1].bytes[0] > 3 {
            return Err(invalid("via kind"));
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut payload = Vec::with_capacity(256);
        payload.extend_from_slice(&1u16.to_le_bytes());
        payload.extend_from_slice(&(self.fields.len() as u16).to_le_bytes());
        for f in &self.fields {
            let len = u32::try_from(f.bytes.len()).map_err(|_| invalid("field length"))?;
            if payload
                .len()
                .checked_add(8)
                .and_then(|n| n.checked_add(f.bytes.len()))
                .ok_or_else(|| invalid("overflow"))?
                > MAX_PAYLOAD
            {
                return Err(invalid("payload budget"));
            }
            payload.extend_from_slice(&f.id.to_le_bytes());
            payload.extend_from_slice(&f.ty.to_le_bytes());
            payload.extend_from_slice(&len.to_le_bytes());
            payload.extend_from_slice(&f.bytes);
        }
        let mut out = Vec::with_capacity(ENVELOPE_LEN + payload.len());
        out.extend_from_slice(b"BSIG");
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&self.kind.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        out.extend_from_slice(Sha256::digest(&payload).as_slice());
        out.extend_from_slice(&payload);
        Ok(out)
    }
    pub fn decode(bytes: &[u8], expected_kind: u16) -> Result<Self> {
        if bytes.len() < ENVELOPE_LEN || bytes.len() > ENVELOPE_LEN + MAX_PAYLOAD {
            return Err(invalid("envelope budget"));
        }
        if &bytes[0..4] != b"BSIG" || read_u16(&bytes[4..6]) != 1 || read_u32(&bytes[8..12]) != 0 {
            return Err(invalid("BSIG magic/codec/flags"));
        }
        let kind = read_u16(&bytes[6..8]);
        if kind != expected_kind {
            return Err(invalid("SQL/BLOB geometry kind mismatch"));
        }
        expected_fields(kind)?;
        let length = read_u64(&bytes[12..20]);
        if length != (bytes.len() - ENVELOPE_LEN) as u64 {
            return Err(invalid("payload length"));
        }
        let payload = &bytes[ENVELOPE_LEN..];
        if Sha256::digest(payload).as_slice() != &bytes[20..52] {
            return Err(invalid("payload SHA-256"));
        }
        if payload.len() < 4 || read_u16(&payload[0..2]) != 1 {
            return Err(invalid("record version"));
        }
        let count = read_u16(&payload[2..4]) as usize;
        if count == 0 || count > 64 {
            return Err(invalid("TLV count"));
        }
        let mut fields = Vec::with_capacity(count);
        let mut pos = 4;
        for _ in 0..count {
            let hdr = payload
                .get(pos..pos + 8)
                .ok_or_else(|| invalid("TLV header"))?;
            let id = read_u16(&hdr[..2]);
            let ty = read_u16(&hdr[2..4]);
            let length = read_u32(&hdr[4..8]) as usize;
            pos += 8;
            let data = slice(payload, &mut pos, length)?;
            fields.push(Field {
                id,
                ty,
                bytes: data,
            });
        }
        if pos != payload.len() {
            return Err(invalid("TLV trailing bytes"));
        }
        let record = Self { kind, fields };
        record.validate()?;
        Ok(record)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn field(id: u16, ty: u16, bytes: Vec<u8>) -> Field {
        Field { id, ty, bytes }
    }
    fn p(x: i64, y: i64) -> Vec<u8> {
        let mut b = 0u32.to_le_bytes().to_vec();
        b.extend_from_slice(&x.to_le_bytes());
        b.extend_from_slice(&y.to_le_bytes());
        b
    }
    fn rational(n: i128, d: i128) -> Vec<u8> {
        let mut b = n.to_le_bytes().to_vec();
        b.extend_from_slice(&d.to_le_bytes());
        b
    }
    fn records() -> Vec<GeometryRecord> {
        let p0 = p(0, 0);
        let p1 = p(10, 0);
        let mut point_list = 2u32.to_le_bytes().to_vec();
        point_list.extend_from_slice(&p0);
        point_list.extend_from_slice(&p1);
        let mut b = 1u32.to_le_bytes().to_vec();
        b.push(1);
        b.extend_from_slice(&p0);
        b.extend_from_slice(&p1);
        let empty_refs = 0u32.to_le_bytes().to_vec();
        let mut span = 0u32.to_le_bytes().to_vec();
        span.extend_from_slice(&1u32.to_le_bytes());
        let mut entry = 1u32.to_le_bytes().to_vec();
        entry.extend_from_slice(&0u32.to_le_bytes());
        entry.extend_from_slice(&[0, 0, 0, 0]);
        entry.extend_from_slice(&32u32.to_le_bytes());
        entry.extend_from_slice(&rational(1, 10));
        vec![
            GeometryRecord {
                kind: 1,
                fields: vec![field(1, 7, point_list), field(2, 5, rational(1, 10))],
            },
            GeometryRecord {
                kind: 2,
                fields: vec![
                    field(1, 6, p0.clone()),
                    field(2, 6, p1.clone()),
                    field(3, 6, p0.clone()),
                    field(4, 1, vec![1]),
                    field(5, 5, rational(1, 10)),
                ],
            },
            GeometryRecord {
                kind: 3,
                fields: vec![field(1, 8, b.clone()), field(2, 9, empty_refs.clone())],
            },
            GeometryRecord {
                kind: 4,
                fields: vec![field(1, 8, b.clone())],
            },
            GeometryRecord {
                kind: 5,
                fields: vec![field(1, 11, entry)],
            },
            GeometryRecord {
                kind: 6,
                fields: vec![field(1, 6, p0.clone()), field(2, 10, vec![0, 0, 0, 0])],
            },
            GeometryRecord {
                kind: 7,
                fields: vec![
                    field(1, 6, p0.clone()),
                    field(2, 2, vec![0]),
                    field(3, 12, span.clone()),
                    field(4, 9, empty_refs.clone()),
                ],
            },
            GeometryRecord {
                kind: 8,
                fields: vec![
                    field(1, 5, rational(1, 10)),
                    field(2, 1, vec![1]),
                    field(3, 12, span),
                ],
            },
            GeometryRecord {
                kind: 9,
                fields: vec![field(1, 8, b), field(2, 9, empty_refs)],
            },
        ]
    }
    #[test]
    fn all_nine_round_trip() {
        for r in records() {
            let bytes = r.encode().unwrap();
            assert_eq!(bytes[0..4], *b"BSIG");
            assert_eq!(GeometryRecord::decode(&bytes, r.kind).unwrap(), r);
        }
    }
    #[test]
    fn corruptions_fail_closed() {
        for r in records() {
            let mut bytes = r.encode().unwrap();
            let last = bytes.len() - 1;
            bytes[last] ^= 1;
            assert!(GeometryRecord::decode(&bytes, r.kind).is_err());
        }
    }
    #[test]
    fn rejects_unreduced_and_pending_nan() {
        let mut r = records().remove(0);
        r.fields[1].bytes = rational(2, 4);
        assert!(r.encode().is_err());
        let mut p = records().remove(5);
        let mut o = vec![1, 0, 255, 0];
        o.extend_from_slice(&f64::NAN.to_le_bytes());
        p.fields[1].bytes = o;
        assert!(p.encode().is_err());
    }
    #[test]
    fn exact_i64_extremes() {
        let mut r = records().remove(0);
        let mut points = 2u32.to_le_bytes().to_vec();
        points.extend_from_slice(&p(i64::MIN, 0));
        points.extend_from_slice(&p(i64::MAX, -1));
        r.fields[0].bytes = points;
        assert_eq!(GeometryRecord::decode(&r.encode().unwrap(), 1).unwrap(), r);
    }
}
