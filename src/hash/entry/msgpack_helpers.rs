use std::collections::HashMap;
use std::io::Cursor;

use msgpack_coders::{Encoder,Decoder};
use rustc_serialize::Decodable;
use rustc_serialize::Encodable;

use super::StrHash;

/// Return a buffer with the serialized hash map in it.
pub fn encode(map : &StrHash) -> Result<Vec<u8>, String> {
    let mut buf: Vec<u8> = Vec::new();

    {
        let mut encoder = Encoder::new(&mut buf);
        if let Some(e) = map.encode(&mut encoder).err() {
            return Err(format!("error writing map: {}", e))
        }
    }

    Ok(buf)
}

/// Decode a serialized hash map from a slice
pub fn decode(slice : &[u8]) -> Result<StrHash, String> {
    let c = Cursor::new(slice);

    let mut decoder = Decoder::new(c);
    match Decodable::decode(&mut decoder) {
        Ok(h)  => return Ok(h),
        Err(e) => Err(format!("error reading map: {}", e)),
    }
}
