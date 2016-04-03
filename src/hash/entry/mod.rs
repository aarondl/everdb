mod msgpack_helpers;
mod block_iterator;
mod helpers;

use std::collections::HashMap;

use byteorder::{BigEndian, ByteOrder};

use self::helpers::*;
use super::block::Block;

const ENTRY_SIZE             : u32 = 4;
const SUB_BUCKET_HEADER_SIZE : u32 = 1024;
 // header size 1024 bytes / entry size 4 bytes
const MAX_SUB_BUCKETS        : u32 = SUB_BUCKET_HEADER_SIZE / ENTRY_SIZE;
const MAX_DATA_SIZE          : u32 = 4096 - SUB_BUCKET_HEADER_SIZE - super::HASH_FOOTER_SIZE - super::block::BLOCK_FOOTER_SIZE;

static CANT_GROW_ERR : &'static str = "We can't grow in this implementation so indexes past 256 are untenable";

#[derive(Copy, Clone)]
pub struct Entry {
    offset: u16,
    size: u16,
}

pub fn put_sub_bucket(block : &mut Block, index : u32, map : &HashMap<i32,i32>) -> Result<(), String> {
    if index >= MAX_SUB_BUCKETS {
        panic!(CANT_GROW_ERR)
    }

    let buf : Vec<u8>;
    match msgpack_helpers::encode(map) {
        Ok(v)  => buf = v,
        Err(e) => return Err(e),
    }

    let size       = buf.len() as u16;
    let mut entry  = get_entry(&block, index);

    // Hard case - it doesn't fit, allocate new
    if size > entry.size {
        match find_space(&block, size) {
            Some(off) => entry.offset = off,
            None      => return Err("Ran out of disk space I guess".to_string()),
        }
    }

    entry.size = size;
    put_entry(block, index, entry);
    for (i, b) in buf.iter().enumerate() {
        block[i+(entry.offset as usize)] = *b;
    }

    Ok(())
}

pub fn del_sub_bucket(block : &mut Block, index : u32) {
    if index >= MAX_SUB_BUCKETS {
        panic!(CANT_GROW_ERR)
    }

    put_entry(block, index, Entry { offset: 0, size: 0 });
}

pub fn get_sub_bucket(block : &Block, index : u32) -> Result<Option<HashMap<i32,i32>>, String> {
    if index >= MAX_SUB_BUCKETS {
        panic!(CANT_GROW_ERR)
    }

    let entry = get_entry(block, index);
    if entry.size == 0 {
        return Ok(None)
    }

    let offset = entry.offset as usize;
    let size = entry.size as usize;

    match msgpack_helpers::decode(&block[offset..offset+size]) {
        Ok(v)  => Ok(Some(v)),
        Err(e) => return Err(e),
    }
}

/// Put an entry to some index, panics on bad index
fn put_entry(block : &mut Block, index : u32, e : Entry) {
    if index >= MAX_SUB_BUCKETS {
        panic!(CANT_GROW_ERR)
    }

    let i = (index * 4) as usize;
    let j = i + 2;

    BigEndian::write_u16(&mut block[i..i+2], e.offset);
    BigEndian::write_u16(&mut block[j..j+2], e.size);
}

/// Get an entry from an index, panics on bad index
fn get_entry(block : &Block, index : u32) -> Entry {
    if index >= MAX_SUB_BUCKETS {
        panic!(CANT_GROW_ERR)
    }

    let i = (index * 4) as usize;
    let j = i + 2;

    Entry {
        offset: BigEndian::read_u16(&block[i..i+2]),
        size: BigEndian::read_u16(&block[j..j+2]),
    }
}

/// Find a offset in memory for our data length, or None
fn find_space(block : &Block, want_size : u16) -> Option<u16> {
    let mut next_offset = SUB_BUCKET_HEADER_SIZE as u16;
    let size = next_power_of_2(want_size);

    // Sort all the entries such that they're ordered by offset
    let mut entries : Vec<Entry> = block_iterator::BlockIterator::new(&block).collect();
    entries.sort();

    for e in entries {
        if e.size == 0 {
            continue;
        }

        if next_offset + size <= e.offset {
            break;
        }

        next_offset = e.offset + next_power_of_2(e.size);
    }

    if next_offset + size > MAX_DATA_SIZE as u16 {
        return None;
    }

    Some(next_offset)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::block::*;

    #[test]
    fn get_put_sub_bucket() {
        let mut b : Block = new_block!();

        let mut h = HashMap::new();
        h.insert(5, 6);
        h.insert(7, 8);

        super::put_sub_bucket(&mut b, 1, &h).unwrap();

        let h2 = super::get_sub_bucket(&b, 1).unwrap().unwrap();
        assert_eq!(h, h2);

        let h3 = super::get_sub_bucket(&b, 2).unwrap();
        assert!(h3.is_none());
    }

    #[test]
    fn del_sub_bucket() {
        let mut b : Block = new_block!();

        let mut h = HashMap::new();
        h.insert(5, 6);
        h.insert(7, 8);

        super::put_sub_bucket(&mut b, 0, &mut h).unwrap();
        super::del_sub_bucket(&mut b, 0);
        let h = super::get_sub_bucket(&mut b, 0).unwrap();
        assert!(h.is_none());
    }

    #[test]
    fn get_entry() {
        let mut b : Block = new_block!();

        // write an entry at b[40] of size: 257, offset: 258
        b[40] = 1;
        b[41] = 1;
        b[42] = 1;
        b[43] = 2;

        let entry = super::get_entry(&b, 10);

        assert_eq!(entry.offset, 257);
        assert_eq!(entry.size, 258);
    }

    #[test]
    fn put_entry() {
        let mut b : Block = new_block!();
        let e = super::Entry { size: 257, offset: 258 };

        super::put_entry(&mut b, 10, e);

        let entry = super::get_entry(&b, 10);

        assert_eq!(entry.size, 257);
        assert_eq!(entry.offset, 258);
    }

    #[test]
    fn find_space() {
        let mut b : Block = new_block!();

        // If the entire space is taken error
        super::put_entry(&mut b, 0, super::Entry { offset: 0, size: super::MAX_DATA_SIZE as u16 });
        assert_eq!(super::find_space(&b, 9), None);
    }

    #[test]
    fn find_space_end() {
        let mut b : Block = new_block!();

        // If there's space at the end
        super::put_entry(&mut b, 0, super::Entry { offset: 0, size: 9 });
        super::put_entry(&mut b, 2, super::Entry { offset: 16, size: 25 });

        assert_eq!(super::find_space(&b, 28).unwrap(), 48);
    }

    #[test]
    fn find_space_middle() {
        let mut b : Block = new_block!();

        // If there's a chunk in between somewhere (out of order)
        super::put_entry(&mut b, 0, super::Entry { offset: 0, size: 9 });
        super::put_entry(&mut b, 1, super::Entry { offset: 64, size: 26 });
        super::put_entry(&mut b, 2, super::Entry { offset: 16, size: 25 });

        assert_eq!(super::find_space(&b, 9).unwrap(), 48);
        assert_eq!(super::find_space(&b, 28).unwrap(), 96);
    }
}
