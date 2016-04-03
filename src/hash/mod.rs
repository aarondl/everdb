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

const HASH_FOOTER_SIZE : u32 = 64 + 32 + 8; // sizeof(Hash)

/*
/// Hash is a linear hash table
struct Hash {
    count: u64,
    split: u32,
    level: u8,
}

impl Hash {
    fn new() -> Hash {
        Hash { count: 0, split: 0, level: 0 }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn new() {
        let h = super::Hash::new();

        assert!(h.count == 0);
        assert!(h.split == 0);
        assert!(h.level == 0);
    }
}
*/
