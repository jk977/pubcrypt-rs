#[cfg(feature = "rand")]
pub mod primes;

#[cfg(test)]
mod tests;

use std::mem;

pub type Num = u64;
pub type BigNum = u128;

const BITS_PER_BYTE: usize = 8;

/**
 * Shortcuts for modular exponentiation. The mathematical justification(s) are
 * (for the most part) briefly listed within each branch.
 */

fn get_optimization(base: Num, exponent: Num, modulus: Num) -> Option<Num> {
    if exponent == 0 {
        // x^0 == 1
        Some(1 % modulus)
    } else if base == 0 || base == modulus || modulus == 1 {
        // 0^x == n^x == 0 (mod n)
        // x % 1 == 0
        Some(0)
    } else if base == 1 || modulus == 2 {
        // 1^x == 1 (mod n)
        // x^n % 2 == x % 2

        /*
         * Justification for the modulo 2 identity:
         *
         * An even number times another even number is even, while an odd number
         * times another odd number is odd. In modulo 2 arithmetic, because even
         * numbers are congruent to 0 and odd numbers are congruent to 1, no
         * exponentiation is needed before taking the result modulo 2. The
         * exponentiation would only lead to repeated unnecessary odd-times-odd
         * or even-times-even operations.
         *
         * More formally, because `a*b == (a % n) * (b % n) (mod n)`, exponentiation
         * can be written as `a^b == (a % n) * (a % n) * ... == (a % n)^b (mod n)`.
         * If `a` is even, `a % 2` is `0`. Otherwise, `a % 2` is 1. `(a^b) % n` is
         * therefore equivalent to either `0^b == 0` (when `a` is even) or `1^b`
         * (when `a` is odd). Either way, this is equivalent to simply `a % 2`.
         *
         * Thus, `a^b == a (mod 2)`.
         */

        Some(base % modulus)
    } else {
        None
    }
}

/**
 * Calculate `base` raised to `exponent` modulo `modulus`.
 */
pub fn mod_exp(mut base: Num, exponent: Num, modulus: Num) -> Num {
    assert!(modulus > 0);
    assert!(base.saturating_add(exponent) != 0);

    base %= modulus;

    if let Some(val) = get_optimization(base, exponent, modulus) {
        return val;
    }

    let num_bits = mem::size_of::<Num>() * BITS_PER_BYTE;
    let mut result: BigNum = 1;
    let mut mask: Num = (1 as Num) << (num_bits - 1);

    // iterate through each bit in the exponent, performing the square/multiply ops
    // as specified in the algorithm covered in class
    for _ in 0..num_bits {
        result = result.pow(2) % modulus as BigNum;

        if exponent & mask != 0 {
            result = (result * base as BigNum) % modulus as BigNum;
        }

        mask >>= 1;
    }

    result as Num
}
