/// Bit twiddling magic function.
///
/// Essentially continuously or's it with itself
/// moving bits down the row and once all the bits
/// are filled in it adds one to turn on the next
/// power of two bit.
pub fn next_power_of_2(from : u16) -> u16 {
    let mut n = from - 1;
    n |= n >> 1;
    n |= n >> 2;
    n |= n >> 4;
    n |= n >> 8;
    //n |= n >> 16; // Not needed since I've shrank the size of the integer
    n + 1
}

#[cfg(test)]
mod tests {
    #[test]
    fn next_power_of_2() {
        assert_eq!(super::next_power_of_2(1), 1);
        assert_eq!(super::next_power_of_2(3), 4);
        assert_eq!(super::next_power_of_2(5), 8);
        assert_eq!(super::next_power_of_2(6), 8);
        assert_eq!(super::next_power_of_2(11), 16);
        assert_eq!(super::next_power_of_2(17), 32);
    }
}
