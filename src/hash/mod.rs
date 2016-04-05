//! Let's talk about databases.
//! This database is made of virtualized blocks. Here in this rust implementation
//! those annoying parts are avoided and we simply leverage a single block for the
//! entire database.
//!
//! Our block looks like this:
//! ```
//! [{len: u16, off: u16}][msgpack_maps][{count: u64, split: u32, level: u8} (offset 4096-13)]
//! ```
//!
//! The first section is a list headers that point to msgpack_maps
//! The second section is the msgpack_maps themselves (because we're too lazy to do quadratic hashing)
//! The third section is the hash table information (footer)

#[macro_use]
mod entry;

use std::collections::HashMap;

use self::entry::SubBucketer;

const HASH_FOOTER_SIZE : u32 = 64 + 32 + 8; // sizeof(Hash)

/// Hash is a linear hash table
struct Hash {
    count: u64,
    split: u32,
    level: u8,

    blocks: Vec<SubBucketer>,
}

const SUB_BUCKET_BITS : u32 = 8;
const SUB_BUCKET      : u32 = (1 << SUB_BUCKET_BITS); //(256)
const SUB_BUCKET_MASK : u32 = (SUB_BUCKET - 1);       //(255 (1111 1111))

type MaybeHash = Option<HashMap<i32,i32>>;

impl Hash {
    fn new() -> Hash {
        Hash {
            count: 1,
            split: 0,
            level: 1,
            blocks: vec![SubBucketer::new()],
        }
    }

    fn get_bucket(&self, key : String) -> (u8, MaybeHash) {
        let hash : u32 = key.as_bytes().iter().fold(0, |a, b| a + (*b as u32)); // "hash" the string LOL

        let level = 1 << self.level;
        // h & (256)^(level+1) - 1  -- truncate the hash value to bits under some power of 2 based on level
        let mut bucket = hash & ((SUB_BUCKET << 1 << level) - 1);

        // (2^level + self.split)^8 -- check to ensure that the bucket fits within the
        // parameters of level and split if not we'll truncate it again
        if bucket >= ((1 << level) + self.split) << SUB_BUCKET_BITS {
            // truncate the hash value to bits under one less power of 2 of the level than before
            bucket = hash & ((SUB_BUCKET << level) - 1);
        }

        // Truncate again to ensure we fall within our sub bucket indexing
        let sub_bucket_index = (bucket & SUB_BUCKET_MASK) as u8;
        // Divide by 2^8 - this is effectively a % operation to ensure we select the right
        // block, but the modding is mostly done above. This succeeds because count = level + split
        let block = &self.blocks[(bucket >> SUB_BUCKET_BITS) as usize];

        match block.get_sub_bucket(sub_bucket_index) {
            Ok(x)  => return (sub_bucket_index, x),
            Err(x) => panic!("how can there be no sub_bucket here?".to_string() + &x.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn new() {
        let h = super::Hash::new();

        assert_eq!(h.count, 1);
        assert_eq!(h.split, 0);
        assert_eq!(h.level, 1);
        assert_eq!(h.blocks.len(), 1);
    }

    #[test]
    fn get_bucket() {
        let h = super::Hash::new();
    }
}
