use super::*;

use rand::{rngs::StdRng, SeedableRng};

#[test]
fn test_check_witness() {
    assert_eq!(is_witness(221, 174), false);
    assert_eq!(is_witness(221, 137), true);
    assert_eq!(is_witness(252601, 85132), true);
    assert_eq!(is_witness(3057601, 99908), true);
    assert_eq!(is_witness(104717, 96152), false);
    assert_eq!(is_witness(577757, 314997), false);
    assert_eq!(is_witness(101089, 5), false);
    assert_eq!(is_witness(280001, 105532), false);
    assert_eq!(is_witness(95721889, 21906436), true);
}

#[test]
fn test_pick_random() {
    const ITERATIONS: usize = 100;
    const MIN: Num = (u32::MAX as Num) + 1;
    const MAX: Num = MIN + u32::MAX as Num;
    let mut rng = StdRng::from_entropy();

    for _ in 0..ITERATIONS {
        let (prime, _) = pick_random_with_root(MIN, MAX, &mut rng);
        assert!(prime >= MIN);
        assert!(prime <= MAX);
        assert!(is_prime(prime));
    }
}
