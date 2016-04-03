mod msgpack_helpers;
mod block_iterator;
mod helpers;

use std::collections::HashMap;

use byteorder::{BigEndian, ByteOrder};

use self::helpers::*;
use super::block::Block;

const ENTRY_SIZE             : u32 = 4;
const SUB_BUCKET_HEADER_SIZE : u32 = 1024;
const MAX_SUB_BUCKETS        : u32 = SUB_BUCKET_HEADER_SIZE / ENTRY_SIZE;
const MAX_DATA_SIZE          : u32 = 4096 - SUB_BUCKET_HEADER_SIZE - super::HASH_FOOTER_SIZE - super::block::BLOCK_FOOTER_SIZE;

#[derive(Copy, Clone, Debug)]
pub struct Entry {
    offset: u16,
    size: u16,
}

/// SubBucketer is a Block enhancement that adds sub bucket operations.
pub struct SubBucketer(Block);

/// Implement both Deref and DerefMut in order to be able to get back our Block because
/// if we don't we lose all of the indexing capabilities etc due to newtyping.
impl ::std::ops::Deref for SubBucketer {
    type Target = Block;
    fn deref(&self) -> &Block {
        &self.0
    }
}

impl ::std::ops::DerefMut for SubBucketer {
    fn deref_mut<'a>(&'a mut self) -> &'a mut Block {
        &mut self.0
    }
}

impl SubBucketer {
    /// Put a sub bucket into the block. Use msgpack to serialize the hashmap.
    pub fn put_sub_bucket(&mut self, index : u8, map : &HashMap<i32,i32>) -> Result<(), String> {
        let buf : Vec<u8> = try!(msgpack_helpers::encode(map));
        let size       = buf.len() as u16;
        let mut entry  = self.get_entry(index);

        // Hard case - it doesn't fit, allocate new
        if size > entry.size {
            match self.find_space(size) {
                Some(off) => entry.offset = off,
                None      => return Err("Ran out of disk space I guess".to_string()),
            }
        }

        entry.size = size;
        self.put_entry(index, entry);
        for (i, b) in buf.iter().enumerate() {
            self[i+(entry.offset as usize)] = *b;
        }

        Ok(())
    }

    /// Delete a sub bucket from the block, essentially just zeroes out the
    /// entry so that it appears unused. The data is still present.
    pub fn del_sub_bucket(&mut self, index : u8) {
        self.put_entry(index, Entry { offset: 0, size: 0 });
    }

    /// Get a sub bucket from the block, the result is gross and can return
    /// an Option hash map. In case the size of the block we're trying to get is 0
    /// which indicates a sub bucket that doesn't exist.
    pub fn get_sub_bucket(&self, index : u8) -> Result<Option<HashMap<i32,i32>>, String> {
        let entry = self.get_entry(index);
        if entry.size == 0 {
            return Ok(None)
        }

        let offset = entry.offset as usize;
        let size = entry.size as usize;

        match msgpack_helpers::decode(&self[offset..offset+size]) {
            Ok(v)  => Ok(Some(v)),
            Err(e) => return Err(e),
        }
    }

    /// Put an entry to some index, panics on bad index
    fn put_entry(&mut self, index : u8, e : Entry) {
        let i = (index as usize) * 4;
        let j = i + 2;

        BigEndian::write_u16(&mut self[i..i+2], e.offset);
        BigEndian::write_u16(&mut self[j..j+2], e.size);
    }

    /// Get an entry from an index, panics on bad index
    fn get_entry(&self, index : u8) -> Entry {
        let i = (index as usize) * 4;
        let j = i + 2;

        Entry {
            offset: BigEndian::read_u16(&self[i..i+2]),
            size: BigEndian::read_u16(&self[j..j+2]),
        }
    }

    /// Find a offset in memory for our data length, or None
    fn find_space(&self, want_size : u16) -> Option<u16> {
        let mut next_offset = SUB_BUCKET_HEADER_SIZE as u16;
        let size = next_power_of_2(want_size);

        // Sort all the entries such that they're ordered by offset
        let mut entries : Vec<Entry> = block_iterator::BlockIterator::new(self).collect();
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::block::*;
    use super::SubBucketer;

    #[test]
    fn get_put_sub_bucket() {
        let mut b = SubBucketer(EMPTY_BLOCK);

        let mut h = HashMap::new();
        h.insert(5, 6);
        h.insert(7, 8);

        b.put_sub_bucket(1, &h).unwrap();

        let h2 = b.get_sub_bucket(1).unwrap().unwrap();
        assert_eq!(h, h2);

        let h3 = b.get_sub_bucket(2).unwrap();
        assert!(h3.is_none());
    }

    #[test]
    fn del_sub_bucket() {
        let mut b = SubBucketer(EMPTY_BLOCK);

        let mut h = HashMap::new();
        h.insert(5, 6);
        h.insert(7, 8);

        b.put_sub_bucket(0, &mut h).unwrap();
        b.del_sub_bucket(0);
        let h = b.get_sub_bucket(0).unwrap();
        assert!(h.is_none());
    }

    #[test]
    fn get_entry() {
        let mut b = SubBucketer(EMPTY_BLOCK);

        // write an entry at b[40] of size: 257, offset: 258
        b[40] = 1;
        b[41] = 1;
        b[42] = 1;
        b[43] = 2;

        let entry = b.get_entry(10);

        assert_eq!(entry.offset, 257);
        assert_eq!(entry.size, 258);
    }

    #[test]
    fn put_entry() {
        let mut b = SubBucketer(EMPTY_BLOCK);
        let e = super::Entry { size: 257, offset: 258 };

        b.put_entry(10, e);

        let entry = b.get_entry(10);

        assert_eq!(entry.size, 257);
        assert_eq!(entry.offset, 258);
    }

    #[test]
    fn find_space() {
        let mut b = SubBucketer(EMPTY_BLOCK);

        // If the entire space is taken error
        b.put_entry(0, super::Entry { offset: 0, size: super::MAX_DATA_SIZE as u16 });
        assert_eq!(b.find_space(9), None);
    }

    #[test]
    fn find_space_end() {
        let mut b = SubBucketer(EMPTY_BLOCK);

        // If there's space at the end
        b.put_entry(0, super::Entry { offset: 0, size: 9 });
        b.put_entry(2, super::Entry { offset: 16, size: 25 });

        assert_eq!(b.find_space(28).unwrap(), 48);
    }

    #[test]
    fn find_space_middle() {
        let mut b = SubBucketer(EMPTY_BLOCK);

        // If there's a chunk in between somewhere (out of order)
        b.put_entry(0, super::Entry { offset: 0, size: 9 });
        b.put_entry(1, super::Entry { offset: 64, size: 26 });
        b.put_entry(2, super::Entry { offset: 16, size: 25 });

        assert_eq!(b.find_space(9).unwrap(), 48);
        assert_eq!(b.find_space(28).unwrap(), 96);
    }
}
