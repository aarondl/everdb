use std::cmp::Ordering;

use super::Entry;
use super::SubBucketer;

pub struct BlockIterator<'a> {
    i: u16,
    block: &'a SubBucketer,
}

impl<'a> BlockIterator<'a> {
    pub fn new(b : &SubBucketer) -> BlockIterator {
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

        let entry = self.block.get_entry(self.i as u8);
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
mod test {
    use super::BlockIterator;
    use super::super::MAX_SUB_BUCKETS;
    use super::super::super::block::{EMPTY_BLOCK};
    use super::super::{Entry,SubBucketer};

    #[test]
    fn iterators() {
        let b = SubBucketer(EMPTY_BLOCK);
        let mut bi = BlockIterator::new(&b);

        let mut count = 0;
        for _ in bi {
            count += 1;
        }

        bi = BlockIterator::new(&b);
        bi.collect::<Vec<Entry>>().sort();

        assert_eq!(count, MAX_SUB_BUCKETS);
    }

    #[test]
    fn sorting() {
        let mut b = SubBucketer(EMPTY_BLOCK);

        b.put_entry(0, Entry { offset: 196, size: 16 });
        b.put_entry(1, Entry { offset: 16, size: 32 });
        b.put_entry(2, Entry { offset: 36, size: 32 });

        let mut entries : Vec<Entry> = BlockIterator::new(&b).collect();
        entries.sort();

        let offsets : Vec<u16> = entries.iter().map(|i| i.offset).collect();

        // There's going to be a lot of zeroes in here, cull them out
        assert_eq!(offsets[offsets.len()-3..], [16, 36, 196]);
    }
}
