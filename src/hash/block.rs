pub const BLOCK_FOOTER_SIZE : u32 = 16; // sizeof(u32)

pub type Block = [u8 ; 4096];

macro_rules! new_block {
    () => ([0 ; 4096]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_block() {
        let b : Block = new_block!();

        assert!(b.len() == 4096);
        assert!(b[0] == 0);
    }
}
