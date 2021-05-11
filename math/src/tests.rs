use super::*;

// List of test values for modular exponentiation that does not contain the
// potentially problematic values 0 or 1.
const SAFE_VALS: [Num; 33] = [
    2,
    3,
    4,
    8,
    9,
    10,
    50,
    99,
    100,
    127,
    128,
    129,
    256,
    512,
    999,
    1024,
    5008,
    7777,
    9998,
    9999,
    10000,
    12500,
    15000,
    20000,
    50000,
    100000,
    1000000,
    10000000,
    10000000000,
    10000000000000,
    10000000000000000,
    10000000000000000000,
    Num::MAX,
];

fn check(base: Num, exp: Num, modulus: Num, expected: Num) {
    let result = mod_exp(base, exp, modulus);
    assert_eq!(
        result, expected,
        "Result {} is not equal to expected value {}",
        result, expected
    );
}

/**
 * Each argument in `args` is optional. Any missing argument is taken as a wildcard
 * for the set of values in `SAFE_VALS`.
 */
fn check_wildcard(mut args: [Option<Num>; 3], expected: Num) {
    for i in 0..args.len() {
        if let None = args[i] {
            for val in &SAFE_VALS {
                args[i] = Some(*val);
                check_wildcard(args, expected);
            }
        }
    }

    let [base, exp, modulus] = args;
    check(base.unwrap(), exp.unwrap(), modulus.unwrap(), expected);
}

#[test]
fn test_mod_exp() {
    // 0^x == 0 (mod n)
    check_wildcard([Some(0), None, None], 0);

    // x^0 == 1 (mod n)
    check_wildcard([None, Some(0), None], 1);

    // n^x == 0 (mod n)
    for val in &SAFE_VALS {
        check_wildcard([Some(*val), None, Some(*val)], 0);
    }

    // misc potential edge cases

    // manual tests
    check(1, 1, 1, 0);
    check(16, 4, 13, 3);
    check(23, 20, 29, 24);
    check(23, 391, 55, 12);
    check(31, 397, 55, 26);
    check(Num::MAX, 2, 100, 25);
    check(Num::MAX, Num::MAX, u32::MAX as Num, 0);
    check(Num::MAX, Num::MAX, Num::MAX, 0);
}
