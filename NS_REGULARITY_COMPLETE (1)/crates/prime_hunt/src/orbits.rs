//! Orbit Analysis: Multiplicative structure on the CR=18980 torus.
//!
//! Key insight: Each element g ∈ M_H generates an orbit {g^k mod CR : k=1,2,...}
//! with period ord(g) = multiplicative order of g mod CR.
//!
//! Questions:
//! 1. How do orbits partition M_H?
//! 2. Does prime density vary across orbit families?
//! 3. Which generators are primitive roots (maximal order)?
//! 4. Do orbit periods correlate with CRT lane structure?
//!
//! CR = 18980 = 2² × 5 × 13 × 73
//! φ(CR) = 18980 × (1-1/2)(1-1/5)(1-1/13)(1-1/73) = 6912
//! λ(CR) = lcm(λ(4), λ(5), λ(13), λ(73)) = lcm(2, 4, 12, 72) = 72
//!
//! Maximum multiplicative order is λ(CR) = 72, NOT φ(CR) = 6912.
//! This means NO primitive root exists mod CR (since CR is not 1, 2, 4, p^k, or 2p^k).

use crate::crt::{self, gcd, factorize, is_prime, euler_totient, mod_pow};
use crate::monoid::{self, in_monoid, classify_monoid, MonoidClass, M_COPRIME};
use crate::helical::CR;

/// Carmichael's lambda function: the maximum multiplicative order mod n.
/// λ(2^a) = 2^(a-2) for a≥3, 2^(a-1) for a≤2
/// λ(p^a) = p^(a-1)(p-1) for odd prime p
/// λ(n) = lcm(λ(p1^a1), ..., λ(pk^ak))
pub fn carmichael_lambda(n: u128) -> u128 {
    if n <= 2 { return 1; }
    let factors = factorize(n);
    let mut lambda = 1u128;

    for &(p, a) in &factors {
        let lam_pa = if p == 2 {
            if a <= 2 {
                1u128 << (a - 1) // 2^(a-1)
            } else {
                1u128 << (a - 2) // 2^(a-2) for a >= 3
            }
        } else {
            // λ(p^a) = p^(a-1) × (p-1)
            let mut val = p - 1;
            for _ in 1..a {
                val *= p;
            }
            val
        };
        lambda = crt::lcm(lambda, lam_pa);
    }
    lambda
}

/// Compute multiplicative order of g mod n.
/// Returns None if gcd(g, n) ≠ 1.
pub fn multiplicative_order(g: u128, n: u128) -> Option<u128> {
    if gcd(g, n) != 1 { return None; }
    if n == 1 { return Some(1); }

    let lambda = carmichael_lambda(n);

    // The order must divide λ(n). Find the smallest divisor d of λ(n) such that g^d ≡ 1.
    let divisors = divisors_of(lambda);
    for d in divisors {
        if mod_pow(g, d, n) == 1 {
            return Some(d);
        }
    }
    Some(lambda) // should always be found
}

/// Get sorted divisors of n.
pub fn divisors_of(n: u128) -> Vec<u128> {
    let mut divs = Vec::new();
    let mut d = 1u128;
    while d * d <= n {
        if n % d == 0 {
            divs.push(d);
            if d != n / d {
                divs.push(n / d);
            }
        }
        d += 1;
    }
    divs.sort();
    divs
}

/// Compute the full orbit of g mod CR: {g, g², g³, ..., g^ord = 1}
pub fn compute_orbit(g: u128) -> Option<Orbit> {
    let ord = multiplicative_order(g, CR)?;

    let mut theta_sequence = Vec::with_capacity(ord as usize);
    let mut current = 1u128;
    for _ in 0..ord {
        current = crt::mod_mul(current, g % CR, CR);
        theta_sequence.push(current);
    }

    // Verify closure
    assert_eq!(*theta_sequence.last().unwrap(), 1,
        "Orbit of {} did not close: last element = {}", g, theta_sequence.last().unwrap());

    // CRT decomposition of the orbit: track residues mod each factor of CR
    // CR = 4 × 5 × 13 × 73
    let cr_factors = [4u128, 5, 13, 73];
    let sub_orders: Vec<u128> = cr_factors.iter()
        .map(|&m| multiplicative_order(g % m, m).unwrap_or(1))
        .collect();

    Some(Orbit {
        generator: g,
        order: ord,
        theta_sequence,
        sub_orders,
        cr_factors: cr_factors.to_vec(),
    })
}

#[derive(Debug)]
pub struct Orbit {
    pub generator: u128,
    pub order: u128,
    pub theta_sequence: Vec<u128>,
    pub sub_orders: Vec<u128>,  // orders mod each CR factor
    pub cr_factors: Vec<u128>,
}

/// Orbit family: group generators by their multiplicative order.
#[derive(Debug)]
pub struct OrbitFamily {
    pub order: u128,
    pub generators: Vec<u128>,
    pub prime_count: u32,
    pub composite_count: u32,
    pub prime_density: f64,
}

/// Classify all elements of M_H in [2, limit] by orbit family.
pub fn orbit_family_census(limit: u128) -> OrbitFamilyCensus {
    let lambda = carmichael_lambda(CR);
    let possible_orders = divisors_of(lambda);

    let mut families: std::collections::HashMap<u128, OrbitFamily> = std::collections::HashMap::new();

    // Initialize families for all divisors of λ(CR)
    for &ord in &possible_orders {
        families.insert(ord, OrbitFamily {
            order: ord,
            generators: Vec::new(),
            prime_count: 0,
            composite_count: 0,
            prime_density: 0.0,
        });
    }

    let mut total_computed = 0u32;
    let mut order_failures = 0u32;

    for n in 2..=limit {
        if !in_monoid(n) { continue; }
        if gcd(n, CR) != 1 { continue; }

        total_computed += 1;
        match multiplicative_order(n, CR) {
            Some(ord) => {
                let family = families.entry(ord).or_insert(OrbitFamily {
                    order: ord,
                    generators: Vec::new(),
                    prime_count: 0,
                    composite_count: 0,
                    prime_density: 0.0,
                });
                family.generators.push(n);
                if is_prime(n) {
                    family.prime_count += 1;
                } else {
                    family.composite_count += 1;
                }
            }
            None => order_failures += 1,
        }
    }

    // Compute densities
    for family in families.values_mut() {
        let total = family.prime_count + family.composite_count;
        family.prime_density = if total > 0 {
            family.prime_count as f64 / total as f64
        } else { 0.0 };
    }

    // Sort by order
    let mut sorted_families: Vec<OrbitFamily> = families.into_values().collect();
    sorted_families.sort_by_key(|f| f.order);

    OrbitFamilyCensus {
        limit,
        lambda_cr: lambda,
        phi_cr: euler_totient(CR),
        total_computed,
        order_failures,
        families: sorted_families,
        possible_orders,
    }
}

#[derive(Debug)]
pub struct OrbitFamilyCensus {
    pub limit: u128,
    pub lambda_cr: u128,
    pub phi_cr: u128,
    pub total_computed: u32,
    pub order_failures: u32,
    pub families: Vec<OrbitFamily>,
    pub possible_orders: Vec<u128>,
}

/// Deep orbit analysis: for selected generators, compute full orbit
/// and analyze prime density AT EACH θ position.
pub fn orbit_prime_density(generators: &[u128], winding_limit: u128) -> Vec<OrbitPrimeDensity> {
    let mut results = Vec::new();

    for &g in generators {
        let orbit = match compute_orbit(g) {
            Some(o) => o,
            None => continue,
        };

        // For each θ in the orbit, count how many n = θ + k*CR are prime
        let mut theta_prime_counts: Vec<(u128, u32, u32)> = Vec::new(); // (θ, primes, total)

        for &theta in &orbit.theta_sequence {
            let mut primes = 0u32;
            let mut total = 0u32;
            for k in 0..winding_limit {
                let n = theta + k * CR;
                if n < 2 { continue; }
                total += 1;
                if is_prime(n) {
                    primes += 1;
                }
            }
            theta_prime_counts.push((theta, primes, total));
        }

        // Overall density for this orbit vs off-orbit
        let on_orbit_primes: u32 = theta_prime_counts.iter().map(|&(_, p, _)| p).sum();
        let on_orbit_total: u32 = theta_prime_counts.iter().map(|&(_, _, t)| t).sum();

        results.push(OrbitPrimeDensity {
            generator: g,
            order: orbit.order,
            sub_orders: orbit.sub_orders,
            theta_prime_counts,
            on_orbit_primes,
            on_orbit_total,
            density: if on_orbit_total > 0 {
                on_orbit_primes as f64 / on_orbit_total as f64
            } else { 0.0 },
        });
    }
    results
}

#[derive(Debug)]
pub struct OrbitPrimeDensity {
    pub generator: u128,
    pub order: u128,
    pub sub_orders: Vec<u128>,
    pub theta_prime_counts: Vec<(u128, u32, u32)>,
    pub on_orbit_primes: u32,
    pub on_orbit_total: u32,
    pub density: f64,
}

/// Analyze the CRT decomposition of orbits.
/// CR = 4 × 5 × 13 × 73 means each orbit decomposes into sub-orbits
/// on each factor ring. The full order = lcm of sub-orders.
pub fn crt_orbit_decomposition(generators: &[u128]) -> Vec<CRTOrbitDecomp> {
    let cr_factors = [4u128, 5, 13, 73];
    let sub_lambdas: Vec<u128> = cr_factors.iter()
        .map(|&m| carmichael_lambda(m))
        .collect();

    let mut results = Vec::new();

    for &g in generators {
        if gcd(g, CR) != 1 { continue; }

        let sub_residues: Vec<u128> = cr_factors.iter().map(|&m| g % m).collect();
        let sub_orders: Vec<u128> = cr_factors.iter().enumerate()
            .map(|(i, &m)| multiplicative_order(sub_residues[i], m).unwrap_or(0))
            .collect();
        let full_order = sub_orders.iter().copied().reduce(crt::lcm).unwrap_or(1);

        // For each sub-ring, compute the orbit
        let sub_orbits: Vec<Vec<u128>> = cr_factors.iter().enumerate()
            .map(|(i, &m)| {
                let ord = sub_orders[i];
                let mut orbit = Vec::new();
                let mut current = 1u128;
                for _ in 0..ord {
                    current = (current * (g % m)) % m;
                    orbit.push(current);
                }
                orbit
            })
            .collect();

        results.push(CRTOrbitDecomp {
            generator: g,
            is_prime: is_prime(g),
            full_order,
            sub_residues,
            sub_orders: sub_orders.clone(),
            sub_lambdas: sub_lambdas.clone(),
            sub_orbits,
            cr_factors: cr_factors.to_vec(),
            at_max_order: sub_orders.iter().zip(sub_lambdas.iter())
                .all(|(&o, &l)| o == l),
        });
    }
    results
}

#[derive(Debug)]
pub struct CRTOrbitDecomp {
    pub generator: u128,
    pub is_prime: bool,
    pub full_order: u128,
    pub sub_residues: Vec<u128>,
    pub sub_orders: Vec<u128>,
    pub sub_lambdas: Vec<u128>,
    pub sub_orbits: Vec<Vec<u128>>,
    pub cr_factors: Vec<u128>,
    pub at_max_order: bool,
}

/// Artin primitive root analysis:
/// For which primes p is g a primitive root (ord_p(g) = p-1)?
/// Artin's conjecture: every non-square non-{-1} integer is a primitive root
/// for infinitely many primes.
pub fn artin_primitive_root_test(g: u128, prime_limit: u128) -> ArtinReport {
    let mut primes_tested = 0u32;
    let mut is_primitive_root = 0u32;
    let mut not_primitive_root = 0u32;
    let mut primitive_root_primes = Vec::new();

    for p in 2..=prime_limit {
        if !is_prime(p) { continue; }
        if p == g { continue; }
        if gcd(g, p) != 1 { continue; }

        primes_tested += 1;
        let ord = multiplicative_order(g, p).unwrap_or(0);

        if ord == p - 1 {
            is_primitive_root += 1;
            if primitive_root_primes.len() < 30 {
                primitive_root_primes.push(p);
            }
        } else {
            not_primitive_root += 1;
        }
    }

    let density = if primes_tested > 0 {
        is_primitive_root as f64 / primes_tested as f64
    } else { 0.0 };

    // Artin's constant ≈ 0.3739558136...
    let artin_constant = 0.3739558136;

    ArtinReport {
        generator: g,
        primes_tested,
        is_primitive_root_count: is_primitive_root,
        not_primitive_root_count: not_primitive_root,
        density,
        artin_constant,
        ratio_to_artin: density / artin_constant,
        primitive_root_primes,
    }
}

#[derive(Debug)]
pub struct ArtinReport {
    pub generator: u128,
    pub primes_tested: u32,
    pub is_primitive_root_count: u32,
    pub not_primitive_root_count: u32,
    pub density: f64,
    pub artin_constant: f64,
    pub ratio_to_artin: f64,
    pub primitive_root_primes: Vec<u128>,
}

/// Orbit collision analysis: when two orbits share θ positions,
/// how does that affect prime density at those positions?
pub fn orbit_collision_analysis(generators: &[u128]) -> CollisionReport {
    let mut theta_to_generators: std::collections::HashMap<u128, Vec<u128>> = std::collections::HashMap::new();

    for &g in generators {
        if let Some(orbit) = compute_orbit(g) {
            for &theta in &orbit.theta_sequence {
                theta_to_generators.entry(theta).or_default().push(g);
            }
        }
    }

    // Count collisions (θ values hit by multiple generators)
    let mut collision_count = 0u32;
    let mut max_collision = 0u32;
    let mut collision_thetas = Vec::new();

    for (&theta, gens) in &theta_to_generators {
        if gens.len() > 1 {
            collision_count += 1;
            if gens.len() as u32 > max_collision {
                max_collision = gens.len() as u32;
            }
            if collision_thetas.len() < 20 {
                collision_thetas.push((theta, gens.clone()));
            }
        }
    }

    // For collision vs non-collision θ values, compare prime density
    // (over several windings)
    let winding_check = 20u128;
    let mut collision_primes = 0u32;
    let mut collision_total = 0u32;
    let mut solo_primes = 0u32;
    let mut solo_total = 0u32;

    for (&theta, gens) in &theta_to_generators {
        for k in 0..winding_check {
            let n = theta + k * CR;
            if n < 2 { continue; }
            let prime = is_prime(n);
            if gens.len() > 1 {
                collision_total += 1;
                if prime { collision_primes += 1; }
            } else {
                solo_total += 1;
                if prime { solo_primes += 1; }
            }
        }
    }

    CollisionReport {
        total_thetas: theta_to_generators.len(),
        collision_count,
        max_collision,
        collision_thetas,
        collision_prime_density: if collision_total > 0 {
            collision_primes as f64 / collision_total as f64
        } else { 0.0 },
        solo_prime_density: if solo_total > 0 {
            solo_primes as f64 / solo_total as f64
        } else { 0.0 },
    }
}

#[derive(Debug)]
pub struct CollisionReport {
    pub total_thetas: usize,
    pub collision_count: u32,
    pub max_collision: u32,
    pub collision_thetas: Vec<(u128, Vec<u128>)>,
    pub collision_prime_density: f64,
    pub solo_prime_density: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carmichael_lambda_cr() {
        // CR = 2² × 5 × 13 × 73
        // λ(4) = 2, λ(5) = 4, λ(13) = 12, λ(73) = 72
        // λ(CR) = lcm(2, 4, 12, 72) = 72
        assert_eq!(carmichael_lambda(CR), 72);
    }

    #[test]
    fn test_no_primitive_root_mod_cr() {
        // Since CR = 4×5×13×73 has more than one odd prime factor,
        // no primitive root exists. Max order = λ(CR) = 72 < φ(CR) = 6912.
        let lambda = carmichael_lambda(CR);
        let phi = euler_totient(CR);
        assert_eq!(lambda, 72);
        assert_eq!(phi, 6912);
        assert!(lambda < phi, "λ(CR) < φ(CR), so no primitive root");
    }

    #[test]
    fn test_order_27() {
        // 27 mod CR: 27^4 ≡ 1 (mod 18980)
        let ord = multiplicative_order(27, CR).unwrap();
        assert_eq!(ord, 4);
        assert_eq!(mod_pow(27, 4, CR), 1);
    }

    #[test]
    fn test_order_17() {
        let ord = multiplicative_order(17, CR).unwrap();
        // Verify: 17^ord ≡ 1 mod CR
        assert_eq!(mod_pow(17, ord, CR), 1);
        // Verify minimality: 17^(ord-1) ≢ 1
        if ord > 1 {
            assert_ne!(mod_pow(17, ord - 1, CR), 1);
        }
        // Must divide λ(CR) = 72
        assert_eq!(72 % ord, 0);
    }

    #[test]
    fn test_orbit_closure() {
        let orbit = compute_orbit(27).unwrap();
        assert_eq!(orbit.order, 4);
        assert_eq!(*orbit.theta_sequence.last().unwrap(), 1);
        assert_eq!(orbit.theta_sequence, vec![27, 729, 703, 1]);
    }

    #[test]
    fn test_crt_orbit_decomp() {
        // 27 mod 4 = 3, ord(3, 4) = 2
        // 27 mod 5 = 2, ord(2, 5) = 4
        // 27 mod 13 = 1, ord(1, 13) = 1
        // 27 mod 73 = 27, ord(27, 73) = 4
        // lcm(2, 4, 1, 4) = 4 ✓
        let decomps = crt_orbit_decomposition(&[27]);
        assert_eq!(decomps[0].full_order, 4);
        assert_eq!(decomps[0].sub_orders, vec![2, 4, 1, 4]);
    }

    #[test]
    fn test_divisors() {
        assert_eq!(divisors_of(12), vec![1, 2, 3, 4, 6, 12]);
        assert_eq!(divisors_of(72), vec![1, 2, 3, 4, 6, 8, 9, 12, 18, 24, 36, 72]);
    }
}
