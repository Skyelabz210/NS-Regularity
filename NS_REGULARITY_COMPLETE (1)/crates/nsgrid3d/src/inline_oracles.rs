//! # Inline Oracles + One-Shot CRT Learner — ZERO FLOAT
//! Every value is a scaled integer. No f64 in any computation path.

use crate::schema_program::StepMeasurements;

pub const ORACLE_SCALE: i64 = 10_000;
pub const ORACLE_SCALE_128: i128 = ORACLE_SCALE as i128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellVerdict { Safe, Elevated, Critical, Excluded }

#[derive(Debug, Clone)]
pub struct OracleA {
    pub r_elevate: i64,
    pub r_critical: i64,
    pub stretch_ratio_elevate: i64,
}

impl OracleA {
    pub fn default_config() -> Self {
        Self { r_elevate: 100, r_critical: 1000, stretch_ratio_elevate: 20_000 }
    }

    #[inline]
    pub fn classify_cell(&self, _vort_abs: i64, stretch_abs: i64, diff_abs: i64, cap_frac: i64) -> CellVerdict {
        if cap_frac > self.r_critical { return CellVerdict::Critical; }
        if diff_abs > 0 && stretch_abs > 0 {
            let ratio = (stretch_abs as i128 * ORACLE_SCALE_128) / diff_abs.max(1) as i128;
            if ratio > self.stretch_ratio_elevate as i128 {
                return if cap_frac > self.r_elevate { CellVerdict::Critical } else { CellVerdict::Elevated };
            }
        }
        if cap_frac > self.r_elevate { return CellVerdict::Elevated; }
        CellVerdict::Safe
    }
}

#[derive(Debug, Clone, Default)]
pub struct OracleAStats {
    pub safe_count: u64, pub elevated_count: u64,
    pub critical_count: u64, pub excluded_count: u64, pub total_cells: u64,
}

impl OracleAStats {
    #[inline]
    pub fn record(&mut self, v: CellVerdict) {
        match v { CellVerdict::Safe => self.safe_count += 1, CellVerdict::Elevated => self.elevated_count += 1,
                  CellVerdict::Critical => self.critical_count += 1, CellVerdict::Excluded => self.excluded_count += 1 }
        self.total_cells += 1;
    }
    pub fn worst_verdict(&self) -> CellVerdict {
        if self.excluded_count > 0 { CellVerdict::Excluded }
        else if self.critical_count > 0 { CellVerdict::Critical }
        else if self.elevated_count > 0 { CellVerdict::Elevated }
        else { CellVerdict::Safe }
    }
    pub fn stress_fraction_scaled(&self) -> i64 {
        if self.total_cells == 0 { return 0; }
        let s = self.elevated_count + self.critical_count + self.excluded_count;
        ((s as i128 * ORACLE_SCALE_128) / self.total_cells as i128) as i64
    }
}

#[derive(Debug, Clone)]
pub struct OracleB { pub w_c_eff: i64, pub w_bkm: i64, pub w_div: i64, pub w_winding: i64, pub w_oracle_a: i64 }

impl OracleB {
    pub fn default_weights() -> Self { Self { w_c_eff: 3, w_bkm: 2, w_div: 1, w_winding: 1, w_oracle_a: 2 } }

    pub fn score_step(&self, meas: &StepMeasurements, oa: &OracleAStats, prev_bkm: i64) -> (i64, FeatureVector) {
        let features = FeatureVector {
            c_eff_scaled: meas.c_eff_scaled, bkm_sum_scaled: prev_bkm + meas.bkm_increment_scaled,
            div_defect: meas.div_defect, winding_stress: meas.winding_stress,
            stress_fraction_scaled: oa.stress_fraction_scaled(), rn_scaled: meas.rn_scaled,
        };
        let tw = (self.w_c_eff + self.w_bkm + self.w_div + self.w_winding + self.w_oracle_a) as i128;
        let sc = |ok: bool, fallback_denom: i128| -> i64 {
            if ok { ORACLE_SCALE } else { ((ORACLE_SCALE_128 * ORACLE_SCALE_128) / fallback_denom) as i64 }
        };
        let s1 = if features.c_eff_scaled <= 0 { ORACLE_SCALE }
                 else { ((ORACLE_SCALE_128*ORACLE_SCALE_128) / (ORACLE_SCALE_128 + features.c_eff_scaled.abs() as i128)) as i64 };
        let s2 = ((ORACLE_SCALE_128*ORACLE_SCALE_128*10) / (10*ORACLE_SCALE_128 + features.bkm_sum_scaled.abs() as i128)) as i64;
        let s3 = if features.div_defect == 0 { ORACLE_SCALE }
                 else { ((ORACLE_SCALE_128*ORACLE_SCALE_128) / (ORACLE_SCALE_128 + features.div_defect.abs() as i128)) as i64 };
        let s4 = ((ORACLE_SCALE_128*ORACLE_SCALE_128) / (ORACLE_SCALE_128 + features.winding_stress.abs() as i128)) as i64;
        let s5 = ORACLE_SCALE - features.stress_fraction_scaled;
        let score = ((self.w_c_eff as i128*s1 as i128 + self.w_bkm as i128*s2 as i128 + self.w_div as i128*s3 as i128
                    + self.w_winding as i128*s4 as i128 + self.w_oracle_a as i128*s5 as i128) / tw) as i64;
        (score, features)
    }
}

#[derive(Debug, Clone)]
pub struct FeatureVector {
    pub c_eff_scaled: i64, pub bkm_sum_scaled: i64, pub div_defect: i64,
    pub winding_stress: i64, pub stress_fraction_scaled: i64, pub rn_scaled: i64,
}

impl FeatureVector {
    pub fn to_crt_residues(&self) -> CRTResidues {
        let v = [self.c_eff_scaled.unsigned_abs(), self.bkm_sum_scaled.unsigned_abs(),
                 self.div_defect.unsigned_abs(), self.winding_stress.unsigned_abs(),
                 self.stress_fraction_scaled.unsigned_abs(), self.rn_scaled.unsigned_abs()];
        CRTResidues { channel_a: v.map(|x| (x%127) as u16), channel_b: v.map(|x| (x%131) as u16), channel_c: v.map(|x| (x%137) as u16) }
    }
}

#[derive(Debug, Clone)]
pub struct CRTResidues { pub channel_a: [u16;6], pub channel_b: [u16;6], pub channel_c: [u16;6] }

#[derive(Debug, Clone)]
pub struct OneShotLearner {
    pa: Vec<[i64;6]>, pb: Vec<[i64;6]>, pc: Vec<[i64;6]>, counts: Vec<u64>, pub n_classes: usize,
}

impl OneShotLearner {
    pub fn new() -> Self { Self { pa: vec![[0;6];2], pb: vec![[0;6];2], pc: vec![[0;6];2], counts: vec![0;2], n_classes: 2 } }
    pub fn learn(&mut self, f: &FeatureVector, label: usize) {
        let r = f.to_crt_residues();
        for i in 0..6 { self.pa[label][i] += r.channel_a[i] as i64; self.pb[label][i] += r.channel_b[i] as i64; self.pc[label][i] += r.channel_c[i] as i64; }
        self.counts[label] += 1;
    }
    pub fn classify(&self, f: &FeatureVector) -> (usize, u32) {
        if self.counts.iter().all(|&c| c==0) { return (0,0); }
        let r = f.to_crt_residues();
        let va = self.vote(&r.channel_a, &self.pa, 127);
        let vb = self.vote(&r.channel_b, &self.pb, 131);
        let vc = self.vote(&r.channel_c, &self.pc, 137);
        let mut t = vec![0u32; self.n_classes]; t[va]+=1; t[vb]+=1; t[vc]+=1;
        let (p, &mv) = t.iter().enumerate().max_by_key(|(_,&v)| v).unwrap();
        (p, mv)
    }
    fn vote(&self, q: &[u16;6], protos: &[[i64;6]], m: u64) -> usize {
        let mut bc = 0; let mut bd = i64::MAX;
        for (c, p) in protos.iter().enumerate() {
            if self.counts[c]==0 { continue; }
            let mut d: i64 = 0;
            for i in 0..6 { let pa = ((p[i]/self.counts[c] as i64).unsigned_abs())%m; d += circ_dist(q[i] as u64, pa, m) as i64; }
            if d < bd { bd = d; bc = c; }
        }
        bc
    }
    pub fn is_trained(&self) -> bool { self.counts.iter().all(|&c| c>0) }
    pub fn example_count(&self, c: usize) -> u64 { self.counts.get(c).copied().unwrap_or(0) }
}

fn circ_dist(a: u64, b: u64, m: u64) -> u64 { let d = if a>=b { a-b } else { b-a }; d.min(m-d) }

#[derive(Debug, Clone)]
pub struct InlineOracles {
    pub oracle_a: OracleA, pub oracle_b: OracleB, pub learner: OneShotLearner,
    pub oracle_a_stats: OracleAStats, pub bkm_running_sum_scaled: i64,
    pub step_scores: Vec<i64>, pub step_verdicts: Vec<(usize, u32)>,
}

impl InlineOracles {
    pub fn new() -> Self {
        Self { oracle_a: OracleA::default_config(), oracle_b: OracleB::default_weights(),
               learner: OneShotLearner::new(), oracle_a_stats: OracleAStats::default(),
               bkm_running_sum_scaled: 0, step_scores: Vec::new(), step_verdicts: Vec::new() }
    }
    pub fn begin_step(&mut self) { self.oracle_a_stats = OracleAStats::default(); }
    #[inline]
    pub fn classify_cell(&mut self, va: i64, sa: i64, da: i64, cf: i64) -> CellVerdict {
        let v = self.oracle_a.classify_cell(va, sa, da, cf); self.oracle_a_stats.record(v); v
    }
    pub fn end_step(&mut self, meas: &StepMeasurements) {
        let (score, features) = self.oracle_b.score_step(meas, &self.oracle_a_stats, self.bkm_running_sum_scaled);
        self.bkm_running_sum_scaled += meas.bkm_increment_scaled;
        self.step_scores.push(score);
        if score > 7000 { self.learner.learn(&features, 0); }
        else if score < 3000 { self.learner.learn(&features, 1); }
        let v = if self.learner.is_trained() { self.learner.classify(&features) } else { (0,0) };
        self.step_verdicts.push(v);
    }
    pub fn summary(&self) -> OracleSummary {
        let n = self.step_scores.len();
        let avg = if n > 0 { (self.step_scores.iter().map(|&s| s as i128).sum::<i128>() / n as i128) as i64 } else { 0 };
        OracleSummary {
            total_steps: n,
            avg_score_scaled: avg,
            regularity_verdicts: self.step_verdicts.iter().filter(|(c,v)| *c == 0 && *v >= 2).count(),
            blowup_verdicts: self.step_verdicts.iter().filter(|(c,v)| *c == 1 && *v >= 2).count(),
            learner_trained: self.learner.is_trained(),
            final_oracle_a: self.oracle_a_stats.worst_verdict(),
        }
    }
}

#[derive(Debug)]
pub struct OracleSummary {
    pub total_steps: usize,
    pub avg_score_scaled: i64,
    pub regularity_verdicts: usize,
    pub blowup_verdicts: usize,
    pub learner_trained: bool,
    pub final_oracle_a: CellVerdict,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_oracle_a_safe() { assert_eq!(OracleA::default_config().classify_cell(100,10,50,0), CellVerdict::Safe); }
    #[test] fn test_oracle_a_critical() { assert_eq!(OracleA::default_config().classify_cell(100,10,50,2000), CellVerdict::Critical); }
    #[test] fn test_oracle_b_integer() {
        let ob = OracleB::default_weights();
        let m = StepMeasurements { m_t: 9000, rn_scaled: 1, bkm_increment_scaled: 90, c_eff_scaled: -10, winding_stress: 0, div_defect: 0, lock_score_scaled: 0 };
        let s = OracleAStats { safe_count: 512, total_cells: 512, ..Default::default() };
        let (score, _) = ob.score_step(&m, &s, 0);
        assert!(score > 8000, "Decaying: {}", score);
    }
    #[test] fn test_learner_integer() {
        let mut l = OneShotLearner::new();
        l.learn(&FeatureVector { c_eff_scaled: -10, bkm_sum_scaled: 500, div_defect: 0, winding_stress: 0, stress_fraction_scaled: 0, rn_scaled: 1 }, 0);
        l.learn(&FeatureVector { c_eff_scaled: 1_000_000, bkm_sum_scaled: 500_000, div_defect: 5000, winding_stress: 9000, stress_fraction_scaled: 8000, rn_scaled: 5000 }, 1);
        assert!(l.is_trained());
        let (c, v) = l.classify(&FeatureVector { c_eff_scaled: -5, bkm_sum_scaled: 300, div_defect: 0, winding_stress: 0, stress_fraction_scaled: 0, rn_scaled: 1 });
        assert_eq!(c, 0); assert!(v >= 2);
    }
    #[test] fn test_circ_dist() { assert_eq!(circ_dist(1, 126, 127), 2); }
    #[test] fn test_harness() {
        let mut o = InlineOracles::new();
        for _ in 0..5 { o.begin_step(); for i in 0..100 { o.classify_cell(9000-i*10,50,200,0); }
            o.end_step(&StepMeasurements { m_t: 9000, rn_scaled: 1, bkm_increment_scaled: 90, c_eff_scaled: -10, winding_stress: 0, div_defect: 0, lock_score_scaled: 0 }); }
        assert_eq!(o.step_scores.len(), 5);
        assert!(*o.step_scores.last().unwrap() > 7000);
    }
}
