use super::*;

use rand::{rngs::StdRng, SeedableRng};

#[test]
fn random_test_crypt_block() {
    const KEYS: usize = 10;
    const BLOCKS_PER_KEY: usize = 100_000;
    let mut rng = StdRng::from_entropy();

    for _ in 0..KEYS {
        let keys = KeyPair::generate(&mut rng).unwrap();

        for _ in 0..BLOCKS_PER_KEY {
            let block = rng.gen_range(0..=Block::MAX);
            let (c1, c2) = encrypt_block(block, &keys.public, &mut rng);
            let decrypted_block = decrypt_block(c1, c2, &keys.private);

            assert_eq!(
                block, decrypted_block,
                concat!(
                    "Encryption process failed. Plaintext = 0x{:x}. ",
                    "Ciphertext = (0x{:x}, 0x{:x}). Decrypted block = 0x{:x}."
                ),
                block, c1, c2, decrypted_block
            );
        }
    }
}
