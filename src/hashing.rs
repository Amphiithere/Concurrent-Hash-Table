use std::num::Wrapping;

/// Jenkins 'one_at_a_time' hashing function \
/// https://en.wikipedia.org/wiki/Jenkins_hash_function
///
/// # Arguments
/// * `string` - The string to be hashed
///
/// # Returns
/// * The unsigned 32-bit integer hash code
///
/// # Examples
/// ```
/// let hash = hashing::one_at_a_time("The quick brown fox jumps over the lazy dog");
/// assert_eq!(hash, 0x519e91f5 /* hash of above input */);
/// ```
pub fn one_at_a_time<T: Into<String>>(string: T) -> u32
{
    // Permits integer overflow in computations
    let mut hash = Wrapping(0);

    for &byte in string.into().as_bytes()
    {
        hash += byte as u32;
        hash += hash << 10;
        hash ^= hash >> 6;
    }
    hash += hash << 3;
    hash ^= hash >> 11;
    hash += hash << 15;

    return hash.0;
}

pub fn assert() {
    let hash = one_at_a_time("The quick brown fox jumps over the lazy dog");
    assert_eq!(hash, 0x519e91f5);
    let hash = one_at_a_time("a");
    assert_eq!(hash, 0xca2e9442);
}
