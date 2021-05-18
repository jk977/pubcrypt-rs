#[cfg(test)]
mod tests;

use crate::{mod_exp, Num};
use rand::prelude::{Rng, IteratorRandom};
use std::iter;

#[derive(Debug, Copy, Clone)]
pub enum PrimeError {
    InvalidRange,
    PrimeNotFound,
    RngError,
}

/**
 * Returns true if `val` is a witness for the compositeness of `n`, otherwise false.
 */
pub fn is_witness(n: Num, val: Num) -> bool {
    assert!(n >= 3);
    assert_eq!(n % 2, 1);

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
pub fn check_random_witnesses(n: Num, witness_count: usize) -> bool {
    assert!(n >= 3);
    assert_eq!(n % 2, 1);

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
 * Calculate a random primitive root of `prime`.
 */
pub fn get_primitive_root<T: Rng>(prime: Num, rng: &mut T) -> Num {
    assert!(is_prime(prime));
    unimplemented!()
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
 * Pick a random prime in the inclusive range `[min, max]` using `rng`.
 */
pub fn pick_random_prime<T: Rng>(min: Num, max: Num, rng: &mut T) -> Result<Num, PrimeError> {
    if min > max || max < 3 {
        return Err(PrimeError::InvalidRange);
    } else if min == max {
        return if is_prime(min) {
            Ok(min)
        } else {
            Err(PrimeError::PrimeNotFound)
        }
    }

    assert!(min < max);
    let gen_candidate = || (min..=max).choose(rng).unwrap();

    if range_contains_known_prime(min, max) {
        return iter::repeat_with(gen_candidate)
            .find(|n| is_prime(*n))
            .ok_or(PrimeError::PrimeNotFound);
    } else {
        // use a probabilistic approach to determine if a prime is present in the range.
        // `prime_attempts` is an arbitrary upper bound on the number of attempts allowed to choose
        // a random number in the range and check if it's prime.
        let prime_attempts = 10_usize.saturating_mul((min - max + 1) as usize);

        for n in iter::repeat_with(gen_candidate).take(prime_attempts) {
            if is_prime(n) {
                return Ok(n);
            }
        }
    }

    Err(PrimeError::PrimeNotFound)
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
    rng: &mut T
) -> Result<(Num, Num), PrimeError> {
    let prime = pick_random_prime(min, max, rng)?;
    let root = get_primitive_root(prime, rng);
    Ok((prime, root))
}
