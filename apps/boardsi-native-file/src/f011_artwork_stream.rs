//! Opt-in F011 framed presentation transport. The legacy native receipt is
//! unchanged for callers without a request token. No on-disk format changes.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::Write;

pub(crate) const MAX_ITEMS: usize = 10_000;
const MAX_FRAME: usize = 60_000;
const MAX_TOTAL: usize = 16 * 1024 * 1024;
const MAX_BATCH_ITEMS: usize = 128;
const MAX_BATCH_BYTES: usize = 48_000;
const PROTOCOL: &str = "BSI_ARTWORK_STREAM_V1";

pub(crate) fn request() -> Result<Option<String>, &'static str> {
    match std::env::var("BOARDSI_ARTWORK_REQUEST") {
        Ok(s) if s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit()) => Ok(Some(s)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        _ => Err("BSI_F011_REQUEST_INVALID"),
    }
}

pub(crate) fn item_limit() -> usize {
    if matches!(request(), Ok(Some(_))) {
        MAX_ITEMS
    } else {
        128
    }
}

struct Frames {
    lines: Vec<Vec<u8>>,
    bytes: usize,
    digest: Sha256,
}

impl Frames {
    fn new() -> Self {
        Self {
            lines: Vec::new(),
            bytes: 0,
            digest: Sha256::new(),
        }
    }

    fn push(&mut self, value: &Value, hash: bool) -> Result<(), &'static str> {
        let mut line = serde_json::to_vec(value).map_err(|_| "BSI_F011_SERIALIZE")?;
        line.push(b'\n');
        if line.len() >= MAX_FRAME || line.len() > MAX_TOTAL - self.bytes {
            return Err("BSI_F011_TRANSFER_LIMIT");
        }
        self.bytes += line.len();
        if hash {
            self.digest.update(&line);
        }
        self.lines.push(line);
        Ok(())
    }
}

fn batch(
    frames: &mut Frames,
    token: &str,
    sequence: usize,
    items: Vec<Value>,
) -> Result<(), &'static str> {
    frames.push(
        &json!({"protocol":PROTOCOL,"request":token,"kind":"batch",
            "sequence":sequence,"items":items}),
        true,
    )
}

// Build the complete bounded transport before writing a byte. If any frame or
// aggregate exceeds the presentation budget, a structural-only receipt is
// emitted instead; an incomplete stream can never certify a full artwork.
fn encode(mut receipt: Value, token: &str) -> Result<Frames, &'static str> {
    let mut rows = Vec::new();
    if receipt["artwork_status"] == "RENDERABLE" {
        let artwork = receipt["artwork"]
            .as_object_mut()
            .ok_or("BSI_F011_ARTWORK_INVALID")?;
        rows = artwork
            .remove("items")
            .and_then(|v| match v {
                Value::Array(items) => Some(items),
                _ => None,
            })
            .ok_or("BSI_F011_ARTWORK_INVALID")?;
        if rows.is_empty() || rows.len() > MAX_ITEMS {
            return Err("BSI_F011_TRANSFER_LIMIT");
        }
    }
    let count = rows.len();
    let mut frames = Frames::new();
    frames.push(
        &json!({"protocol":PROTOCOL,"request":token,"kind":"begin",
            "item_count":count,"receipt":receipt}),
        true,
    )?;
    let mut pending = Vec::new();
    let mut pending_bytes = 0usize;
    let mut sequence = 0usize;
    for row in rows {
        let bytes = serde_json::to_vec(&row)
            .map_err(|_| "BSI_F011_SERIALIZE")?
            .len()
            + 1;
        if bytes > MAX_BATCH_BYTES {
            return Err("BSI_F011_TRANSFER_LIMIT");
        }
        if !pending.is_empty()
            && (pending.len() == MAX_BATCH_ITEMS || pending_bytes + bytes > MAX_BATCH_BYTES)
        {
            batch(&mut frames, token, sequence, std::mem::take(&mut pending))?;
            sequence += 1;
            pending_bytes = 0;
        }
        pending_bytes += bytes;
        pending.push(row);
    }
    if !pending.is_empty() {
        batch(&mut frames, token, sequence, pending)?;
        sequence += 1;
    }
    let digest = format!("{:x}", frames.digest.clone().finalize());
    frames.push(
        &json!({"protocol":PROTOCOL,"request":token,"kind":"end",
            "batch_count":sequence,"item_count":count,"sha256":digest}),
        false,
    )?;
    Ok(frames)
}

pub(crate) fn write_receipt(
    mut receipt: Value,
    token: &str,
    mut out: impl Write,
) -> Result<(), &'static str> {
    // Retain only structural metadata for the fallback, not a second full
    // copy of the geometry. Move the artwork back for the first encode.
    let artwork = receipt
        .as_object_mut()
        .ok_or("BSI_F011_ARTWORK_INVALID")?
        .remove("artwork");
    let mut fallback = receipt.clone();
    if let Some(artwork) = artwork {
        receipt["artwork"] = artwork;
    }
    let frames = match encode(receipt, token) {
        Ok(frames) => frames,
        Err(reason) if fallback["artwork_status"] == "RENDERABLE" => {
            fallback["artwork_status"] = json!("NOT_RENDERED");
            fallback["artwork_reason"] = json!(reason);
            encode(fallback, token)?
        }
        Err(reason) => return Err(reason),
    };
    for line in frames.lines {
        out.write_all(&line).map_err(|_| "BSI_F011_WRITE")?;
    }
    out.flush().map_err(|_| "BSI_F011_WRITE")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";

    fn receipt(count: usize) -> Value {
        let items = (0..count)
            .map(|i| json!({"kind":"trace","id":i.to_string(),"a":[i,0],"b":[i,1]}))
            .collect::<Vec<_>>();
        json!({"protocol":"BSI_NATIVE_FILE_V1","action":"open",
            "artwork_status":"RENDERABLE","artwork":{"items":items,"units":"integer-nm"}})
    }

    #[test]
    fn frames_reassemble_exactly_at_scale() {
        for count in [128, 129, 1_000, 10_000] {
            let original = receipt(count);
            let mut bytes = Vec::new();
            write_receipt(original.clone(), TOKEN, &mut bytes).unwrap();
            assert!(bytes.len() <= MAX_TOTAL);
            let lines = bytes.split_inclusive(|b| *b == b'\n').collect::<Vec<_>>();
            let mut digest = Sha256::new();
            let mut rows = Vec::new();
            for (i, line) in lines.iter().enumerate() {
                assert!(line.len() < MAX_FRAME);
                let frame: Value = serde_json::from_slice(line).unwrap();
                assert_eq!(frame["protocol"], PROTOCOL);
                assert_eq!(frame["request"], TOKEN);
                if i + 1 == lines.len() {
                    assert_eq!(frame["kind"], "end");
                    assert_eq!(frame["batch_count"], i - 1);
                    assert_eq!(frame["item_count"], count);
                    assert_eq!(frame["sha256"], format!("{:x}", digest.finalize()));
                    break;
                }
                digest.update(line);
                if i == 0 {
                    assert_eq!(frame["kind"], "begin");
                    assert_eq!(frame["item_count"], count);
                    assert!(frame["receipt"]["artwork"].get("items").is_none());
                } else {
                    assert_eq!(frame["kind"], "batch");
                    assert_eq!(frame["sequence"], i - 1);
                    rows.extend(frame["items"].as_array().unwrap().iter().cloned());
                }
            }
            assert_eq!(&rows, original["artwork"]["items"].as_array().unwrap());
        }
    }

    #[test]
    fn limits_produce_no_partial_artwork() {
        for original in [
            receipt(MAX_ITEMS + 1),
            json!({"action":"open","artwork_status":"RENDERABLE",
                "artwork":{"items":[{"id":"x".repeat(MAX_FRAME)}]}}),
        ] {
            let mut bytes = Vec::new();
            write_receipt(original, TOKEN, &mut bytes).unwrap();
            let frames = bytes
                .split(|b| *b == b'\n')
                .filter(|line| !line.is_empty())
                .map(|line| serde_json::from_slice::<Value>(line).unwrap())
                .collect::<Vec<_>>();
            assert_eq!(frames.len(), 2);
            assert_eq!(frames[0]["receipt"]["artwork_status"], "NOT_RENDERED");
            assert!(frames[0]["receipt"].get("artwork").is_none());
            assert_eq!(frames[1]["item_count"], 0);
            assert_eq!(frames[1]["batch_count"], 0);
        }
    }
}
