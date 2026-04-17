//! # CDHS Telemetry — W2.3
//!
//! Continuous Diagnostic Health System: monitors Hardware Axioms H1-H4
//! and computes threat vector for Loki adaptive security.
//!
//! ## Hardware Axioms Monitored
//!
//! - **H1 (Temporal Isolation)**: CPU core affinity, IRQ distribution
//! - **H2 (Cache Isolation)**: resctrl partition status
//! - **H3 (Speculation Shield)**: Vulnerability mitigation status
//! - **H4 (Thermal Stability)**: CPU frequency, governor, turbo status
//!
//! ## Threat Vector
//!
//! θ = weighted sum of axiom violation scores:
//!   θ = w₁·v₁ + w₂·v₂ + w₃·v₃ + w₄·v₄
//!
//! where vᵢ ∈ [0, 10000] is the violation severity for axiom Hᵢ.
//! θ ∈ [0, 10000]. Higher = more threatened.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

use std::fmt;

// ============================================================================
// AXIOM STATUS
// ============================================================================

/// Violation severity: 0 = fully compliant, 10000 = fully violated.
pub type ViolationScore = u64;

/// Status of a single hardware axiom.
#[derive(Clone, Debug)]
pub struct AxiomStatus {
    pub name: &'static str,
    pub description: &'static str,
    pub violation: ViolationScore,
    pub details: String,
}

impl AxiomStatus {
    fn compliant(name: &'static str, desc: &'static str, details: String) -> Self {
        Self { name, description: desc, violation: 0, details }
    }
    fn violated(name: &'static str, desc: &'static str, score: ViolationScore, details: String) -> Self {
        Self { name, description: desc, violation: score, details }
    }
    fn unknown(name: &'static str, desc: &'static str) -> Self {
        Self { name, description: desc, violation: 5000, details: "Unable to read status".into() }
    }
}

// ============================================================================
// HARDWARE PROBES
// ============================================================================

/// Read H1: Temporal Isolation (CPU core affinity).
pub fn probe_h1() -> AxiomStatus {
    let name = "H1";
    let desc = "Temporal Isolation";

    // Check if isolcpus is set
    match std::fs::read_to_string("/proc/cmdline") {
        Ok(cmdline) => {
            if cmdline.contains("isolcpus=") {
                let nohz = cmdline.contains("nohz_full=");
                let rcu = cmdline.contains("rcu_nocbs=");
                let details = format!(
                    "isolcpus=present, nohz_full={}, rcu_nocbs={}",
                    nohz, rcu
                );
                if nohz && rcu {
                    AxiomStatus::compliant(name, desc, details)
                } else {
                    AxiomStatus::violated(name, desc, 3000, details)
                }
            } else {
                AxiomStatus::violated(name, desc, 8000,
                    "isolcpus not set in kernel cmdline".into())
            }
        }
        Err(_) => AxiomStatus::unknown(name, desc),
    }
}

/// Read H2: Cache Isolation (resctrl partitioning).
pub fn probe_h2() -> AxiomStatus {
    let name = "H2";
    let desc = "Cache Isolation";

    if std::path::Path::new("/sys/fs/resctrl/hackfate").exists() {
        match std::fs::read_to_string("/sys/fs/resctrl/hackfate/schemata") {
            Ok(schema) => AxiomStatus::compliant(name, desc,
                format!("HackFate cache partition active: {}", schema.trim())),
            Err(_) => AxiomStatus::violated(name, desc, 2000,
                "HackFate partition exists but schemata unreadable".into()),
        }
    } else if std::path::Path::new("/sys/fs/resctrl").exists() {
        AxiomStatus::violated(name, desc, 6000,
            "resctrl available but no HackFate partition".into())
    } else {
        AxiomStatus::violated(name, desc, 5000,
            "resctrl not available (no Intel CAT/AMD QoS)".into())
    }
}

/// Read H3: Speculation Shield (vulnerability mitigations).
pub fn probe_h3() -> AxiomStatus {
    let name = "H3";
    let desc = "Speculation Shield";
    let vuln_dir = "/sys/devices/system/cpu/vulnerabilities";

    if !std::path::Path::new(vuln_dir).exists() {
        return AxiomStatus::unknown(name, desc);
    }

    let mut total = 0u32;
    let mut mitigated = 0u32;
    let mut unmitigated = Vec::new();

    if let Ok(entries) = std::fs::read_dir(vuln_dir) {
        for entry in entries.flatten() {
            total += 1;
            if let Ok(status) = std::fs::read_to_string(entry.path()) {
                let s = status.trim().to_lowercase();
                if s.contains("not affected") || s.contains("mitigation") {
                    mitigated += 1;
                } else {
                    unmitigated.push(format!("{}: {}",
                        entry.file_name().to_string_lossy(), status.trim()));
                }
            }
        }
    }

    if total == 0 {
        return AxiomStatus::unknown(name, desc);
    }

    let score = if total == mitigated { 0 }
    else { ((total - mitigated) as u64 * 10000) / total as u64 };

    let details = if unmitigated.is_empty() {
        format!("{mitigated}/{total} vulnerabilities mitigated")
    } else {
        format!("{mitigated}/{total} mitigated, unmitigated: {:?}", unmitigated)
    };

    if score == 0 {
        AxiomStatus::compliant(name, desc, details)
    } else {
        AxiomStatus::violated(name, desc, score, details)
    }
}

/// Read H4: Thermal Stability (frequency pinning, turbo status).
pub fn probe_h4() -> AxiomStatus {
    let name = "H4";
    let desc = "Thermal Stability";
    let mut issues = Vec::new();
    let mut score: u64 = 0;

    // Check governor
    let governor_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
    match std::fs::read_to_string(governor_path) {
        Ok(gov) => {
            let g = gov.trim();
            if g != "performance" {
                issues.push(format!("governor={g} (expected performance)"));
                score += 4000;
            }
        }
        Err(_) => {
            issues.push("cpufreq not available".into());
            score += 2000;
        }
    }

    // Check Intel turbo
    let turbo_path = "/sys/devices/system/cpu/intel_pstate/no_turbo";
    match std::fs::read_to_string(turbo_path) {
        Ok(val) => {
            if val.trim() != "1" {
                issues.push("Intel turbo boost enabled".into());
                score += 3000;
            }
        }
        Err(_) => {} // Not Intel pstate, skip
    }

    // Check AMD boost
    let boost_path = "/sys/devices/system/cpu/cpufreq/boost";
    match std::fs::read_to_string(boost_path) {
        Ok(val) => {
            if val.trim() != "0" {
                issues.push("AMD boost enabled".into());
                score += 3000;
            }
        }
        Err(_) => {} // Not AMD, skip
    }

    score = score.min(10000);
    let details = if issues.is_empty() {
        "Frequency pinned, turbo disabled".into()
    } else {
        issues.join("; ")
    };

    if score == 0 {
        AxiomStatus::compliant(name, desc, details)
    } else {
        AxiomStatus::violated(name, desc, score, details)
    }
}

// ============================================================================
// THREAT VECTOR
// ============================================================================

/// Threat vector weights (how much each axiom violation contributes).
/// Sum to 10000 for normalized scoring.
pub const THREAT_WEIGHTS: [u64; 4] = [
    3000, // H1: Temporal isolation is critical for timing attacks
    2000, // H2: Cache isolation prevents cache-timing leaks
    3000, // H3: Speculation shield prevents Spectre/Meltdown class
    2000, // H4: Thermal stability prevents frequency-based side channels
];

/// Compute threat vector from axiom statuses.
///
/// θ = Σᵢ (wᵢ × vᵢ) / 10000
/// Result in [0, 10000].
pub fn compute_threat_vector(statuses: &[AxiomStatus; 4]) -> u64 {
    let weighted_sum: u64 = statuses.iter()
        .zip(THREAT_WEIGHTS.iter())
        .map(|(s, &w)| (s.violation * w) / 10000)
        .sum();
    weighted_sum.min(10000)
}

// ============================================================================
// HEALTH REPORT
// ============================================================================

/// Complete health report.
#[derive(Clone, Debug)]
pub struct HealthReport {
    pub axioms: [AxiomStatus; 4],
    pub threat_vector: u64,
    pub overall_health: u64, // 10000 - threat_vector
    pub timestamp_ticks: u64,
}

impl HealthReport {
    /// Overall health score: 10000 = perfect, 0 = fully compromised.
    pub fn health_score(&self) -> u64 {
        10000u64.saturating_sub(self.threat_vector)
    }

    /// Is the system in a safe state? (threat < 3000)
    pub fn is_safe(&self) -> bool {
        self.threat_vector < 3000
    }

    /// Is any axiom critically violated? (violation > 8000)
    pub fn has_critical_violation(&self) -> bool {
        self.axioms.iter().any(|a| a.violation >= 8000)
    }
}

impl fmt::Display for HealthReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== CDHS Health Report ===")?;
        for ax in &self.axioms {
            let status = if ax.violation == 0 { "OK" }
                else if ax.violation < 5000 { "WARN" }
                else { "FAIL" };
            writeln!(f, "  {}: [{}] {} (violation={}/10000)",
                ax.name, status, ax.description, ax.violation)?;
            writeln!(f, "       {}", ax.details)?;
        }
        writeln!(f, "  Threat Vector: {}/10000", self.threat_vector)?;
        writeln!(f, "  Health Score:  {}/10000", self.health_score())?;
        writeln!(f, "  Status: {}", if self.is_safe() { "SAFE" } else { "ELEVATED" })?;
        Ok(())
    }
}

// ============================================================================
// CDHS COLLECTOR
// ============================================================================

/// The CDHS telemetry collector.
///
/// Implements periodic health checks and feeds Loki with threat data.
pub struct CDHSCollector {
    /// Running tick counter (from TimeCrystal in production)
    tick: u64,
    /// Last health report
    last_report: Option<HealthReport>,
    /// History of threat vectors (ring buffer)
    threat_history: Vec<u64>,
    max_history: usize,
}

impl CDHSCollector {
    pub fn new() -> Self {
        Self {
            tick: 0,
            last_report: None,
            threat_history: Vec::new(),
            max_history: 1024,
        }
    }

    /// Collect a fresh health report from hardware probes.
    pub fn collect(&mut self) -> HealthReport {
        self.tick += 1;
        let axioms = [probe_h1(), probe_h2(), probe_h3(), probe_h4()];
        let threat = compute_threat_vector(&axioms);

        self.threat_history.push(threat);
        if self.threat_history.len() > self.max_history {
            self.threat_history.remove(0);
        }

        let report = HealthReport {
            axioms,
            threat_vector: threat,
            overall_health: 10000u64.saturating_sub(threat),
            timestamp_ticks: self.tick,
        };

        self.last_report = Some(report.clone());
        report
    }

    /// Collect from manual axiom statuses (for testing / mock).
    pub fn collect_manual(&mut self, axioms: [AxiomStatus; 4]) -> HealthReport {
        self.tick += 1;
        let threat = compute_threat_vector(&axioms);

        self.threat_history.push(threat);
        if self.threat_history.len() > self.max_history {
            self.threat_history.remove(0);
        }

        let report = HealthReport {
            axioms,
            threat_vector: threat,
            overall_health: 10000u64.saturating_sub(threat),
            timestamp_ticks: self.tick,
        };

        self.last_report = Some(report.clone());
        report
    }

    /// Get trend: average threat over last N samples.
    pub fn threat_trend(&self, window: usize) -> u64 {
        let samples: Vec<&u64> = self.threat_history.iter().rev().take(window).collect();
        if samples.is_empty() { return 0; }
        let sum: u64 = samples.iter().copied().sum();
        sum / samples.len() as u64
    }

    /// Is threat escalating? (recent average > overall average)
    pub fn is_escalating(&self) -> bool {
        if self.threat_history.len() < 10 { return false; }
        let recent = self.threat_trend(10);
        let overall = self.threat_trend(self.threat_history.len());
        recent > overall + 500 // 5% margin
    }

    pub fn last_report(&self) -> Option<&HealthReport> {
        self.last_report.as_ref()
    }

    pub fn tick_count(&self) -> u64 { self.tick }
}

impl Default for CDHSCollector {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_compliant() -> [AxiomStatus; 4] {
        [
            AxiomStatus::compliant("H1", "Temporal Isolation", "isolcpus=2-3".into()),
            AxiomStatus::compliant("H2", "Cache Isolation", "resctrl active".into()),
            AxiomStatus::compliant("H3", "Speculation Shield", "all mitigated".into()),
            AxiomStatus::compliant("H4", "Thermal Stability", "performance governor".into()),
        ]
    }

    fn mock_partially_violated() -> [AxiomStatus; 4] {
        [
            AxiomStatus::violated("H1", "Temporal", 8000, "no isolcpus".into()),
            AxiomStatus::compliant("H2", "Cache", "ok".into()),
            AxiomStatus::compliant("H3", "Speculation", "ok".into()),
            AxiomStatus::violated("H4", "Thermal", 4000, "turbo enabled".into()),
        ]
    }

    #[test]
    fn threat_vector_all_compliant() {
        let axioms = mock_compliant();
        let theta = compute_threat_vector(&axioms);
        assert_eq!(theta, 0, "Fully compliant should have zero threat");
    }

    #[test]
    fn threat_vector_partial_violation() {
        let axioms = mock_partially_violated();
        let theta = compute_threat_vector(&axioms);
        // H1: 8000 * 3000 / 10000 = 2400
        // H4: 4000 * 2000 / 10000 = 800
        // Total: 3200
        assert_eq!(theta, 3200, "Partial violation should sum weighted scores");
    }

    #[test]
    fn threat_vector_all_violated() {
        let axioms = [
            AxiomStatus::violated("H1", "", 10000, "".into()),
            AxiomStatus::violated("H2", "", 10000, "".into()),
            AxiomStatus::violated("H3", "", 10000, "".into()),
            AxiomStatus::violated("H4", "", 10000, "".into()),
        ];
        let theta = compute_threat_vector(&axioms);
        assert_eq!(theta, 10000, "Fully violated should max out at 10000");
    }

    #[test]
    fn health_report_safe() {
        let mut collector = CDHSCollector::new();
        let report = collector.collect_manual(mock_compliant());
        assert!(report.is_safe());
        assert!(!report.has_critical_violation());
        assert_eq!(report.health_score(), 10000);
    }

    #[test]
    fn health_report_critical() {
        let mut collector = CDHSCollector::new();
        let report = collector.collect_manual(mock_partially_violated());
        assert!(!report.is_safe()); // threat=3200 > 3000
        assert!(report.has_critical_violation()); // H1 at 8000
    }

    #[test]
    fn collector_tracks_history() {
        let mut collector = CDHSCollector::new();
        for _ in 0..5 {
            collector.collect_manual(mock_compliant());
        }
        for _ in 0..5 {
            collector.collect_manual(mock_partially_violated());
        }
        assert_eq!(collector.tick_count(), 10);
        assert_eq!(collector.threat_history.len(), 10);
    }

    #[test]
    fn threat_trend_computation() {
        let mut collector = CDHSCollector::new();
        for _ in 0..10 {
            collector.collect_manual(mock_compliant());
        }
        assert_eq!(collector.threat_trend(5), 0);

        for _ in 0..10 {
            collector.collect_manual(mock_partially_violated());
        }
        assert_eq!(collector.threat_trend(5), 3200);
    }

    #[test]
    fn escalation_detection() {
        let mut collector = CDHSCollector::new();
        // Start safe
        for _ in 0..20 {
            collector.collect_manual(mock_compliant());
        }
        assert!(!collector.is_escalating());

        // Escalate
        for _ in 0..15 {
            collector.collect_manual(mock_partially_violated());
        }
        assert!(collector.is_escalating());
    }

    #[test]
    fn live_probe_doesnt_panic() {
        // These probe real hardware — should never panic, just return status
        let _h1 = probe_h1();
        let _h2 = probe_h2();
        let _h3 = probe_h3();
        let _h4 = probe_h4();
    }

    #[test]
    fn live_collect_doesnt_panic() {
        let mut collector = CDHSCollector::new();
        let report = collector.collect();
        // In a container, most axioms will be violated — that's expected
        assert!(report.threat_vector <= 10000);
        assert!(report.health_score() <= 10000);
    }

    #[test]
    fn report_display() {
        let mut collector = CDHSCollector::new();
        let report = collector.collect_manual(mock_partially_violated());
        let output = format!("{}", report);
        assert!(output.contains("CDHS Health Report"));
        assert!(output.contains("H1"));
        assert!(output.contains("Threat Vector"));
    }
}
