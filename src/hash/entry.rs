use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;
use std::io::Read;

use byteorder::{BigEndian, ByteOrder};
use msgpack_coders::Encoder;
use rustc_serialize::Decodable;
use rustc_serialize::Encodable;

use super::block::Block;

const ENTRY_SIZE             : u32 = 4;
const SUB_BUCKET_HEADER_SIZE : u32 = 1024;
 // header size 1024 bytes / entry size 4 bytes
const MAX_SUB_BUCKETS        : u32 = SUB_BUCKET_HEADER_SIZE / ENTRY_SIZE;
const MAX_DATA_SIZE          : u32 = 4096 - super::HASH_FOOTER_SIZE - super::block::BLOCK_FOOTER_SIZE;

static CANT_GROW_ERR : &'static str = "We can't grow in this implementation so indexes past 256 are untenable";

pub struct Entry {
    offset: u16,
    len: u16,
}

pub fn put_sub_bucket(block : &mut Block, index : u32, map : &HashMap<i32,i32>) -> Result<(), String> {
    if index >= MAX_SUB_BUCKETS {
        panic!(CANT_GROW_ERR)
    }

    let mut buf: Vec<u8> = Vec::new();

    {
        let mut encoder = Encoder::new(&mut buf);
        if let Some(e) = map.encode(&mut encoder).err() {
            return Err(format!("error writing map: {}", e))
        }
    }

    let e = Entry { len: buf.len() as u16, offset: 0 };

    Ok(())
}

pub fn get_sub_bucket(block : &mut Block, index : i32) -> Result<&mut HashMap<i32,i32>, String> {
    Err("sorry guys".to_string())
}

/// Put an entry to some index, panics on bad index
fn put_entry(block : &mut Block, index : u32, e : Entry) {
    if index >= MAX_SUB_BUCKETS {
        panic!(CANT_GROW_ERR)
    }

    let i = (index * 4) as usize;
    let j = i + 2;

    BigEndian::write_u16(&mut block[i..i+2], e.offset);
    BigEndian::write_u16(&mut block[j..j+2], e.len);
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
        len: BigEndian::read_u16(&block[j..j+2]),
    }
}

/// Find a offset in memory for our data length, or None
fn find_space(block : &Block, want_size : u32) -> Option<u32> {
    let mut next_offset = SUB_BUCKET_HEADER_SIZE;
    let size = next_power_of_2(want_size);

    for i in 0..MAX_SUB_BUCKETS-1 {
        let entry = get_entry(&block, i);

        //next_offset = std::cmp::max(entry.offset + entry.len, next_offset);
    }

    None
}

/// Bit twiddling magic function.
///
/// Essentially continuously or's it with itself
/// moving bits down the row and once all the bits
/// are filled in it adds one to turn on the next
/// power of two bit.
fn next_power_of_2(mut n : u32) -> u32 {
    n -= 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    n |= n >> 16;
    n + 1
}

struct BlockIterator<'a> {
    i: u32,
    block: &'a [ u8; 4096 ],
}

impl<'a> BlockIterator<'a> {
    fn new(b : &Block) -> BlockIterator {
        BlockIterator {
            i: 0,
            block: b,
        }
    }
}

impl<'a> Iterator for BlockIterator<'a> {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        if self.i == 256 {
            return None
        }

        let entry = get_entry(self.block, self.i);
        self.i += 1;

        return Some(entry);
    }
}


// This chunk of code is pretty ridiculous. Haskell beats you here pretty hard Rust.
impl PartialOrd for Entry {
    fn partial_cmp(&self, other : &Entry) -> Option<Ordering> { self.offset.partial_cmp(&other.offset) }
}
impl PartialEq for Entry {
    fn eq(&self, other : &Entry) -> bool { self.offset.eq(&other.offset) }
}
impl Ord for Entry {
    fn cmp(&self, other : &Entry) -> Ordering { self.offset.cmp(&other.offset) }
}
impl Eq for Entry {}


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

        //assert!(b[0] != 0);
    }

    #[test]
    fn get_entry() {
        let mut b : Block = new_block!();

        // write an entry at b[40] of len: 257, offset: 258
        b[40] = 1;
        b[41] = 1;
        b[42] = 1;
        b[43] = 2;

        let entry = super::get_entry(&b, 10);

        assert_eq!(entry.offset, 257);
        assert_eq!(entry.len, 258);
    }

    #[test]
    fn put_entry() {
        let mut b : Block = new_block!();
        let e = super::Entry { len: 257, offset: 258 };

        super::put_entry(&mut b, 10, e);

        let entry = super::get_entry(&b, 10);

        assert_eq!(entry.len, 257);
        assert_eq!(entry.offset, 258);
    }

    #[test]
    fn find_space() {
        let mut b : Block = new_block!();

        super::put_entry(&mut b, 0, super::Entry { offset: 0, len: 16 });
        super::put_entry(&mut b, 1, super::Entry { offset: 16, len: 32 });
        super::put_entry(&mut b, 2, super::Entry { offset: 64, len: 32 });

        //let offset = super::find_space(&b, 30).unwrap();
        //assert_eq!(offset, 32);
    }

    #[test]
    fn next_power_of_2() {
        assert_eq!(super::next_power_of_2(1), 1);
        assert_eq!(super::next_power_of_2(3), 4);
        assert_eq!(super::next_power_of_2(5), 8);
        assert_eq!(super::next_power_of_2(6), 8);
        assert_eq!(super::next_power_of_2(11), 16);
        assert_eq!(super::next_power_of_2(17), 32);
    }

    #[test]
    fn iterators() {
        let b : Block = new_block!();
        let mut bi = super::BlockIterator::new(&b);

        let mut count = 0;
        for e in bi {
            count += 1;
        }

        bi = super::BlockIterator::new(&b);
        bi.collect::<Vec<super::Entry>>().sort();

        assert_eq!(count, super::MAX_SUB_BUCKETS);
    }

    #[test]
    fn sorting() {
        let mut b : Block = new_block!();

        super::put_entry(&mut b, 0, super::Entry { offset: 196, len: 16 });
        super::put_entry(&mut b, 1, super::Entry { offset: 16, len: 32 });
        super::put_entry(&mut b, 2, super::Entry { offset: 36, len: 32 });

        let mut entries : Vec<super::Entry> = super::BlockIterator::new(&b).collect();
        entries.sort();

        let offsets : Vec<u16> = entries.iter().map(|i| i.offset).collect();

        // There's going to be a lot of zeroes in here, cull them out
        assert_eq!(offsets[offsets.len()-3..], [16, 36, 196]);
    }
}
