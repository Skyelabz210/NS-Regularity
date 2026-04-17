//! Exact Rational Arithmetic — Zero Float Density Computations
//!
//! Every density, ratio, comparison, and threshold in the Hydra
//! is an exact (numerator, denominator) pair. Cross-multiply to compare.
//! No f64 anywhere in the production path.
//!
//! "Truth cannot be approximated. But approximation can be exactly absorbed."

use crate::crt;

/// Exact rational: (numerator, denominator), always in lowest terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    pub num: u64,
    pub den: u64,
}

impl Rational {
    pub fn new(num: u64, den: u64) -> Self {
        if den == 0 { return Self { num: 0, den: 1 }; }
        let g = crt::gcd(num as u128, den as u128) as u64;
        Self { num: num / g, den: den / g }
    }

    pub fn zero() -> Self { Self { num: 0, den: 1 } }
    pub fn one() -> Self { Self { num: 1, den: 1 } }

    /// Compare: self > other ⟺ self.num * other.den > other.num * self.den
    pub fn gt(&self, other: &Self) -> bool {
        (self.num as u128) * (other.den as u128) > (other.num as u128) * (self.den as u128)
    }

    pub fn lt(&self, other: &Self) -> bool { other.gt(self) }

    pub fn ge(&self, other: &Self) -> bool { !self.lt(other) }

    /// Multiply: (a/b) × (c/d) = (a×c)/(b×d) reduced
    pub fn mul(&self, other: &Self) -> Self {
        let n = (self.num as u128) * (other.num as u128);
        let d = (self.den as u128) * (other.den as u128);
        let g = crt::gcd(n, d);
        Self { num: (n / g) as u64, den: (d / g) as u64 }
    }

    /// Add: (a/b) + (c/d) = (a×d + c×b)/(b×d) reduced
    pub fn add(&self, other: &Self) -> Self {
        let n = (self.num as u128) * (other.den as u128)
              + (other.num as u128) * (self.den as u128);
        let d = (self.den as u128) * (other.den as u128);
        let g = crt::gcd(n, d);
        Self { num: (n / g) as u64, den: (d / g) as u64 }
    }

    /// Subtract (saturating at 0): max(a/b - c/d, 0)
    pub fn sub_saturating(&self, other: &Self) -> Self {
        let lhs = (self.num as u128) * (other.den as u128);
        let rhs = (other.num as u128) * (self.den as u128);
        if lhs <= rhs { return Self::zero(); }
        let n = lhs - rhs;
        let d = (self.den as u128) * (other.den as u128);
        let g = crt::gcd(n, d);
        Self { num: (n / g) as u64, den: (d / g) as u64 }
    }

    /// Integer permille: (num × 1000) / den (truncated)
    pub fn permille(&self) -> u64 {
        if self.den == 0 { return 0; }
        (self.num as u128 * 1000 / self.den as u128) as u64
    }

    /// Display as "num/den" for exact reporting
    pub fn display(&self) -> String {
        if self.den == 1 { format!("{}", self.num) }
        else { format!("{}/{}", self.num, self.den) }
    }

    /// Display as permille string "XXX‰"
    pub fn display_permille(&self) -> String {
        format!("{}‰", self.permille())
    }
}

impl std::fmt::Display for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.num, self.den)
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let lhs = (self.num as u128) * (other.den as u128);
        let rhs = (other.num as u128) * (self.den as u128);
        lhs.cmp(&rhs)
    }
}

impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Exact Gini coefficient as a rational.
/// Gini = Σ|xᵢ - xⱼ| / (2 × n² × mean)
/// All numerators and denominators tracked exactly.
pub fn exact_gini(densities: &[Rational]) -> Rational {
    if densities.is_empty() { return Rational::zero(); }
    let n = densities.len() as u64;

    // Compute mean = sum / n
    let mut sum = Rational::zero();
    for d in densities {
        sum = sum.add(d);
    }
    // mean = sum / n → mean.num = sum.num, mean.den = sum.den * n
    let mean = Rational::new(sum.num, sum.den * n);
    if mean.num == 0 { return Rational::zero(); }

    // Compute sum of |xᵢ - xⱼ| for all pairs
    let mut abs_diff_sum = Rational::zero();
    for i in 0..densities.len() {
        for j in 0..densities.len() {
            let diff = if densities[i].gt(&densities[j]) {
                densities[i].sub_saturating(&densities[j])
            } else {
                densities[j].sub_saturating(&densities[i])
            };
            abs_diff_sum = abs_diff_sum.add(&diff);
        }
    }

    // Gini = abs_diff_sum / (2 × n² × mean)
    // = abs_diff_sum.num / abs_diff_sum.den   ÷   (2 × n² × mean.num / mean.den)
    // = abs_diff_sum.num × mean.den   /   (abs_diff_sum.den × 2 × n² × mean.num)
    let gini_num = (abs_diff_sum.num as u128) * (mean.den as u128);
    let gini_den = (abs_diff_sum.den as u128) * 2 * (n as u128) * (n as u128) * (mean.num as u128);

    if gini_den == 0 { return Rational::zero(); }
    let g = crt::gcd(gini_num, gini_den);
    Rational { num: (gini_num / g) as u64, den: (gini_den / g) as u64 }
}

/// Exact comparison: is density_a > density_b × (threshold_num/threshold_den)?
/// All integer cross-multiplication, no floats.
pub fn exceeds_threshold(density: &Rational, threshold: &Rational) -> bool {
    density.gt(threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rational_compare() {
        let a = Rational::new(2, 3); // 2/3
        let b = Rational::new(3, 5); // 3/5
        assert!(a.gt(&b)); // 2/3 > 3/5 ⟺ 10 > 9 ✓
    }

    #[test]
    fn test_rational_add() {
        let a = Rational::new(1, 3);
        let b = Rational::new(1, 6);
        let c = a.add(&b); // 1/3 + 1/6 = 1/2
        assert_eq!(c, Rational::new(1, 2));
    }

    #[test]
    fn test_rational_mul() {
        let a = Rational::new(2, 3);
        let b = Rational::new(3, 4);
        let c = a.mul(&b); // 2/3 × 3/4 = 1/2
        assert_eq!(c, Rational::new(1, 2));
    }

    #[test]
    fn test_rational_sub_saturating() {
        let a = Rational::new(3, 4);
        let b = Rational::new(1, 4);
        let c = a.sub_saturating(&b); // 3/4 - 1/4 = 1/2
        assert_eq!(c, Rational::new(1, 2));

        let d = b.sub_saturating(&a); // 1/4 - 3/4 = 0 (saturated)
        assert_eq!(d, Rational::zero());
    }

    #[test]
    fn test_rational_permille() {
        let a = Rational::new(1, 3);
        assert_eq!(a.permille(), 333); // 333‰ = 33.3%
        let b = Rational::new(1, 2);
        assert_eq!(b.permille(), 500);
    }

    #[test]
    fn test_exact_gini_uniform() {
        // All same density → Gini = 0
        let densities = vec![Rational::new(1, 4); 5];
        let g = exact_gini(&densities);
        assert_eq!(g, Rational::zero());
    }

    #[test]
    fn test_exact_gini_nonuniform() {
        // One stratum has all primes, rest have none → Gini > 0
        let densities = vec![
            Rational::new(1, 1), // 100%
            Rational::zero(),
            Rational::zero(),
            Rational::zero(),
        ];
        let g = exact_gini(&densities);
        assert!(g.gt(&Rational::zero()));
    }

    #[test]
    fn test_ordering() {
        let mut rats = vec![
            Rational::new(3, 7),
            Rational::new(1, 2),
            Rational::new(2, 5),
        ];
        rats.sort();
        // 2/5 < 3/7 < 1/2
        assert_eq!(rats[0], Rational::new(2, 5));
        assert_eq!(rats[1], Rational::new(3, 7));
        assert_eq!(rats[2], Rational::new(1, 2));
    }
}
