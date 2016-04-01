use std::collections::HashMap;

use msgpack_coders;
use rustc_serialize::Decodable;
use rustc_serialize::Encodable;

/// Block for Hash
struct Block {
    data: [u8 ; 4096],
}

impl Block {
    fn new() -> Block {
        Block {
            data: [0 ; 4096],
        }
    }

    fn get_sub(&self, index : i32) -> Result<&mut HashMap<i32,i32>, String> {
        Err("hello world".to_string())
    }

    fn set_sub(&mut self, index : i32, map : &HashMap<i32,i32>) -> Option<String> {
        let mut data = &mut &mut self.data[..];
        let mut encoder = msgpack_coders::Encoder::new(data);

        if let Some(e) = map.encode(&mut encoder).err() {
            return Some(format!("error writing map: {}", e))
        }
        None
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    #[test]
    fn new() {
        let b = super::Block::new();

        assert!(b.data.len() == 4096);
        assert!(b.data[0] == 0);
    }

    #[test]
    fn set_sub() {
        //let mut b = super::Block::new();

        //let mut h = HashMap<i32,i32>::new();
        //h.set(5, 6);
        //h.set(7, 8);

        //b.set_sub(0, h).unwrap();

        //let mut h2 = b.get_sub(0).unwrap();
    }

    #[test]
    fn get_sub() {
        let mut b = super::Block::new();

        let mut h = HashMap::new();
        h.insert(5, 6);
        h.insert(7, 8);

        b.set_sub(0, &mut h).unwrap();

        assert!(b.data[0] != 0)
    }
}
