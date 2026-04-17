//! Composite Anchor Superiority Test
//!
//! Core question: Is there a modulus M where composite generators
//! outperform prime generators in covering prime-rich residue classes?
//!
//! This connects to Theorem T3 (98.8% of spectral power in composite
//! interaction modes) and the 323=17×19 full-lane-saturation discovery.
//!
//! Strategy:
//! 1. Test moduli with varying λ(M)/φ(M) ratios
//! 2. For each M, compare orbit coverage of prime vs composite generators
//! 3. Measure prime density at covered θ positions
//! 4. Test lane saturation: does saturating all CRT sub-lanes matter?

use crate::crt::{self, gcd, factorize, is_prime, euler_totient, mod_pow};
use crate::orbits::{self, carmichael_lambda, multiplicative_order};

/// A modulus candidate with its structural properties.
#[derive(Debug, Clone)]
pub struct ModulusProfile {
    pub m: u128,
    pub factors: Vec<(u128, u32)>,
    pub phi: u128,
    pub lambda: u128,
    pub lambda_phi_ratio: f64,
    pub num_prime_factors: usize,
}

impl ModulusProfile {
    pub fn new(m: u128) -> Self {
        let factors = factorize(m);
        let phi = euler_totient(m);
        let lambda = carmichael_lambda(m);
        Self {
            m,
            factors: factors.clone(),
            phi,
            lambda,
            lambda_phi_ratio: lambda as f64 / phi as f64,
            num_prime_factors: factors.len(),
        }
    }
}

/// Generator performance on a given modulus.
#[derive(Debug)]
pub struct GeneratorPerformance {
    pub generator: u128,
    pub is_prime: bool,
    pub is_composite_coprime: bool, // composite with all factors > 13
    pub order: u128,
    pub order_ratio: f64, // order / lambda(M)
    pub orbit_theta_count: usize, // distinct θ positions
    pub orbit_primes: u32, // primes found at orbit θ positions (over K windings)
    pub orbit_total: u32,
    pub prime_density: f64,
    pub sub_orders: Vec<u128>,
    pub lanes_saturated: u32, // how many sub-lanes are at max order
    pub total_lanes: u32,
}

/// Competition result between prime and composite generators on a modulus.
#[derive(Debug)]
pub struct CompetitionResult {
    pub modulus: ModulusProfile,
    pub prime_generators: Vec<GeneratorPerformance>,
    pub composite_generators: Vec<GeneratorPerformance>,
    pub avg_prime_gen_density: f64,
    pub avg_composite_gen_density: f64,
    pub composite_advantage: f64, // ratio composite/prime density
    pub best_prime_gen: Option<u128>,
    pub best_composite_gen: Option<u128>,
    pub best_prime_density: f64,
    pub best_composite_density: f64,
    pub max_order_achievers_prime: u32,
    pub max_order_achievers_composite: u32,
    pub full_saturation_prime: u32,
    pub full_saturation_composite: u32,
}

/// Run the generator competition on a single modulus.
pub fn compete_on_modulus(
    m: u128,
    gen_limit: u128,
    winding_limit: u128,
) -> CompetitionResult {
    let profile = ModulusProfile::new(m);
    let factors = &profile.factors;
    let factor_primes: Vec<u128> = factors.iter().map(|&(p, _)| p).collect();

    // Compute sub-lambdas for CRT decomposition
    let sub_moduli: Vec<u128> = factors.iter()
        .map(|&(p, a)| {
            let mut v = 1u128;
            for _ in 0..a { v *= p; }
            v
        })
        .collect();
    let sub_lambdas: Vec<u128> = sub_moduli.iter()
        .map(|&sm| carmichael_lambda(sm))
        .collect();

    let mut prime_gens = Vec::new();
    let mut composite_gens = Vec::new();

    for g in 2..=gen_limit {
        if gcd(g, m) != 1 { continue; }

        let is_p = is_prime(g);
        let g_factors = factorize(g);
        let is_composite_coprime = !is_p && g > 1
            && g_factors.iter().all(|&(p, _)| p > 13)
            && gcd(g, m) == 1;

        // Only test primes and "interesting" composites
        if !is_p && !is_composite_coprime { continue; }

        let ord = match multiplicative_order(g, m) {
            Some(o) => o,
            None => continue,
        };

        // Sub-orders
        let sub_orders: Vec<u128> = sub_moduli.iter()
            .map(|&sm| multiplicative_order(g % sm, sm).unwrap_or(0))
            .collect();

        let lanes_saturated = sub_orders.iter().zip(sub_lambdas.iter())
            .filter(|(&o, &l)| o == l)
            .count() as u32;

        // Compute orbit θ positions
        let mut orbit_thetas = Vec::new();
        let mut current = 1u128;
        for _ in 0..ord {
            current = crt::mod_mul(current, g % m, m);
            orbit_thetas.push(current);
        }

        // Count primes at orbit positions over windings
        let mut primes = 0u32;
        let mut total = 0u32;
        for &theta in &orbit_thetas {
            for k in 0..winding_limit {
                let n = theta + k * m;
                if n < 2 { continue; }
                total += 1;
                if is_prime(n) {
                    primes += 1;
                }
            }
        }

        let density = if total > 0 { primes as f64 / total as f64 } else { 0.0 };

        let perf = GeneratorPerformance {
            generator: g,
            is_prime: is_p,
            is_composite_coprime,
            order: ord,
            order_ratio: ord as f64 / profile.lambda as f64,
            orbit_theta_count: orbit_thetas.len(),
            orbit_primes: primes,
            orbit_total: total,
            prime_density: density,
            sub_orders,
            lanes_saturated,
            total_lanes: sub_moduli.len() as u32,
        };

        if is_p {
            prime_gens.push(perf);
        } else {
            composite_gens.push(perf);
        }
    }

    // Sort by density descending
    prime_gens.sort_by(|a, b| b.prime_density.partial_cmp(&a.prime_density).unwrap());
    composite_gens.sort_by(|a, b| b.prime_density.partial_cmp(&a.prime_density).unwrap());

    // Compute averages
    let avg_prime = if prime_gens.is_empty() { 0.0 }
        else { prime_gens.iter().map(|g| g.prime_density).sum::<f64>() / prime_gens.len() as f64 };
    let avg_composite = if composite_gens.is_empty() { 0.0 }
        else { composite_gens.iter().map(|g| g.prime_density).sum::<f64>() / composite_gens.len() as f64 };

    let best_prime_gen = prime_gens.first().map(|g| g.generator);
    let best_composite_gen = composite_gens.first().map(|g| g.generator);
    let best_prime_d = prime_gens.first().map(|g| g.prime_density).unwrap_or(0.0);
    let best_composite_d = composite_gens.first().map(|g| g.prime_density).unwrap_or(0.0);

    let max_order_p = prime_gens.iter().filter(|g| g.order == profile.lambda).count() as u32;
    let max_order_c = composite_gens.iter().filter(|g| g.order == profile.lambda).count() as u32;
    let full_sat_p = prime_gens.iter().filter(|g| g.lanes_saturated == g.total_lanes).count() as u32;
    let full_sat_c = composite_gens.iter().filter(|g| g.lanes_saturated == g.total_lanes).count() as u32;

    CompetitionResult {
        modulus: profile,
        prime_generators: prime_gens,
        composite_generators: composite_gens,
        avg_prime_gen_density: avg_prime,
        avg_composite_gen_density: avg_composite,
        composite_advantage: if avg_prime > 0.0 { avg_composite / avg_prime } else { 0.0 },
        best_prime_gen,
        best_composite_gen,
        best_prime_density: best_prime_d,
        best_composite_density: best_composite_d,
        max_order_achievers_prime: max_order_p,
        max_order_achievers_composite: max_order_c,
        full_saturation_prime: full_sat_p,
        full_saturation_composite: full_sat_c,
    }
}

/// Run competition across many moduli to find where composites win.
pub fn modulus_sweep(moduli: &[u128], gen_limit: u128, winding_limit: u128) -> Vec<CompetitionResult> {
    moduli.iter()
        .map(|&m| compete_on_modulus(m, gen_limit, winding_limit))
        .collect()
}

/// Find "interesting" moduli: those with high λ/φ ratio (more orbit diversity)
/// and those with low λ/φ ratio (like CR=18980).
pub fn find_interesting_moduli(limit: u128) -> Vec<ModulusProfile> {
    let mut profiles = Vec::new();

    // Primes (λ = φ = p-1, ratio = 1.0 — maximum)
    for p in [29u128, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 97, 127, 211] {
        if is_prime(p) && p <= limit {
            profiles.push(ModulusProfile::new(p));
        }
    }

    // Prime powers (still have primitive roots)
    for &pp in &[49u128, 121, 169, 289, 343, 529] {
        if pp <= limit {
            profiles.push(ModulusProfile::new(pp));
        }
    }

    // Products of 2 primes (intermediate λ/φ)
    let small_primes = [3u128, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
    for i in 0..small_primes.len() {
        for j in (i+1)..small_primes.len() {
            let m = small_primes[i] * small_primes[j];
            if m <= limit {
                profiles.push(ModulusProfile::new(m));
            }
        }
    }

    // Products of 3+ primes (low λ/φ, like CR)
    for &m in &[30u128, 42, 66, 210, 330, 390, 462, 510, 546, 570, 2310,
                 819, 18980, 30030] {
        if m <= limit && m > 1 {
            profiles.push(ModulusProfile::new(m));
        }
    }

    // Sort by λ/φ ratio
    profiles.sort_by(|a, b| b.lambda_phi_ratio.partial_cmp(&a.lambda_phi_ratio).unwrap());
    profiles.dedup_by_key(|p| p.m);
    profiles
}

/// Ramanujan interaction mode analysis on a modulus.
/// For composite M = p₁ × p₂ × ... × pₖ, the "interaction modes" are
/// residue classes where multiple CRT coordinates are non-zero.
/// Theorem T3 claims these carry 98.8% of spectral power.
pub fn interaction_mode_analysis(m: u128, winding_limit: u128) -> InteractionReport {
    let factors = factorize(m);
    if factors.len() < 2 {
        return InteractionReport {
            modulus: m,
            total_residues: euler_totient(m) as u32,
            interaction_residues: 0,
            marginal_residues: 0,
            interaction_primes: 0,
            marginal_primes: 0,
            interaction_total: 0,
            marginal_total: 0,
            interaction_density: 0.0,
            marginal_density: 0.0,
            interaction_fraction: 0.0,
        };
    }

    let prime_factors: Vec<u128> = factors.iter().map(|&(p, _)| p).collect();
    let mut interaction_thetas = Vec::new();
    let mut marginal_thetas = Vec::new();

    for theta in 1..m {
        if gcd(theta, m) != 1 { continue; }

        // Count how many CRT lanes have non-zero residue
        // (they all should be non-zero since gcd=1, but check which are "active")
        let crt_coords: Vec<u128> = prime_factors.iter().map(|&p| theta % p).collect();
        let nonzero_count = crt_coords.iter().filter(|&&r| r != 0).count();

        if nonzero_count >= 2 {
            interaction_thetas.push(theta);
        } else {
            marginal_thetas.push(theta);
        }
    }

    // Count primes at interaction vs marginal positions
    let mut interaction_primes = 0u32;
    let mut interaction_total = 0u32;
    let mut marginal_primes = 0u32;
    let mut marginal_total = 0u32;

    for &theta in &interaction_thetas {
        for k in 0..winding_limit {
            let n = theta + k * m;
            if n < 2 { continue; }
            interaction_total += 1;
            if is_prime(n) {
                interaction_primes += 1;
            }
        }
    }

    for &theta in &marginal_thetas {
        for k in 0..winding_limit {
            let n = theta + k * m;
            if n < 2 { continue; }
            marginal_total += 1;
            if is_prime(n) {
                marginal_primes += 1;
            }
        }
    }

    InteractionReport {
        modulus: m,
        total_residues: (interaction_thetas.len() + marginal_thetas.len()) as u32,
        interaction_residues: interaction_thetas.len() as u32,
        marginal_residues: marginal_thetas.len() as u32,
        interaction_primes,
        marginal_primes,
        interaction_total,
        marginal_total,
        interaction_density: if interaction_total > 0 {
            interaction_primes as f64 / interaction_total as f64
        } else { 0.0 },
        marginal_density: if marginal_total > 0 {
            marginal_primes as f64 / marginal_total as f64
        } else { 0.0 },
        interaction_fraction: interaction_thetas.len() as f64
            / (interaction_thetas.len() + marginal_thetas.len()) as f64,
    }
}

#[derive(Debug)]
pub struct InteractionReport {
    pub modulus: u128,
    pub total_residues: u32,
    pub interaction_residues: u32,
    pub marginal_residues: u32,
    pub interaction_primes: u32,
    pub marginal_primes: u32,
    pub interaction_total: u32,
    pub marginal_total: u32,
    pub interaction_density: f64,
    pub marginal_density: f64,
    pub interaction_fraction: f64,
}

/// The order-density correlation test.
/// For a given modulus, bin generators by their order and compare prime
/// densities. This is the definitive test of whether orbit structure
/// predicts prime concentration.
pub fn order_density_correlation(
    m: u128,
    gen_limit: u128,
    winding_limit: u128,
) -> OrderDensityReport {
    let lambda = carmichael_lambda(m);
    let divs = orbits::divisors_of(lambda);

    let mut bins: std::collections::HashMap<u128, (Vec<f64>, Vec<f64>)> = std::collections::HashMap::new();
    // bins[order] = (prime_gen_densities, composite_gen_densities)

    for d in &divs {
        bins.insert(*d, (Vec::new(), Vec::new()));
    }

    let sub_moduli: Vec<u128> = factorize(m).iter()
        .map(|&(p, a)| (0..a).fold(1u128, |acc, _| acc * p))
        .collect();
    let sub_lambdas: Vec<u128> = sub_moduli.iter()
        .map(|&sm| carmichael_lambda(sm))
        .collect();

    for g in 2..=gen_limit {
        if gcd(g, m) != 1 { continue; }
        let is_p = is_prime(g);
        let g_factors = factorize(g);
        let is_cc = !is_p && g > 1 && g_factors.iter().all(|&(p, _)| p > 13);
        if !is_p && !is_cc { continue; }

        let ord = match multiplicative_order(g, m) {
            Some(o) => o,
            None => continue,
        };

        // Compute density
        let mut primes = 0u32;
        let mut total = 0u32;
        let mut current = 1u128;
        for _ in 0..ord {
            current = crt::mod_mul(current, g % m, m);
            for k in 0..winding_limit {
                let n = current + k * m;
                if n < 2 { continue; }
                total += 1;
                if is_prime(n) { primes += 1; }
            }
        }
        let density = if total > 0 { primes as f64 / total as f64 } else { 0.0 };

        let entry = bins.entry(ord).or_insert((Vec::new(), Vec::new()));
        if is_p {
            entry.0.push(density);
        } else {
            entry.1.push(density);
        }
    }

    let mut order_bins: Vec<OrderBin> = bins.into_iter()
        .filter(|(_, (p, c))| !p.is_empty() || !c.is_empty())
        .map(|(order, (prime_d, comp_d))| {
            let avg_p = if prime_d.is_empty() { 0.0 }
                else { prime_d.iter().sum::<f64>() / prime_d.len() as f64 };
            let avg_c = if comp_d.is_empty() { 0.0 }
                else { comp_d.iter().sum::<f64>() / comp_d.len() as f64 };
            OrderBin {
                order,
                prime_count: prime_d.len() as u32,
                composite_count: comp_d.len() as u32,
                avg_prime_density: avg_p,
                avg_composite_density: avg_c,
                composite_advantage: if avg_p > 0.0 { avg_c / avg_p } else { 0.0 },
            }
        })
        .collect();
    order_bins.sort_by_key(|b| b.order);

    OrderDensityReport {
        modulus: m,
        lambda,
        order_bins,
    }
}

#[derive(Debug)]
pub struct OrderDensityReport {
    pub modulus: u128,
    pub lambda: u128,
    pub order_bins: Vec<OrderBin>,
}

#[derive(Debug)]
pub struct OrderBin {
    pub order: u128,
    pub prime_count: u32,
    pub composite_count: u32,
    pub avg_prime_density: f64,
    pub avg_composite_density: f64,
    pub composite_advantage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulus_profile() {
        let p = ModulusProfile::new(97);
        assert_eq!(p.phi, 96);
        assert_eq!(p.lambda, 96);
        assert!((p.lambda_phi_ratio - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_modulus_profile_composite() {
        let p = ModulusProfile::new(30);
        // 30 = 2×3×5, φ=8, λ=lcm(1,2,4)=4
        assert_eq!(p.phi, 8);
        assert_eq!(p.lambda, 4);
        assert!((p.lambda_phi_ratio - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_competition_small() {
        let result = compete_on_modulus(30, 50, 10);
        assert!(!result.prime_generators.is_empty());
        // There should be both prime and composite generators coprime to 30
    }

    #[test]
    fn test_interaction_modes() {
        let report = interaction_mode_analysis(30, 10);
        // 30 = 2×3×5, φ(30) = 8
        assert_eq!(report.total_residues, 8);
        // All coprime residues mod 30 have non-zero coords mod 2,3,5
        // (since gcd=1 means no coordinate is 0)
        // So ALL are interaction modes, ZERO are marginal
        assert_eq!(report.interaction_residues, 8);
        assert_eq!(report.marginal_residues, 0);
    }
}
