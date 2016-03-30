
const BLOCK_BITS : u32 = 12; // 4096 bytes
const BLOCK_SIZE : u32 = 1 << BLOCK_BITS;
const BLOCK_MASK : u32 = BLOCK_SIZE - 1;

const INDEX_BITS : u32 = 10;
const INDEX_SIZE : u32 = 1 << INDEX_BITS;
const INDEX_MASK : u32 = INDEX_SIZE - 1;

// ammount of space to use in the index
// for single-level block pointers
static ONE_LEVEL  : u32 = (INDEX_SIZE >> 1);

// logical page number and offset
macro_rules! block {
    ($x) => ($x >> BLOCK_BITS);
}
macro_rules! offset {
    ($x) => ($x & BLOCK_MASK);
}

// block pointer to indexes into blob header / page block
// for multi level page tables

//static INDEX0 : u32 = lambda x:ONE_LEVEL + (((x - ONE_LEVEL) >> INDEX_BITS) & INDEX_MASK)
macro_rules! index0 {
    ($x) => (ONE_LEVEL + ((($x - ONE_LEVEL) >> INDEX_BITS) & INDEX_MASK));
}

//static INDEX1 : u32 = lambda x:((x - ONE_LEVEL)               & INDEX_MASK)
macro_rules! index1 {
    ($x) => (($x - ONE_LEVEL) & INDEX_MASK);
}

const ZERO_BLOCK : [u8 ; BLOCK_SIZE as usize] = [0; BLOCK_SIZE as usize];

const SMALL_PAGE : u32   = 1;
const REGULAR_PAGE : u32 = 2;

struct BucketHeader {
    key:     u16,
    val:     u16,
    key_len: u16,
    val_len: u16,
}

struct Bucket {
    header: BucketHeader,
    data: [u8],
}

struct Entry {
    offset: u16,
    len:    u16,
}
