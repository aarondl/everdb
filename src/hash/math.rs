const BLOCK_BITS : u32 = 12; // 4096 bytes
const BLOCK_SIZE : u32 = 1 << BLOCK_BITS;
const BLOCK_MASK : u32 = BLOCK_SIZE - 1;

const INDEX_BITS : u32 = 10;
const INDEX_SIZE : u32 = 1 << INDEX_BITS;
const INDEX_MASK : u32 = INDEX_SIZE - 1;

const ZERO_BLOCK : [u8; BLOCK_SIZE as usize] = [0; BLOCK_SIZE as usize];

const SMALL_PAGE   : u32 = 1;
const REGULAR_PAGE : u32 = 2;

// ammount of space to use in the index
// for single-level block pointers
const ONE_LEVEL  : u32 = (INDEX_SIZE >> 1);

// logical page number and offset
macro_rules! block {
    ($x) => ($x >> BLOCK_BITS);
}
macro_rules! offset {
    ($x) => ($x & BLOCK_MASK);
}

// block pointer to indexes into blob header / page block
// for multi level page tables
macro_rules! index0 {
    ($x) => (ONE_LEVEL + ((($x - ONE_LEVEL) >> INDEX_BITS) & INDEX_MASK));
}
macro_rules! index1 {
    ($x) => (($x - ONE_LEVEL) & INDEX_MASK);
}

#[cfg(test)]
mod tests {
    fn next_block_helper(x : u32) -> u32 {
        let next_block0 = (x + super::INDEX_SIZE) & (!super::INDEX_MASK);
        return next_block0;
    }

    #[test]
    fn next_block() {
        assert!(next_block_helper(0) == 1024);
        assert!(next_block_helper(1) == 1024);
        assert!(next_block_helper(1023) == 1024);
        assert!(next_block_helper(1024) == 2048);
        assert!(next_block_helper(1025) == 2048);
    }
}
