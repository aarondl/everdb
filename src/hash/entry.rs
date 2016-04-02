use std::io::Write;
use std::io::Read;
use std::collections::HashMap;

use byteorder::{BigEndian, ByteOrder};
use msgpack_coders::Encoder;
use rustc_serialize::Decodable;
use rustc_serialize::Encodable;

use super::block::*;

const ENTRY_SIZE : u32             = 32;
const SUB_BUCKET_HEADER_SIZE : u32 = 1024;
 // header size 1024 bytes / entry size 4 bytes
const MAX_SUB_BUCKETS : u32        = SUB_BUCKET_HEADER_SIZE / ENTRY_SIZE;

pub struct Entry {
    len: u16,
    offset: u16,
}

pub fn put_sub_bucket(block : &mut Block, index : i32, map : &HashMap<i32,i32>) -> Result<(), String> {
    let data = &mut &mut block[..];
    let mut encoder = Encoder::new(data);

    if let Some(e) = map.encode(&mut encoder).err() {
        return Err(format!("error writing map: {}", e))
    }
    Ok(())
}

pub fn get_sub_bucket(block : &mut Block, index : i32) -> Result<&mut HashMap<i32,i32>, String> {
    Err("sorry guys".to_string())
}

fn find_next_sub_bucket(block : &Block) -> Option<u32> {
    for i in 0..MAX_SUB_BUCKETS {
        let off = (i * ENTRY_SIZE) as usize;
        let mut entry = Entry { len: 0, offset: 0 };

        entry.len = BigEndian::read_u16(&block[off..(off+2)]);
        entry.offset = BigEndian::read_u16(&block[off..(off+2)]);

        if entry.len == 0 {
            return Some(i)
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::block::*;

    #[test]
    fn get_sub_bucket() {
        //let mut b = super::Block::new();
        //
        //let mut h : HashMap<i32,i32> = HashMap::new();
        //h.insert(5, 6);
        //h.insert(7, 8);
        //
        //b.set_sub(0, &mut h).unwrap();
        //let h2 = b.get_sub(0).unwrap();
        //
        //assert_eq!(h.get(5).unwrap(), 6);
        //assert_eq!(h.get(7).unwrap(), 8);
    }

    #[test]
    fn put_sub_bucket() {
        let mut b : Block = new_block!();

        let mut h = HashMap::new();
        h.insert(5, 6);
        h.insert(7, 8);

        super::put_sub_bucket(&mut b, 0, &mut h).unwrap();

        assert!(b[0] != 0);
    }

    #[test]
    fn find_next_sub_bucket() {
        let mut b : Block = new_block!();

        let index1 = super::find_next_sub_bucket(&b).unwrap();
        assert_eq!(index1, 0);

        b[0] = 3; // Set the "len" of the first Entry to != 0
        b[1] = 3;

        let index2 = super::find_next_sub_bucket(&b).unwrap();
        assert_eq!(index2, 1);
    }
}
