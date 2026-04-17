//! Core CRT primitives: Extended GCD, modular inverse, K-Elimination.
//! Zero floats. All operations exact.

/// Extended Euclidean algorithm. Returns (gcd, x, y) where a*x + b*y = gcd.
pub fn extended_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    if b == 0 {
        return (a, 1, 0);
    }
    let (g, x1, y1) = extended_gcd(b, a % b);
    (g, y1, x1 - (a / b) * y1)
}

/// GCD for u128
pub fn gcd(a: u128, b: u128) -> u128 {
    if b == 0 { a } else { gcd(b, a % b) }
}

/// LCM
pub fn lcm(a: u128, b: u128) -> u128 {
    if a == 0 || b == 0 { 0 } else { a / gcd(a, b) * b }
}

/// Modular inverse of a mod m. Returns None if gcd(a, m) != 1.
pub fn mod_inv(a: i128, m: i128) -> Option<i128> {
    let (g, x, _) = extended_gcd(a.rem_euclid(m), m);
    if g != 1 {
        None
    } else {
        Some(x.rem_euclid(m))
    }
}

/// K-Elimination: compute winding number κ = (v_anchor - v_main) · M⁻¹ mod A
/// Requires gcd(m_main, m_anchor) = 1.
/// Returns None if coprimality violated.
pub fn k_eliminate(v_main: u128, m_main: u128, v_anchor: u128, m_anchor: u128) -> Option<u128> {
    let m_inv = mod_inv(m_main as i128, m_anchor as i128)?;
    let v_m_mod_a = (v_main % m_anchor) as i128;
    let v_a = v_anchor as i128;
    let m_a = m_anchor as i128;

    let diff = (v_a - v_m_mod_a).rem_euclid(m_a);
    let k = (diff * m_inv).rem_euclid(m_a);
    Some(k as u128)
}

/// CRT reconstruction from residue tuple. Output only — never on hot path.
pub fn crt_reconstruct(residues: &[(u128, u128)]) -> Option<u128> {
    // residues: [(r_i, m_i), ...]
    // Verify pairwise coprimality
    for i in 0..residues.len() {
        for j in (i + 1)..residues.len() {
            if gcd(residues[i].1, residues[j].1) != 1 {
                return None;
            }
        }
    }

    let big_m: u128 = residues.iter().map(|(_, m)| *m).product();
    let mut result: i128 = 0;

    for &(r_i, m_i) in residues {
        let big_m_i = big_m / m_i;
        let inv = mod_inv(big_m_i as i128, m_i as i128)?;
        result = (result + (r_i as i128) * (big_m_i as i128) * inv) % (big_m as i128);
    }

    Some(result.rem_euclid(big_m as i128) as u128)
}

/// Trial division factorization. Returns Vec<(prime, power)>.
pub fn factorize(mut n: u128) -> Vec<(u128, u32)> {
    let mut factors = Vec::new();
    let mut d = 2u128;
    while d * d <= n {
        let mut count = 0u32;
        while n % d == 0 {
            count += 1;
            n /= d;
        }
        if count > 0 {
            factors.push((d, count));
        }
        d += if d == 2 { 1 } else { 2 };
    }
    if n > 1 {
        factors.push((n, 1));
    }
    factors
}

/// Simple primality test (trial division, sufficient for our ranges)
pub fn is_prime(n: u128) -> bool {
    if n < 2 { return false; }
    if n < 4 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    let mut d = 5u128;
    while d * d <= n {
        if n % d == 0 || n % (d + 2) == 0 {
            return false;
        }
        d += 6;
    }
    true
}

/// Miller-Rabin for larger numbers. Deterministic for n < 3,317,044,064,679,887,385,961,981.
pub fn is_prime_miller_rabin(n: u128) -> bool {
    if n < 2 { return false; }
    if n == 2 || n == 3 { return true; }
    if n % 2 == 0 { return false; }

    // Write n-1 = 2^r * d
    let mut d = n - 1;
    let mut r = 0u32;
    while d % 2 == 0 {
        d /= 2;
        r += 1;
    }

    // Witnesses sufficient for deterministic results up to very large n
    let witnesses: &[u128] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

    'witness: for &a in witnesses {
        if a >= n { continue; }
        let mut x = mod_pow(a, d, n);
        if x == 1 || x == n - 1 { continue; }
        for _ in 0..r - 1 {
            x = mod_mul(x, x, n);
            if x == n - 1 { continue 'witness; }
        }
        return false;
    }
    true
}

/// Modular exponentiation: base^exp mod modulus
pub fn mod_pow(mut base: u128, mut exp: u128, modulus: u128) -> u128 {
    if modulus == 1 { return 0; }
    let mut result = 1u128;
    base %= modulus;
    while exp > 0 {
        if exp % 2 == 1 {
            result = mod_mul(result, base, modulus);
        }
        exp /= 2;
        base = mod_mul(base, base, modulus);
    }
    result
}

/// Modular multiplication avoiding overflow via u128 → widening
pub fn mod_mul(a: u128, b: u128, m: u128) -> u128 {
    // For values that fit in u64, direct multiplication is safe
    if a < (1u128 << 64) && b < (1u128 << 64) {
        return (a * b) % m;
    }
    // Otherwise use Russian peasant multiplication
    let mut result = 0u128;
    let mut a = a % m;
    let mut b = b % m;
    while b > 0 {
        if b & 1 == 1 {
            result = result.checked_add(a).map_or_else(
                || (result - (m - a)) % m,
                |v| v % m,
            );
        }
        a = a.checked_add(a).map_or_else(
            || (a - (m - a)) % m,
            |v| v % m,
        );
        b >>= 1;
    }
    result
}

/// Ramanujan sum c_q(n) for prime q
/// c_q(n) = μ(q/gcd(q,n)) · φ(q) / φ(q/gcd(q,n))
/// For prime q: c_q(n) = -1 if q ∤ n, q-1 if q | n
pub fn ramanujan_sum_prime(q: u128, n: u128) -> i128 {
    if n % q == 0 {
        (q - 1) as i128
    } else {
        -1
    }
}

/// Euler's totient for prime powers
pub fn euler_totient(n: u128) -> u128 {
    let factors = factorize(n);
    let mut result = n;
    for (p, _) in &factors {
        result = result / p * (p - 1);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k_eliminate_basic() {
        // x = 17, M = 6 (2×3), A = 5
        // v_M = 17 % 6 = 5, v_A = 17 % 5 = 2
        // k = (2 - 5) * 6^{-1} mod 5 = (-3) * 1 mod 5 = 2
        // Verify: x = k*M + v_M = 2*6 + 5 = 17 ✓
        let k = k_eliminate(5, 6, 2, 5).unwrap();
        assert_eq!(k, 2);
    }

    #[test]
    fn test_k_eliminate_coprimality_check() {
        // M = 6, A = 4: gcd = 2 ≠ 1, should fail
        assert!(k_eliminate(0, 6, 0, 4).is_none());
    }

    #[test]
    fn test_crt_reconstruct() {
        // x ≡ 2 mod 3, x ≡ 3 mod 5, x ≡ 2 mod 7 → x = 23
        let r = crt_reconstruct(&[(2, 3), (3, 5), (2, 7)]).unwrap();
        assert_eq!(r, 23);
    }

    #[test]
    fn test_factorize() {
        assert_eq!(factorize(260), vec![(2, 2), (5, 1), (13, 1)]);
        assert_eq!(factorize(819), vec![(3, 2), (7, 1), (13, 1)]);
        assert_eq!(factorize(11960), vec![(2, 3), (5, 1), (13, 1), (23, 1)]);
    }

    #[test]
    fn test_ramanujan_sum() {
        assert_eq!(ramanujan_sum_prime(5, 10), 4);  // 5 | 10
        assert_eq!(ramanujan_sum_prime(5, 7), -1);   // 5 ∤ 7
        assert_eq!(ramanujan_sum_prime(11, 22), 10);  // 11 | 22
        assert_eq!(ramanujan_sum_prime(11, 13), -1);  // 11 ∤ 13
    }

    #[test]
    fn test_miller_rabin() {
        assert!(is_prime_miller_rabin(2));
        assert!(is_prime_miller_rabin(17));
        assert!(is_prime_miller_rabin(104729));
        assert!(!is_prime_miller_rabin(4));
        assert!(!is_prime_miller_rabin(100));
    }

    // Validation identities from the spec
    #[test]
    fn test_validation_identities() {
        assert_eq!(gcd(260, 365), 5);       // V08
        assert_eq!(lcm(260, 365), 18980);   // V09
        assert_eq!(11960 % 780, 260);       // V13
        assert_eq!(11960 % 584, 280);       // V14
        assert_eq!(11960 % 378, 242);       // V15
        assert_eq!(260 % 11, 7);            // V22
        assert_eq!(11960 % 11, 3);          // V28
        assert_eq!(1872000 % 18980, 11960); // V19
        assert_eq!(1872000 / 18980, 98);    // V20
    }
}
