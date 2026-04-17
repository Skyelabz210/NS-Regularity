//! # ARCHIVED CRAM-Native 1D NS Regression Harness — Zero Float
//!
//! This module is retained only for regression testing of arithmetic,
//! demotion, overflow tracking, and K-Elimination behavior. It is NOT
//! part of the active 3D proof path and is compiled only under #[cfg(test)].
//!
//! Active theorem-bearing work is 3D-only. Keep this file out of agent
//! planning unless the task is explicitly regression or debugging.
//!
//! Additional guardrail: this harness reconstructs through a simplified path
//! suitable for fast checks, not for final theorem-bearing 3D analysis.

use qmnf_crtbigint::EXACT_MODULI;

pub const SCALE: i64 = 10_000;
pub const SCALE_SQ: i128 = (SCALE as i128) * (SCALE as i128);
pub const DT: i64 = 100;
pub const NU: i64 = 6;
pub const NUM_LANES: usize = 8;

fn mulmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 * b as u128) % m as u128) as u64 }
fn addmod(a: u64, b: u64, m: u64) -> u64 { ((a as u128 + b as u128) % m as u128) as u64 }
fn submod(a: u64, b: u64, m: u64) -> u64 { if a >= b { a - b } else { m - (b - a) } }

#[derive(Clone)]
pub struct VortCell { pub res: [u64; NUM_LANES] }

impl VortCell {
    pub fn from_signed(val: i64) -> Self {
        let mut res = [0u64; NUM_LANES];
        for i in 0..NUM_LANES {
            let p = EXACT_MODULI[i];
            res[i] = if val >= 0 { (val as u64) % p } else { p - ((-val) as u64 % p) };
        }
        VortCell { res }
    }

    pub fn to_signed(&self) -> i64 {
        let p0 = EXACT_MODULI[0];
        let r0 = self.res[0];
        if r0 <= p0 / 2 { r0 as i64 } else { r0 as i64 - p0 as i64 }
    }

    pub fn add(&self, other: &VortCell) -> VortCell {
        let mut r = [0u64; NUM_LANES];
        for i in 0..NUM_LANES { r[i] = addmod(self.res[i], other.res[i], EXACT_MODULI[i]); }
        VortCell { res: r }
    }

    pub fn sub(&self, other: &VortCell) -> VortCell {
        let mut r = [0u64; NUM_LANES];
        for i in 0..NUM_LANES { r[i] = submod(self.res[i], other.res[i], EXACT_MODULI[i]); }
        VortCell { res: r }
    }

    pub fn mul(&self, other: &VortCell) -> VortCell {
        let mut r = [0u64; NUM_LANES];
        for i in 0..NUM_LANES { r[i] = mulmod(self.res[i], other.res[i], EXACT_MODULI[i]); }
        VortCell { res: r }
    }

    /// K-Elim division via a deliberately simplified archived decode path.
    ///
    /// This is acceptable for regression because the purpose is to stress the
    /// arithmetic transitions, not to certify the active 3D solver. The active
    /// path should continue to use the full recumbent/CRT stack instead.
    pub fn k_div(&self, d: i64) -> VortCell {
        let v = self.to_signed();
        let q = if (v >= 0) == (d > 0) { v.abs() / d.abs() } else { -(v.abs() / d.abs()) };
        VortCell::from_signed(q)
    }
}

/// One NS step: archived port of the 1D regression harness.
///
/// Important: this function is intentionally *not* a theorem-bearing 3D update.
/// It exists only so arithmetic regressions can be caught in a tiny, cheap loop.
pub fn archived_ns_step(omega: &[VortCell], n: usize) -> Vec<VortCell> {
    let n_i = n as i64;
    let nu_n2 = VortCell::from_signed(NU * n_i * n_i);
    let dt_cell = VortCell::from_signed(DT);
    let two = VortCell::from_signed(2);

    (0..n).map(|j| {
        let jp = (j + 1) % n;
        let jm = (j + n - 1) % n;

        let lap = omega[jp].add(&omega[jm]).sub(&omega[j].mul(&two));
        let diff_num = nu_n2.mul(&lap);
        let diffusion = diff_num.k_div(SCALE).k_div(SCALE);

        let grad = omega[jp].sub(&omega[jm]);
        let n_cell = VortCell::from_signed(n_i);
        let stretch_num = omega[j].mul(&n_cell).mul(&grad);
        let stretching = stretch_num.k_div(2 * SCALE);

        let f_total = diffusion.add(&stretching);
        let delta = dt_cell.mul(&f_total).k_div(SCALE);
        omega[j].add(&delta)
    }).collect()
}

/// Measure overshoot with CRTBigInt lanes for the archived 1D regression harness.
/// Returns `(initial_max, peak_max, final_max, peak_step)`.
///
/// The data from this function should be treated as regression telemetry only.
pub fn archived_measure_overshoot(n: usize, steps: usize) -> (i64, i64, i64, usize) {
    let mut omega: Vec<VortCell> = (0..n).map(|i| {
        VortCell::from_signed(qmnf_crtbigint::integer_sine(i, n, SCALE))
    }).collect();
    let ini = omega.iter().map(|c| c.to_signed().abs()).max().unwrap();
    let mut pk = ini;
    let mut pk_step = 0usize;
    for step in 0..steps {
        omega = archived_ns_step(&omega, n);
        let cm = omega.iter().map(|c| c.to_signed().abs()).max().unwrap();
        if cm > pk { pk = cm; pk_step = step + 1; }
    }
    let fin = omega.iter().map(|c| c.to_signed().abs()).max().unwrap();
    (ini, pk, fin, pk_step)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for v in [-10000i64, -1, 0, 1, 10000] {
            assert_eq!(VortCell::from_signed(v).to_signed(), v);
        }
    }

    #[test]
    fn sub_works() {
        assert_eq!(VortCell::from_signed(3).sub(&VortCell::from_signed(5)).to_signed(), -2);
    }

    #[test]
    fn k_div_works() {
        assert_eq!(VortCell::from_signed(42).k_div(6).to_signed(), 7);
        assert_eq!(VortCell::from_signed(-30).k_div(6).to_signed(), -5);
    }

    #[test]
    fn constant_preserved() {
        let omega: Vec<VortCell> = (0..8).map(|_| VortCell::from_signed(600)).collect();
        let r = archived_ns_step(&omega, 8);
        for c in &r { assert_eq!(c.to_signed(), 600); }
    }

    #[test]
    fn theorem_e_scaling() {
        // Archived 1D regression evidence only.
        println!("\n{}", "=".repeat(75));
        println!("  ARCHIVED 1D REGRESSION: nsgrid3d-exact 1D, CRTBigInt K-Elim division");
        println!("  dt=100 nu=6 SCALE=10000 (exact port of 3D solver constants)");
        println!("{}\n", "=".repeat(75));
        println!("{:>5} {:>8} {:>8} {:>8} {:>6} {:>8}",
            "N", "init", "peak", "final", "pk_st", "over%");
        println!("{}", "-".repeat(52));

        for &n in &[8usize, 16, 32, 64, 128, 256] {
            let (ini, pk, fin, pks) = archived_measure_overshoot(n, 500);
            let over_pct = if ini > 0 { pk * 100 / ini } else { 0 };
            println!("{:>5} {:>8} {:>8} {:>8} {:>6} {:>7}%", n, ini, pk, fin, pks, over_pct);
        }
    }
}
