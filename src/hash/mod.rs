mod math;
mod hash_block;

/// Hash is a linear hash table
struct Hash {
    size: u64,
    split: u32,
    level: u8,
}

impl Hash {
    fn new() -> Hash {
        Hash { size: 0, split: 0, level: 0 }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn new() {
        let h = super::Hash::new();

        assert!(h.size == 0);
        assert!(h.split == 0);
        assert!(h.level == 0);
    }
}
