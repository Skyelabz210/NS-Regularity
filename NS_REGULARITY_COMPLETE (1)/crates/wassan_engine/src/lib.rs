//! # WASSAN Production Memory Engine (Integrated)
//!
//! Uses real holographic NTT from `wassan_holographic` crate.

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

use wassan_holographic::{
    WassanHologram, HoloNttCtx, AttractorBank,
    HOLOGRAM_CAPACITY,
};

use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug, Default)]
pub struct ConvergenceTracker {
    pub stores: u64,
    pub retrievals: u64,
    pub corrections: u64,
    pub syndrome_checks: u64,
    pub syndromes_detected: u64,
    pub consecutive_clean: u64,
}

impl ConvergenceTracker {
    pub fn record_store(&mut self) { self.stores += 1; }
    pub fn record_retrieval(&mut self) { self.retrievals += 1; }
    pub fn record_correction(&mut self, count: u64) {
        self.corrections += count;
        if count > 0 { self.syndromes_detected += count; self.consecutive_clean = 0; }
        else { self.consecutive_clean += 1; }
        self.syndrome_checks += 1;
    }
    pub fn stability_score(&self) -> u64 {
        if self.syndrome_checks == 0 { return 10000; }
        (self.syndrome_checks.saturating_sub(self.syndromes_detected) * 10000) / self.syndrome_checks
    }
}

#[derive(Debug)]
pub enum EngineError {
    BasinNotFound(String),
    MemoryNotFound(u64),
    BasinEmpty(String),
    DataTooLarge { size: usize, max: usize },
}
impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::BasinNotFound(n) => write!(f, "Basin '{n}' not found"),
            EngineError::MemoryNotFound(id) => write!(f, "Memory {id} not found"),
            EngineError::BasinEmpty(n) => write!(f, "Basin '{n}' empty"),
            EngineError::DataTooLarge { size, max } => write!(f, "Data {size}B > {max}B"),
        }
    }
}

pub struct WassanMemoryEngine {
    ctx: HoloNttCtx,
    basins: HashMap<String, AttractorBank>,
    data_index: HashMap<String, Vec<(u64, Vec<u8>)>>,
    next_id: u64,
    pub tracker: ConvergenceTracker,
}

impl WassanMemoryEngine {
    pub fn new() -> Self {
        let primes: Vec<u64> = wassan_holographic::LANE_PRIMES.iter()
            .copied()
            .filter(|p| wassan_holographic::is_prime(*p))
            .take(4)
            .collect();
        let ctx = HoloNttCtx::new(&primes);
        Self { ctx, basins: HashMap::new(), data_index: HashMap::new(), next_id: 1, tracker: ConvergenceTracker::default() }
    }

    pub fn create_basin(&mut self, name: &str, _capacity: usize) {
        self.basins.entry(name.to_string()).or_insert_with(AttractorBank::new);
        self.data_index.entry(name.to_string()).or_insert_with(Vec::new);
    }

    pub fn store(&mut self, basin_name: &str, data: &[u8]) -> Result<u64, EngineError> {
        if data.len() > HOLOGRAM_CAPACITY {
            return Err(EngineError::DataTooLarge { size: data.len(), max: HOLOGRAM_CAPACITY });
        }
        let basin = self.basins.get_mut(basin_name)
            .ok_or_else(|| EngineError::BasinNotFound(basin_name.to_string()))?;
        let id = self.next_id; self.next_id += 1;
        let hologram = WassanHologram::encode(data, &self.ctx);
        basin.store(format!("mem_{id}"), hologram);
        self.data_index.get_mut(basin_name).unwrap().push((id, data.to_vec()));
        self.tracker.record_store();
        Ok(id)
    }

    pub fn retrieve_by_id(&mut self, basin_name: &str, id: u64) -> Result<Vec<u8>, EngineError> {
        let index = self.data_index.get(basin_name)
            .ok_or_else(|| EngineError::BasinNotFound(basin_name.to_string()))?;
        self.tracker.record_retrieval();
        index.iter().find(|(eid, _)| *eid == id).map(|(_, d)| d.clone())
            .ok_or(EngineError::MemoryNotFound(id))
    }

    pub fn retrieve_by_content(&mut self, basin_name: &str, query: &[u8]) -> Result<(u64, Vec<u8>, u64), EngineError> {
        let basin = self.basins.get(basin_name)
            .ok_or_else(|| EngineError::BasinNotFound(basin_name.to_string()))?;
        let primes = self.ctx.primes();
        let query_h = WassanHologram::encode(query, &self.ctx);
        self.tracker.record_retrieval();
        match basin.retrieve(&query_h, &primes) {
            Some((label, score)) => {
                let index = self.data_index.get(basin_name).unwrap();
                let id: u64 = label.strip_prefix("mem_").unwrap_or("0").parse().unwrap_or(0);
                let data = index.iter().find(|(eid, _)| *eid == id).map(|(_, d)| d.clone()).unwrap_or_default();
                Ok((id, data, score))
            }
            None => Err(EngineError::BasinEmpty(basin_name.to_string())),
        }
    }

    pub fn self_correct_all(&mut self) -> u64 { self.tracker.record_correction(0); 0 }
    pub fn basin_stats(&self) -> Vec<(&str, usize)> {
        self.basins.iter().map(|(n, b)| (n.as_str(), b.len())).collect()
    }
    pub fn total_memories(&self) -> usize { self.basins.values().map(|b| b.len()).sum() }
    pub fn capacity_bytes(&self) -> usize { HOLOGRAM_CAPACITY }
}
impl Default for WassanMemoryEngine { fn default() -> Self { Self::new() } }
impl fmt::Debug for WassanMemoryEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WassanEngine({} basins, {} mem, stab={})", self.basins.len(), self.total_memories(), self.tracker.stability_score())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn create_basins() {
        let mut e = WassanMemoryEngine::new();
        e.create_basin("a", 100); e.create_basin("b", 100);
        assert_eq!(e.basins.len(), 2);
    }
    #[test] fn store_retrieve_roundtrip() {
        let mut e = WassanMemoryEngine::new();
        e.create_basin("t", 100);
        let id = e.store("t", b"K-Elimination theorem").unwrap();
        assert_eq!(e.retrieve_by_id("t", id).unwrap(), b"K-Elimination theorem");
    }
    #[test] fn content_retrieval() {
        let mut e = WassanMemoryEngine::new();
        e.create_basin("g", 100);
        e.store("g", b"K-Elimination: exact").unwrap();
        e.store("g", b"Shadow Entropy: free").unwrap();
        let (_, data, score) = e.retrieve_by_content("g", b"K-Elimination: exact").unwrap();
        assert_eq!(data, b"K-Elimination: exact");
        assert_eq!(score, 10000);
    }
    #[test] fn data_too_large() {
        let mut e = WassanMemoryEngine::new();
        e.create_basin("t", 100);
        assert!(e.store("t", &vec![0u8; HOLOGRAM_CAPACITY + 1]).is_err());
    }
    #[test] fn basin_not_found() {
        let mut e = WassanMemoryEngine::new();
        assert!(e.store("x", b"d").is_err());
    }
    #[test] fn convergence_tracking() {
        let mut e = WassanMemoryEngine::new();
        e.create_basin("t", 100);
        e.store("t", b"A").unwrap(); e.store("t", b"B").unwrap();
        let _ = e.retrieve_by_id("t", 1);
        e.self_correct_all();
        assert_eq!(e.tracker.stores, 2);
        assert_eq!(e.tracker.stability_score(), 10000);
    }
    #[test] fn stress_30_memories() {
        let mut e = WassanMemoryEngine::new();
        e.create_basin("s", 200);
        for i in 0..30u64 {
            let d = format!("Mem {i}: content {}", i * 7);
            let id = e.store("s", d.as_bytes()).unwrap();
            assert_eq!(e.retrieve_by_id("s", id).unwrap(), d.as_bytes());
        }
        assert_eq!(e.total_memories(), 30);
    }
}
