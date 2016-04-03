pub const BLOCK_FOOTER_SIZE : u32 = 16; // sizeof(u32)
pub const EMPTY_BLOCK : Block = [0 ; 4096];

pub type Block = [u8 ; 4096];

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
