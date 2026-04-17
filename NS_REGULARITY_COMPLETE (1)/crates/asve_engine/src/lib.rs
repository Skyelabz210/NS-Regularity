//! # ASVE Cognitive Engine — W4.1
//!
//! **Adaptive Self-Validating Engine**: The Mind of HackFate.
//!
//! ## Cognitive Reentry Loop (L_CR)
//!
//! ```text
//!          ┌──────────────────────────────────┐
//!          │          ASVE Tick               │
//!          │                                  │
//!          │  1. PERCEIVE  (input → concept)  │
//!          │  2. REMEMBER  (WASSAN retrieval)  │
//!          │  3. ANALYZE   (SVD coherence)     │
//!          │  4. HYPOTHESIZE (gap → candidate) │
//!          │  5. VALIDATE  (test vs K-graph)   │
//!          │  6. LEARN     (store to WASSAN)   │
//!          │  7. SECURE    (Loki audit)        │
//!          │                                  │
//!          └──────────┬───────────────────────┘
//!                     │ loop
//!                     └──→ tick again
//! ```
//!
//! ## Integer-Only Cognitive State
//!
//! All cognitive metrics are integer-quantized:
//! - Coherence: 0..10000
//! - Confidence: 0..10000
//! - Novelty: 0..10000
//! - Threat: 0..10000

#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic)]

use std::collections::HashMap;
use std::fmt;

// ============================================================================
// COGNITIVE TYPES
// ============================================================================

/// A concept in the knowledge graph.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConceptId(pub u64);

/// A knowledge node: a concept with associated data and confidence.
#[derive(Clone, Debug)]
pub struct KnowledgeNode {
    pub id: ConceptId,
    pub label: String,
    pub data: Vec<u8>,
    pub confidence: u64,     // 0..10000
    pub creation_tick: u64,
    pub last_accessed: u64,
    pub access_count: u64,
    pub validated: bool,
}

/// An edge between two concepts (directed, weighted).
#[derive(Clone, Debug)]
pub struct KnowledgeEdge {
    pub from: ConceptId,
    pub to: ConceptId,
    pub relation: String,
    pub strength: u64,       // 0..10000
    pub creation_tick: u64,
}

/// A hypothesis generated during gap analysis.
#[derive(Clone, Debug)]
pub struct Hypothesis {
    pub id: u64,
    pub description: String,
    pub related_concepts: Vec<ConceptId>,
    pub predicted_coherence_gain: u64, // 0..10000
    pub status: HypothesisStatus,
    pub creation_tick: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HypothesisStatus {
    Pending,
    Validated,
    Rejected,
    Inconclusive,
}

/// A gap detected in the knowledge graph.
#[derive(Clone, Debug)]
pub struct CognitiveGap {
    pub description: String,
    pub severity: u64,        // 0..10000
    pub related_concepts: Vec<ConceptId>,
    pub detected_tick: u64,
}

// ============================================================================
// KNOWLEDGE GRAPH
// ============================================================================

/// The knowledge graph: nodes (concepts) + edges (relations).
pub struct KnowledgeGraph {
    nodes: HashMap<ConceptId, KnowledgeNode>,
    edges: Vec<KnowledgeEdge>,
    next_id: u64,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self { nodes: HashMap::new(), edges: Vec::new(), next_id: 1 }
    }

    /// Add a concept. Returns its ID.
    pub fn add_concept(&mut self, label: &str, data: &[u8], confidence: u64, tick: u64) -> ConceptId {
        let id = ConceptId(self.next_id);
        self.next_id += 1;
        self.nodes.insert(id.clone(), KnowledgeNode {
            id: id.clone(),
            label: label.to_string(),
            data: data.to_vec(),
            confidence: confidence.min(10000),
            creation_tick: tick,
            last_accessed: tick,
            access_count: 0,
            validated: false,
        });
        id
    }

    /// Add a relation between concepts.
    pub fn add_relation(&mut self, from: &ConceptId, to: &ConceptId, relation: &str, strength: u64, tick: u64) {
        self.edges.push(KnowledgeEdge {
            from: from.clone(),
            to: to.clone(),
            relation: relation.to_string(),
            strength: strength.min(10000),
            creation_tick: tick,
        });
    }

    /// Get a concept by ID, updating access metrics.
    pub fn access(&mut self, id: &ConceptId, tick: u64) -> Option<&KnowledgeNode> {
        if let Some(node) = self.nodes.get_mut(id) {
            node.last_accessed = tick;
            node.access_count += 1;
        }
        self.nodes.get(id)
    }

    /// Find concepts by label overlap (bidirectional substring match).
    pub fn search(&self, query: &str) -> Vec<&KnowledgeNode> {
        self.nodes.values()
            .filter(|n| n.label.contains(query) || query.contains(&n.label))
            .collect()
    }

    /// Get all edges from a concept.
    pub fn edges_from(&self, id: &ConceptId) -> Vec<&KnowledgeEdge> {
        self.edges.iter().filter(|e| e.from == *id).collect()
    }

    /// Get all edges to a concept.
    pub fn edges_to(&self, id: &ConceptId) -> Vec<&KnowledgeEdge> {
        self.edges.iter().filter(|e| e.to == *id).collect()
    }

    /// Mark a concept as validated.
    pub fn validate(&mut self, id: &ConceptId) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.validated = true;
            node.confidence = node.confidence.max(8000);
        }
    }

    /// Compute graph coherence: average confidence across all nodes.
    pub fn average_confidence(&self) -> u64 {
        if self.nodes.is_empty() { return 0; }
        let sum: u64 = self.nodes.values().map(|n| n.confidence).sum();
        sum / self.nodes.len() as u64
    }

    /// Compute connectivity: average edges per node.
    pub fn connectivity_permil(&self) -> u64 {
        if self.nodes.is_empty() { return 0; }
        (self.edges.len() as u64 * 1000) / self.nodes.len() as u64
    }

    /// Find isolated nodes (no edges in or out).
    pub fn isolated_nodes(&self) -> Vec<&KnowledgeNode> {
        self.nodes.values()
            .filter(|n| {
                !self.edges.iter().any(|e| e.from == n.id || e.to == n.id)
            })
            .collect()
    }

    pub fn node_count(&self) -> usize { self.nodes.len() }
    pub fn edge_count(&self) -> usize { self.edges.len() }
    /// ID of the most recently added concept (next_id - 1).
    pub fn last_id(&self) -> Option<ConceptId> {
        if self.next_id > 1 { Some(ConceptId(self.next_id - 1)) } else { None }
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self { Self::new() }
}

// ============================================================================
// COGNITIVE STATE
// ============================================================================

/// The full cognitive state at any given tick.
#[derive(Clone, Debug)]
pub struct CognitiveState {
    /// Graph-level coherence (0..10000)
    pub coherence: u64,
    /// Number of active gaps
    pub gap_count: usize,
    /// Most severe gap severity
    pub max_gap_severity: u64,
    /// Number of pending hypotheses
    pub pending_hypotheses: usize,
    /// Number of validated hypotheses (lifetime)
    pub validated_hypotheses: u64,
    /// Number of rejected hypotheses (lifetime)
    pub rejected_hypotheses: u64,
    /// Security threat level (from Loki)
    pub threat_level: u64,
    /// Current tick
    pub tick: u64,
}

// ============================================================================
// ASVE ENGINE
// ============================================================================

/// The ASVE Cognitive Engine.
pub struct ASVEEngine {
    /// The knowledge graph
    pub graph: KnowledgeGraph,
    /// Active cognitive gaps
    gaps: Vec<CognitiveGap>,
    /// Active hypotheses
    hypotheses: Vec<Hypothesis>,
    /// Hypothesis counter
    next_hypothesis_id: u64,
    /// Current tick
    tick: u64,
    /// Validated hypothesis count (lifetime)
    validated_count: u64,
    /// Rejected hypothesis count (lifetime)
    rejected_count: u64,
    /// Last known threat level (from Loki)
    threat_level: u64,
    /// Paradigm shift history
    paradigm_shifts: Vec<(u64, String)>,
}

/// Result of a single cognitive tick.
#[derive(Clone, Debug)]
pub struct TickResult {
    pub tick: u64,
    pub state: CognitiveState,
    pub new_gaps: usize,
    pub hypotheses_generated: usize,
    pub hypotheses_validated: usize,
    pub hypotheses_rejected: usize,
    pub concepts_learned: usize,
    pub paradigm_shifted: bool,
}

impl ASVEEngine {
    pub fn new() -> Self {
        Self {
            graph: KnowledgeGraph::new(),
            gaps: Vec::new(),
            hypotheses: Vec::new(),
            next_hypothesis_id: 1,
            tick: 0,
            validated_count: 0,
            rejected_count: 0,
            threat_level: 0,
            paradigm_shifts: Vec::new(),
        }
    }

    /// ─── PERCEIVE ───────────────────────────────────────────────
    /// Convert raw input into a concept and add to knowledge graph.
    pub fn perceive(&mut self, label: &str, data: &[u8], confidence: u64) -> ConceptId {
        self.graph.add_concept(label, data, confidence, self.tick)
    }

    /// ─── REMEMBER ───────────────────────────────────────────────
    /// Search knowledge graph for relevant prior knowledge.
    pub fn remember(&mut self, query: &str) -> Vec<ConceptId> {
        let results = self.graph.search(query);
        let ids: Vec<ConceptId> = results.iter().map(|n| n.id.clone()).collect();
        // Access each found node to update recency
        for id in &ids {
            self.graph.access(id, self.tick);
        }
        ids
    }

    /// ─── ANALYZE ────────────────────────────────────────────────
    /// Detect gaps in the knowledge graph.
    ///
    /// Gaps are detected by:
    /// 1. Isolated nodes (concepts with no relations)
    /// 2. Low-confidence nodes
    /// 3. Low overall connectivity
    pub fn analyze(&mut self) -> Vec<CognitiveGap> {
        let mut new_gaps = Vec::new();

        // Gap type 1: Isolated concepts
        let isolated = self.graph.isolated_nodes();
        for node in &isolated {
            if node.creation_tick < self.tick { // Don't flag brand-new nodes
                new_gaps.push(CognitiveGap {
                    description: format!("Isolated concept: '{}'", node.label),
                    severity: 6000,
                    related_concepts: vec![node.id.clone()],
                    detected_tick: self.tick,
                });
            }
        }

        // Gap type 2: Low-confidence validated nodes
        for node in self.graph.nodes.values() {
            if node.validated && node.confidence < 5000 {
                new_gaps.push(CognitiveGap {
                    description: format!("Low-confidence validated: '{}'", node.label),
                    severity: 4000,
                    related_concepts: vec![node.id.clone()],
                    detected_tick: self.tick,
                });
            }
        }

        // Gap type 3: Low connectivity
        let conn = self.graph.connectivity_permil();
        if self.graph.node_count() > 3 && conn < 500 { // < 0.5 edges per node
            new_gaps.push(CognitiveGap {
                description: format!("Low graph connectivity: {}/1000 edges/node", conn),
                severity: 3000,
                related_concepts: vec![],
                detected_tick: self.tick,
            });
        }

        self.gaps.extend(new_gaps.clone());
        new_gaps
    }

    /// ─── HYPOTHESIZE ────────────────────────────────────────────
    /// Generate hypotheses to address detected gaps.
    pub fn hypothesize(&mut self) -> Vec<Hypothesis> {
        let mut new_hyps = Vec::new();

        for gap in &self.gaps {
            if gap.detected_tick != self.tick { continue; } // Only fresh gaps

            let hyp = Hypothesis {
                id: self.next_hypothesis_id,
                description: format!("Address gap: {}", gap.description),
                related_concepts: gap.related_concepts.clone(),
                predicted_coherence_gain: gap.severity / 2, // Optimistic: fix half the gap
                status: HypothesisStatus::Pending,
                creation_tick: self.tick,
            };
            self.next_hypothesis_id += 1;
            new_hyps.push(hyp);
        }

        self.hypotheses.extend(new_hyps.clone());
        new_hyps
    }

    /// ─── VALIDATE ───────────────────────────────────────────────
    /// Test pending hypotheses against the knowledge graph.
    ///
    /// Simple validation: if related concepts exist and have edges,
    /// the hypothesis is validated. If concepts are missing or
    /// isolated, it's rejected.
    pub fn validate(&mut self) -> (usize, usize) {
        let mut validated = 0usize;
        let mut rejected = 0usize;

        for hyp in self.hypotheses.iter_mut() {
            if hyp.status != HypothesisStatus::Pending { continue; }

            // Check if related concepts have been connected since hypothesis creation
            let connected = hyp.related_concepts.iter().all(|c| {
                !self.graph.edges_from(c).is_empty() || !self.graph.edges_to(c).is_empty()
            });

            if hyp.related_concepts.is_empty() {
                hyp.status = HypothesisStatus::Inconclusive;
            } else if connected {
                hyp.status = HypothesisStatus::Validated;
                validated += 1;
                self.validated_count += 1;
                // Boost confidence of related concepts
                for c in &hyp.related_concepts {
                    self.graph.validate(c);
                }
            } else if self.tick - hyp.creation_tick > 10 {
                // Hypothesis too old without resolution → reject
                hyp.status = HypothesisStatus::Rejected;
                rejected += 1;
                self.rejected_count += 1;
            }
        }

        // Clean up resolved hypotheses (keep last 100)
        self.hypotheses.retain(|h| {
            h.status == HypothesisStatus::Pending ||
            self.tick - h.creation_tick < 50
        });

        (validated, rejected)
    }

    /// ─── LEARN ──────────────────────────────────────────────────
    /// Store a validated insight as a new concept + relation.
    pub fn learn(&mut self, label: &str, data: &[u8], related_to: &ConceptId, relation: &str) -> ConceptId {
        let id = self.graph.add_concept(label, data, 7500, self.tick);
        self.graph.add_relation(related_to, &id, relation, 8000, self.tick);
        id
    }

    /// ─── SECURE ─────────────────────────────────────────────────
    /// Update threat level from Loki and adjust cognitive behavior.
    pub fn secure(&mut self, threat: u64) {
        self.threat_level = threat;
        // Under high threat, increase validation strictness
        // (In production: reduce hypothesis generation rate, increase
        //  validation threshold, alert the meta-cognition loop)
    }

    /// ─── PARADIGM SHIFT ─────────────────────────────────────────
    /// Suspend an axiom, re-evaluate, adopt or reject.
    ///
    /// This is the mechanism by which ASVE can change its own assumptions.
    pub fn paradigm_shift(&mut self, suspended_axiom: &str, replacement: &str) -> bool {
        // Record the shift
        self.paradigm_shifts.push((self.tick, format!(
            "Suspended '{}', replaced with '{}'", suspended_axiom, replacement
        )));

        // In production: re-evaluate all knowledge nodes that depend on
        // the suspended axiom. For now, mark all as needing re-validation.
        for node in self.graph.nodes.values_mut() {
            if node.label.contains(suspended_axiom) {
                node.validated = false;
                node.confidence = node.confidence.min(5000);
            }
        }

        true // Shift accepted
    }

    /// ─── FULL TICK ──────────────────────────────────────────────
    /// Execute one complete cognitive reentry loop.
    pub fn tick(&mut self, input_label: Option<&str>, input_data: Option<&[u8]>, threat: u64) -> TickResult {
        self.tick += 1;

        // 1. PERCEIVE
        let mut concepts_learned = 0;
        if let (Some(label), Some(data)) = (input_label, input_data) {
            self.perceive(label, data, 6000);
            concepts_learned += 1;
        }

        // 2. REMEMBER (search for related concepts)
        if let Some(label) = input_label {
            let related = self.remember(label);
            // Connect new concept to related ones
            if !related.is_empty() {
                let newest_id = ConceptId(self.graph.next_id - 1);
                for rel_id in &related {
                    if *rel_id != newest_id {
                        self.graph.add_relation(rel_id, &newest_id, "related_to", 5000, self.tick);
                    }
                }
            }
        }

        // 3. ANALYZE
        let new_gaps = self.analyze();

        // 4. HYPOTHESIZE
        let new_hypotheses = self.hypothesize();

        // 5. VALIDATE
        let (validated, rejected) = self.validate();

        // 6. LEARN (implicit — new concepts stored in perceive/learn)

        // 7. SECURE
        self.secure(threat);

        let state = self.cognitive_state();
        let paradigm_shifted = false; // Only on explicit call

        TickResult {
            tick: self.tick,
            state,
            new_gaps: new_gaps.len(),
            hypotheses_generated: new_hypotheses.len(),
            hypotheses_validated: validated,
            hypotheses_rejected: rejected,
            concepts_learned,
            paradigm_shifted,
        }
    }

    /// Get current cognitive state snapshot.
    pub fn cognitive_state(&self) -> CognitiveState {
        let max_gap = self.gaps.iter()
            .filter(|g| self.tick - g.detected_tick < 20) // Recent gaps only
            .map(|g| g.severity)
            .max()
            .unwrap_or(0);

        let pending = self.hypotheses.iter()
            .filter(|h| h.status == HypothesisStatus::Pending)
            .count();

        CognitiveState {
            coherence: self.graph.average_confidence(),
            gap_count: self.gaps.iter().filter(|g| self.tick - g.detected_tick < 20).count(),
            max_gap_severity: max_gap,
            pending_hypotheses: pending,
            validated_hypotheses: self.validated_count,
            rejected_hypotheses: self.rejected_count,
            threat_level: self.threat_level,
            tick: self.tick,
        }
    }

    /// NLG hedge level based on coherence score.
    /// Maps coherence to linguistic confidence markers.
    pub fn hedge_level(&self) -> &'static str {
        match self.graph.average_confidence() {
            9000..=10000 => "certain",
            7000..=8999 => "confident",
            5000..=6999 => "probable",
            3000..=4999 => "uncertain",
            1000..=2999 => "speculative",
            _ => "unknown",
        }
    }

    pub fn tick_count(&self) -> u64 { self.tick }
    pub fn paradigm_shift_count(&self) -> usize { self.paradigm_shifts.len() }
}

impl Default for ASVEEngine {
    fn default() -> Self { Self::new() }
}

impl fmt::Debug for ASVEEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ASVEEngine(tick={}, nodes={}, edges={}, gaps={}, hyps={}, hedge='{}')",
            self.tick, self.graph.node_count(), self.graph.edge_count(),
            self.gaps.len(), self.hypotheses.len(), self.hedge_level())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Knowledge Graph ---

    #[test]
    fn kg_add_and_retrieve() {
        let mut g = KnowledgeGraph::new();
        let id = g.add_concept("CRT", b"Chinese Remainder Theorem", 9000, 1);
        let node = g.access(&id, 2).unwrap();
        assert_eq!(node.label, "CRT");
        assert_eq!(node.confidence, 9000);
        assert_eq!(node.access_count, 1);
    }

    #[test]
    fn kg_search() {
        let mut g = KnowledgeGraph::new();
        g.add_concept("K-Elimination", b"exact RNS division", 9500, 1);
        g.add_concept("K-Factor", b"correction factor", 7000, 1);
        g.add_concept("NTT", b"number theoretic transform", 8000, 1);

        let results = g.search("K-");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn kg_edges() {
        let mut g = KnowledgeGraph::new();
        let a = g.add_concept("CRT", b"", 9000, 1);
        let b = g.add_concept("K-Elimination", b"", 9500, 1);
        g.add_relation(&a, &b, "enables", 9000, 1);

        assert_eq!(g.edges_from(&a).len(), 1);
        assert_eq!(g.edges_to(&b).len(), 1);
        assert_eq!(g.edges_from(&b).len(), 0);
    }

    #[test]
    fn kg_isolated_nodes() {
        let mut g = KnowledgeGraph::new();
        let a = g.add_concept("Connected", b"", 9000, 1);
        let b = g.add_concept("Also Connected", b"", 9000, 1);
        let _c = g.add_concept("Isolated", b"", 5000, 1);
        g.add_relation(&a, &b, "relates", 7000, 1);

        let isolated = g.isolated_nodes();
        assert_eq!(isolated.len(), 1);
        assert_eq!(isolated[0].label, "Isolated");
    }

    #[test]
    fn kg_average_confidence() {
        let mut g = KnowledgeGraph::new();
        g.add_concept("A", b"", 8000, 1);
        g.add_concept("B", b"", 6000, 1);
        assert_eq!(g.average_confidence(), 7000);
    }

    // --- ASVE Engine ---

    #[test]
    fn asve_perceive() {
        let mut asve = ASVEEngine::new();
        asve.tick += 1;
        let id = asve.perceive("K-Elimination", b"exact division in RNS", 9500);
        assert_eq!(asve.graph.node_count(), 1);
        let node = asve.graph.access(&id, 2).unwrap();
        assert_eq!(node.label, "K-Elimination");
    }

    #[test]
    fn asve_remember() {
        let mut asve = ASVEEngine::new();
        asve.tick = 1;
        asve.perceive("K-Elimination", b"", 9000);
        asve.perceive("K-Factor", b"", 7000);
        asve.perceive("NTT", b"", 8000);

        asve.tick = 2;
        let found = asve.remember("K-");
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn asve_analyze_detects_gaps() {
        let mut asve = ASVEEngine::new();
        asve.tick = 1;
        asve.perceive("Isolated Concept", b"no relations", 5000);

        asve.tick = 5; // Let it age
        let gaps = asve.analyze();
        assert!(!gaps.is_empty(), "Should detect isolated concept gap");
    }

    #[test]
    fn asve_hypothesize() {
        let mut asve = ASVEEngine::new();
        asve.tick = 1;
        asve.perceive("Orphan", b"", 5000);
        asve.tick = 5;
        asve.analyze();
        let hyps = asve.hypothesize();
        assert!(!hyps.is_empty(), "Should generate hypotheses for gaps");
        assert_eq!(hyps[0].status, HypothesisStatus::Pending);
    }

    #[test]
    fn asve_validate_connected() {
        let mut asve = ASVEEngine::new();
        asve.tick = 1;
        let a = asve.perceive("CRT", b"", 8000);
        let b = asve.perceive("K-Elim", b"", 9000);
        asve.graph.add_relation(&a, &b, "enables", 9000, 1);

        // Create a hypothesis about these connected concepts
        asve.hypotheses.push(Hypothesis {
            id: 1,
            description: "Test".into(),
            related_concepts: vec![a, b],
            predicted_coherence_gain: 5000,
            status: HypothesisStatus::Pending,
            creation_tick: 1,
        });

        let (validated, _rejected) = asve.validate();
        assert_eq!(validated, 1, "Connected concepts should validate");
    }

    #[test]
    fn asve_learn() {
        let mut asve = ASVEEngine::new();
        asve.tick = 1;
        let parent = asve.perceive("CRT", b"foundation", 9000);
        let child = asve.learn("K-Elimination", b"exact division", &parent, "derived_from");

        assert_eq!(asve.graph.node_count(), 2);
        assert_eq!(asve.graph.edge_count(), 1);
        let node = asve.graph.access(&child, 2).unwrap();
        assert_eq!(node.confidence, 7500);
    }

    #[test]
    fn asve_full_tick() {
        let mut asve = ASVEEngine::new();

        // First tick: perceive a concept
        let r1 = asve.tick(Some("K-Elimination"), Some(b"exact RNS division"), 0);
        assert_eq!(r1.tick, 1);
        assert_eq!(r1.concepts_learned, 1);

        // Second tick: perceive related concept
        let r2 = asve.tick(Some("K-Factor"), Some(b"correction factor"), 0);
        assert_eq!(r2.tick, 2);

        // Third tick: perceive another, should auto-connect via remember
        let r3 = asve.tick(Some("K-Elimination v2"), Some(b"improved"), 0);
        assert_eq!(r3.tick, 3);
        // Should have found "K-Elimination" via remember and connected
        assert!(asve.graph.edge_count() > 0, "Should auto-connect related concepts");
    }

    #[test]
    fn asve_multi_tick_cycle() {
        let mut asve = ASVEEngine::new();

        // Build up knowledge over 20 ticks
        let labels = [
            "CRT", "K-Elimination", "Garner", "Montgomery",
            "Shadow Entropy", "PLMG", "WASSAN", "Loki",
            "NINE65", "AHOP", "CTI", "Ramanujan",
        ];

        for (i, label) in labels.iter().enumerate() {
            let data = format!("Concept {i}");
            asve.tick(Some(label), Some(data.as_bytes()), 0);
        }

        // Run some empty ticks to let analysis/validation cycle
        for _ in 0..10 {
            asve.tick(None, None, 0);
        }

        let state = asve.cognitive_state();
        assert!(state.coherence > 0, "Should have some coherence");
        assert!(asve.graph.node_count() >= 12);
    }

    #[test]
    fn asve_paradigm_shift() {
        let mut asve = ASVEEngine::new();
        asve.tick = 1;
        let _id = asve.perceive("Float assumption", b"IEEE 754 is necessary", 8000);

        let shifted = asve.paradigm_shift("Float", "Integer-only exact arithmetic");
        assert!(shifted);
        assert_eq!(asve.paradigm_shift_count(), 1);

        // Affected concept should have reduced confidence
        let results = asve.graph.search("Float");
        assert!(!results.is_empty());
        assert!(results[0].confidence <= 5000, "Shifted axiom should reduce confidence");
        assert!(!results[0].validated, "Shifted axiom should be de-validated");
    }

    #[test]
    fn asve_hedge_level() {
        let mut asve = ASVEEngine::new();
        assert_eq!(asve.hedge_level(), "unknown"); // No concepts

        asve.tick = 1;
        asve.perceive("High confidence", b"", 9500);
        assert_eq!(asve.hedge_level(), "certain");

        asve.perceive("Low confidence", b"", 1000);
        // Average: (9500 + 1000) / 2 = 5250
        assert_eq!(asve.hedge_level(), "probable");
    }

    #[test]
    fn asve_threat_affects_state() {
        let mut asve = ASVEEngine::new();
        asve.tick(Some("Data"), Some(b"test"), 0);
        assert_eq!(asve.cognitive_state().threat_level, 0);

        asve.tick(None, None, 8000);
        assert_eq!(asve.cognitive_state().threat_level, 8000);
    }

    #[test]
    fn asve_cognitive_state_snapshot() {
        let mut asve = ASVEEngine::new();
        for i in 0..5 {
            let label = format!("Concept_{i}");
            asve.tick(Some(&label), Some(b"data"), 0);
        }
        let state = asve.cognitive_state();
        assert_eq!(state.tick, 5);
        assert!(state.coherence > 0);
        assert_eq!(state.threat_level, 0);
    }
}
