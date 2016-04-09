pub const BLOCK_FOOTER_SIZE : u32 = 16; // sizeof(u16)
pub const BLOCK_SIZE        : u32 = 4096;
pub const EMPTY_BLOCK       : Block = [0 ; BLOCK_SIZE as usize];

pub type Block = [u8 ; BLOCK_SIZE as usize];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_block() {
        let b : Block = EMPTY_BLOCK;

        assert!(b.len() == 4096);
        assert!(b[0] == 0);
    }
}
