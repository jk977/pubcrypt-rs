#[cfg(test)]
mod tests;

use crate::{mod_exp, Num};
use rand::prelude::Rng;
use std::iter;

#[derive(Debug, Copy, Clone)]
pub enum PrimeError {
    InvalidRange,
    PrimeNotFound,
}

/**
 * Returns true if `val` is a witness for the compositeness of `n`, otherwise false.
 */
pub fn is_witness(n: Num, val: Num) -> bool {
    if n < 3 || n % 2 == 0 {
        // `n` isn't prime
        return false;
    }

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
    assert_eq!(q % 2, 1);

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
fn check_random_witnesses<T: Rng>(n: Num, witness_count: usize, rng: &mut T) -> bool {
    if n < 3 || n % 2 == 0 {
        // `n` isn't prime
        return false;
    }

    iter::repeat_with(|| rng.gen_range(2..n - 1))
        .take(witness_count)
        .any(|val| is_witness(n, val))
}

fn is_prime<T: Rng>(n: Num, rng: &mut T) -> bool {
    const WITNESS_COUNT: usize = 25;
    !check_random_witnesses(n, WITNESS_COUNT, rng)
}

/**
 * Check if the given range is known to contain a prime. For ranges with a small minimum, this is
 * done via Bertrand's postulate. For ranges with a minimum greater than 25, this is done by
 * checking the value of `max` relative to `min`.
 *
 * The math is based on proofs by Dusart and Nagura, who proved the following:
 *  - For `x >= 25`, a prime exists in the range `[x, 6x / 5]` (See: [Nagura 1952][1])
 *  - For `x >= 3275`, a prime exists in the range `[x, x + x / (2 ln^2 x)]` (See: [Dusart 1998][1])
 *  - For `x >= 89693`, a prime exists in the range `[x, x + x / (ln^3 x)]` (See: [Dusart 2016][3])
 *
 * For use in this function, the upper bounds can be factored into the form `(1 + epsilon) * x`,
 * where `epsilon` is some function of `x`.
 *
 * Returns true if the range is known to contain a prime, or false if it is unknown. Note that a
 * false result does not indicate the lack of primes in the range.
 *
 * [1]: https://projecteuclid.org/download/pdf_1/euclid.pja/1195570997
 * [2]: https://www.unilim.fr/laco/theses/1998/T1998_01.pdf
 * [3]: http://link.springer.com/article/10.1007/s11139-016-9839-4
 */
fn range_contains_known_prime(min: Num, max: Num) -> bool {
    assert!(min >= 3);
    assert!(max >= min);

    const NAGURA_1952_MIN: Num = 25;
    const DUSART_1998_MIN: Num = 3275;
    const DUSART_2016_MIN: Num = 89693;

    if min < NAGURA_1952_MIN {
        // Bertrand's postulate
        return max >= 2 * min;
    }

    let fmin = min as f64;

    let epsilon = if min >= DUSART_2016_MIN {
        fmin.ln().powi(-3)
    } else if min >= DUSART_1998_MIN {
        (2.0 * fmin.ln().powi(2)).powi(-1)
    } else {
        assert!(min >= NAGURA_1952_MIN);
        0.2
    };

    // use `f64::ceil` to ensure no mathematical errors occur due to floating point inaccuracies
    // with a nonzero mantissa, at the cost of a slightly stricter bound than the theoretical value
    let bound = ((1.0 + epsilon) * fmin).ceil() as Num;
    max >= bound
}

/**
 * Pick a random prime in the inclusive range `[min, max]` using `rng`, and a random primitive root
 * of the prime.
 *
 * On success, returns a pair `(prime, primitive_root)`.
 */
pub fn pick_random_with_root<T: Rng>(
    min: Num,
    max: Num,
    rng: &mut T,
) -> Result<(Num, Num), PrimeError> {
    if min > max || max < 3 {
        return Err(PrimeError::InvalidRange);
    }

    let mut primality_rng = rand::thread_rng();
    let gen_candidate = || (rng.gen_range(min..=max) - 1) / 2;
    let get_prime = |n| {
        // FilterMap closure to return only primes with a primitive root of 2
        let prime = 2 * n + 1;
        if n % 2 == 1
            && n % 12 == 5
            && is_prime(n, &mut primality_rng)
            && is_prime(prime, &mut primality_rng)
        {
            Some(prime)
        } else {
            None
        }
    };

    let candidates = iter::repeat_with(gen_candidate);
    let result = if range_contains_known_prime(min, max) {
        candidates.filter_map(get_prime).next()
    } else {
        // unknown whether or not the range contains a prime, so use a probabilistic approach
        let prime_attempts = 5_usize.saturating_mul((min - max + 1) as usize);
        candidates.take(prime_attempts).filter_map(get_prime).next()
    };

    if let Some(prime) = result {
        const ROOT: Num = 2;
        Ok((prime, ROOT))
    } else {
        Err(PrimeError::PrimeNotFound)
    }
}
