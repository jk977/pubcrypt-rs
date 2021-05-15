use rand::Rng;
use std::mem;

pub use math::Num;
use math::{mod_exp, primes, BigNum};

#[cfg(test)]
mod tests;

pub type Block = u32;

pub const BLOCK_BYTES: usize = mem::size_of::<Block>();
pub const NUM_BYTES: usize = mem::size_of::<Num>();
const PRIME_MIN: Num = Block::MAX as Num + 1;
const PRIME_MAX: Num = Num::MAX;

#[derive(Debug)]
pub struct Key {
    prime: Num,
    root: Num,
    value: Num,
}

impl Key {
    pub const KEY_BYTES: usize = NUM_BYTES * 3;

    /**
     * Convert key to bytes that can be saved to the disk.
     */
    pub fn serialize(&self) -> [u8; Self::KEY_BYTES] {
        let mut result = [0u8; Self::KEY_BYTES];
        result[..NUM_BYTES].copy_from_slice(&self.prime.to_be_bytes());
        result[NUM_BYTES..NUM_BYTES * 2].copy_from_slice(&self.root.to_be_bytes());
        result[NUM_BYTES * 2..NUM_BYTES * 3].copy_from_slice(&self.value.to_be_bytes());
        result
    }

    /**
     * Read key from serialized bytes.
     */
    pub fn deserialize(bytes: &[u8; Self::KEY_BYTES]) -> Self {
        let mut prime_buf = [0u8; NUM_BYTES];
        let mut root_buf = [0u8; NUM_BYTES];
        let mut value_buf = [0u8; NUM_BYTES];
        prime_buf.copy_from_slice(&bytes[..NUM_BYTES]);
        root_buf.copy_from_slice(&bytes[NUM_BYTES..NUM_BYTES * 2]);
        value_buf.copy_from_slice(&bytes[NUM_BYTES * 2..NUM_BYTES * 3]);

        Self {
            prime: Num::from_be_bytes(prime_buf),
            root: Num::from_be_bytes(root_buf),
            value: Num::from_be_bytes(value_buf),
        }
    }
}

#[derive(Debug)]
pub struct KeyPair {
    pub public: Key,
    pub private: Key,
}

impl KeyPair {
    /**
     * Generate a pair of keys, private and public, to use for encryption. The
     * return value is wrapped in a structure instead of just using a tuple to
     * avoid confusion about which is the public and which is the private key.
     */
    pub fn generate<T: Rng>(rng: &mut T) -> Self {
        let (prime, root) = primes::pick_random_with_root(PRIME_MIN, PRIME_MAX, rng);
        let priv_exp = rng.gen_range(1..prime - 1);
        let pub_exp = mod_exp(root, priv_exp, prime);

        Self {
            public: Key {
                prime,
                root,
                value: pub_exp,
            },
            private: Key {
                prime,
                root,
                value: priv_exp,
            },
        }
    }
}

/**
 * Encrypt `block` with the given key, using `r` for exponentiation.
 *
 * This function is separate from the randomly encrypted block to allow
 * deterministic testing.
 */
fn encrypt_block_det(block: Block, key: &Key, r: Num) -> (Num, Num) {
    assert!(key.prime > Block::MAX as Num);
    assert!(r < key.prime);
    assert!(r > 0);

    let er_mod_p = mod_exp(key.value, r, key.prime) as BigNum;
    let c1 = mod_exp(key.root, r, key.prime);
    let c2 = ((block as BigNum * er_mod_p) % key.prime as BigNum) as Num;
    (c1, c2)
}

/**
 * Encrypt `block` with the given public key and a random exponent generated by
 * `rng`.
 */
pub fn encrypt_block<T: Rng>(block: Block, key: &Key, rng: &mut T) -> (Num, Num) {
    encrypt_block_det(block, key, rng.gen_range(1..key.prime))
}

/**
 * Decrypt the ciphertext block represented by `c1` and `c2` with the given
 * private key.
 */
pub fn decrypt_block(c1: Num, c2: Num, key: &Key) -> Block {
    let c1_term = mod_exp(c1, key.prime - key.value - 1, key.prime) as BigNum;
    let c2_term = (c2 % key.prime) as BigNum;
    let result = (c1_term * c2_term) % key.prime as BigNum;

    assert!(result <= Block::MAX as BigNum);
    result as Block
}
