#[cfg(test)]
mod tests;

use crate::{mod_exp, Num};
use rand::Rng;
use std::iter;

/**
 * Returns true if `val` is a witness for the compositeness of `n`, otherwise false.
 */
pub fn is_witness(n: Num, val: Num) -> bool {
    assert!(n >= 3);
    assert!(n % 2 == 1);

    let mut k = 0;
    let mut q = n - 1;

    // break n-1 up into 2^k * q
    while q % 2 == 0 {
        q >>= 1;
        k += 1;
    }

    // ensure that the previous section worked correctly
    assert_eq!((2 as Num).pow(k) * q, n - 1);
    assert!(k > 0);
    assert!(q % 2 == 1);

    // check if `i` satisfies a^(2^i * q) != n - 1 (mod n) needed for compositeness
    let check_second = |i| {
        let exponent = (2 as Num).pow(i) * q;
        mod_exp(val, exponent, n) != n - 1
    };

    // check fermat's little theorem
    mod_exp(val, q, n) != 1 && (0..k).all(check_second)
}

/**
 * Run `check_witness` on `n` with `witnesses` random witnesses.
 *
 * Returns true if a witness for the compositeness of `n` was found, otherwise false.
 */
pub fn check_random_witnesses(n: Num, witness_count: usize) -> bool {
    assert!(n >= 3);
    assert!(n % 2 == 1);

    let mut rng = rand::thread_rng();
    iter::repeat_with(|| rng.gen_range(2..n - 1))
        .take(witness_count)
        .any(|val| is_witness(n, val))
}

fn is_prime(n: Num) -> bool {
    const WITNESS_COUNT: usize = 25;
    !check_random_witnesses(n, WITNESS_COUNT)
}

/**
 * Pick a random prime greater than or equal to `min` using `rng`.
 *
 * Returns a pair `(prime, primitive_root)`, where `primitive_root` is currently
 * the constant value 2.
 *
 * WARNING: If no primes exist in the range [min, max] with a primitive root of 2,
 *          this function will not return.
 *
 * Panics if `min >= max`, or `max < 3`.
 */
pub fn pick_random_with_root<T: Rng>(mut min: Num, max: Num, rng: &mut T) -> (Num, Num) {
    const PRIMITIVE_ROOT: Num = 2;

    if max == 3 {
        return (max, PRIMITIVE_ROOT);
    } else if min < 3 {
        // no primes less than 3, and having `min >= 3` makes an optimization
        // possible in the generation
        min = 3;
    }

    assert!(min < max);
    let prime = iter::repeat_with(|| (rng.gen_range(min..=max) - 1) / 2)
        .filter(|q| *q % 2 == 1 && *q % 12 == 5 && is_prime(*q))
        .map(|q| 2 * q + 1)
        .find(|p| is_prime(*p))
        .unwrap();

    (prime, PRIMITIVE_ROOT)
}
