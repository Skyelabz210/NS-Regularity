import { useState, useCallback, useRef, useEffect, createContext, useContext, useMemo } from "react";

// ─────────────────────────────────────────────
// CRAM ENGINE — exact integer arithmetic core
// ─────────────────────────────────────────────

const SAFE_BASIS = [3, 5, 7, 11, 13];
const SAFE_M = SAFE_BASIS.reduce((a, b) => a * b, 1); // 15015

const SHADOW_BASIS = [9, 25, 7, 11, 13];
const SHADOW_M = SHADOW_BASIS.reduce((a, b) => a * b, 1); // 225225

function gcd(a, b) {
  a = Math.abs(a); b = Math.abs(b);
  while (b) { [a, b] = [b, a % b]; }
  return a;
}

function modInverse(a, m) {
  a = ((a % m) + m) % m;
  if (gcd(a, m) !== 1) return null;
  let [old_r, r] = [a, m];
  let [old_s, s] = [1, 0];
  while (r !== 0) {
    const q = Math.floor(old_r / r);
    [old_r, r] = [r, old_r - q * r];
    [old_s, s] = [s, old_s - q * s];
  }
  return ((old_s % m) + m) % m;
}

function crtReconstruct(residues, moduli) {
  const M = moduli.reduce((a, b) => a * b, 1);
  let result = 0;
  for (let i = 0; i < residues.length; i++) {
    const Mi = M / moduli[i];
    const yi = modInverse(Mi, moduli[i]);
    if (yi === null) return null;
    result += residues[i] * Mi * yi;
  }
  return result % M;
}

function kElim(rA, rQ, mA, mQ) {
  const diff = ((rQ - rA) % mQ + mQ) % mQ;
  const inv = modInverse(mA, mQ);
  if (inv === null) return null;
  return (diff * inv) % mQ;
}

function calibrateLane(spins, mod) {
  if (spins.length < 3) return null;
  for (let k = 0; k < spins.length - 2; k++) {
    const r1 = ((spins[k] % mod) + mod) % mod;
    const r2 = ((spins[k+1] % mod) + mod) % mod;
    const r3 = ((spins[k+2] % mod) + mod) % mod;
    const diffR = ((r2 - r1) % mod + mod) % mod;
    const diff1 = ((r3 - r2) % mod + mod) % mod;
    if (gcd(diffR, mod) === 1) {
      const inv = modInverse(diffR, mod);
      const a = (diff1 * inv) % mod;
      const c = ((r2 - (a * r1) % mod) % mod + mod) % mod;
      return { a, c };
    }
  }
  return null;
}

function calibrateAll(spins, basis) {
  const laneParams = {};
  for (const m of basis) {
    const p = calibrateLane(spins, m);
    if (!p) return null;
    laneParams[m] = p;
  }
  const aRes = basis.map(m => laneParams[m].a);
  const cRes = basis.map(m => laneParams[m].c);
  const globalA = crtReconstruct(aRes, basis);
  const globalC = crtReconstruct(cRes, basis);
  if (globalA === null || globalC === null) return null;
  return { laneParams, globalA, globalC };
}

function checkSyndromes(observed, predicted, basis) {
  const results = [];
  for (let i = 0; i < basis.length; i++) {
    for (let j = i + 1; j < basis.length; j++) {
      const kObs = kElim(observed % basis[i], observed % basis[j], basis[i], basis[j]);
      const kPred = kElim(predicted % basis[i], predicted % basis[j], basis[i], basis[j]);
      results.push({
        pair: `${basis[i]}×${basis[j]}`,
        kObs, kPred,
        match: kObs === kPred
      });
    }
  }
  return results;
}

// ─────────────────────────────────────────────
// PRNG TEST GENERATORS
// ─────────────────────────────────────────────

const GENERATORS = {
  lcg_simple: {
    name: "LCG (Simple)",
    desc: "Standard linear congruential. CRAM should achieve 100% after calibration.",
    create: (M) => {
      // Ensure a is coprime to M for full period
      let a;
      do { a = 2 + Math.floor(Math.random() * (M - 3)); } while (gcd(a, M) !== 1);
      const c = Math.floor(Math.random() * M);
      const seed = Math.floor(Math.random() * M);
      return {
        params: { a, c, seed, M },
        state: seed,
        next() { this.state = (this.params.a * this.state + this.params.c) % this.params.M; return this.state; }
      };
    }
  },
  lcg_large: {
    name: "LCG (Large Modulus)",
    desc: "LCG with M=2^31-1. Tests whether CRAM can recover params when generator modulus ≠ basis product.",
    create: () => {
      const M = 2147483647; // 2^31 - 1
      const a = 16807; // Park-Miller
      const c = 0;
      const seed = 1 + Math.floor(Math.random() * (M - 1));
      return {
        params: { a, c, seed, M },
        state: seed,
        next() { this.state = (this.params.a * this.state + this.params.c) % this.params.M; return this.state; }
      };
    }
  },
  lcg_truncated: {
    name: "LCG (Truncated Output)",
    desc: "LCG mod 15015 but output is masked to 10 bits. Simulates reel symbol extraction.",
    create: (M) => {
      let a;
      do { a = 2 + Math.floor(Math.random() * (M - 3)); } while (gcd(a, M) !== 1);
      const c = Math.floor(Math.random() * M);
      const seed = Math.floor(Math.random() * M);
      return {
        params: { a, c, seed, M, mask: 1023 },
        state: seed,
        next() {
          this.state = (this.params.a * this.state + this.params.c) % this.params.M;
          return this.state & this.params.mask; // truncate to 10 bits
        }
      };
    }
  },
  compound_lcg: {
    name: "Compound LCG",
    desc: "Two interleaved LCGs combined via XOR. Non-linear composition — calibration expected to fail.",
    create: (M) => {
      let a1, a2;
      do { a1 = 2 + Math.floor(Math.random() * (M - 3)); } while (gcd(a1, M) !== 1);
      do { a2 = 2 + Math.floor(Math.random() * (M - 3)); } while (gcd(a2, M) !== 1);
      const c1 = Math.floor(Math.random() * M);
      const c2 = Math.floor(Math.random() * M);
      const s1 = Math.floor(Math.random() * M);
      const s2 = Math.floor(Math.random() * M);
      return {
        params: { a1, c1, a2, c2, s1, s2, M },
        st1: s1, st2: s2,
        next() {
          this.st1 = (this.params.a1 * this.st1 + this.params.c1) % this.params.M;
          this.st2 = (this.params.a2 * this.st2 + this.params.c2) % this.params.M;
          return (this.st1 ^ this.st2) % this.params.M;
        }
      };
    }
  },
  xorshift: {
    name: "XorShift32",
    desc: "Non-linear bit-shift generator. Calibration expected to fail — tests CRAM boundary detection.",
    create: () => {
      let state = 1 + Math.floor(Math.random() * 0xFFFFFFFE);
      return {
        params: { seed: state },
        state,
        next() {
          let x = this.state;
          x ^= (x << 13) & 0xFFFFFFFF;
          x ^= (x >>> 17);
          x ^= (x << 5) & 0xFFFFFFFF;
          this.state = x >>> 0;
          return this.state % 15015;
        }
      };
    }
  },
  constant_diff: {
    name: "Constant Additive",
    desc: "x_{n+1} = (x_n + c) mod M. Trivial LCG with a=1. Fastest calibration.",
    create: (M) => {
      const c = 1 + Math.floor(Math.random() * (M - 1));
      const seed = Math.floor(Math.random() * M);
      return {
        params: { a: 1, c, seed, M },
        state: seed,
        next() { this.state = (this.state + this.params.c) % this.params.M; return this.state; }
      };
    }
  }
};

// ─────────────────────────────────────────────
// CRAM CONTEXT — shared state (fixes isolation bug)
// ─────────────────────────────────────────────

const CRAMContext = createContext(null);

function CRAMProvider({ children }) {
  const [mode, setMode] = useState("standard");
  const basis = mode === "standard" ? SAFE_BASIS : SHADOW_BASIS;
  const M = mode === "standard" ? SAFE_M : SHADOW_M;

  const [params, setParams] = useState(null);
  const [spins, setSpins] = useState([]);
  const [predictions, setPredictions] = useState([]);
  const [logs, setLogs] = useState([]);
  const [calStatus, setCalStatus] = useState(null);
  const [activeGen, setActiveGen] = useState(null);
  const [isRunning, setIsRunning] = useState(false);
  const [speed, setSpeed] = useState(300);

  const genRef = useRef(null);
  const intervalRef = useRef(null);
  const paramsRef = useRef(null);
  const spinsRef = useRef([]);

  // Keep refs in sync
  useEffect(() => { paramsRef.current = params; }, [params]);
  useEffect(() => { spinsRef.current = spins; }, [spins]);

  const log = useCallback((msg, level = "info") => {
    setLogs(prev => [{
      id: Date.now() + Math.random(),
      ts: new Date(),
      msg, level
    }, ...prev].slice(0, 200));
  }, []);

  const doCalibrate = useCallback((window, basisArr, Mval) => {
    const result = calibrateAll(window, basisArr);
    if (result) {
      setParams({ a: result.globalA, c: result.globalC });
      setCalStatus({
        ok: true,
        lanes: Object.entries(result.laneParams).map(([m, p]) => ({
          mod: Number(m), a: p.a, c: p.c, status: "ok"
        })),
        globalA: result.globalA,
        globalC: result.globalC
      });
      log(`Calibrated: a=${result.globalA}, c=${result.globalC} (M=${Mval})`, "success");
      return true;
    } else {
      setCalStatus({ ok: false, lanes: basisArr.map(m => ({ mod: m, status: "fail" })) });
      log("Calibration failed — generator may not be LCG-class", "error");
      return false;
    }
  }, [log]);

  const addSpin = useCallback((value) => {
    setSpins(prev => {
      const next = [...prev, value];
      spinsRef.current = next;

      // Auto-calibrate at 20 spins
      if (next.length === 20 && !paramsRef.current) {
        log("Auto-calibration at 20 spins...", "info");
        setTimeout(() => doCalibrate(spinsRef.current, basis, M), 0);
      }

      // Predict if calibrated
      if (paramsRef.current && next.length > 1) {
        const lastSpin = next[next.length - 2];
        const pred = (paramsRef.current.a * lastSpin + paramsRef.current.c) % M;
        const syndromes = checkSyndromes(value, pred, basis);
        const allMatch = syndromes.every(s => s.match);

        setPredictions(p => [...p, {
          spinNum: next.length,
          observed: value,
          predicted: pred,
          delta: Math.abs(value - pred),
          correct: value === pred,
          syndromes,
          allMatch
        }]);

        if (!allMatch && next.length >= 8) {
          log(`Syndrome mismatch at spin ${next.length} — recalibrating`, "warn");
          setTimeout(() => doCalibrate(spinsRef.current.slice(-8), basis, M), 0);
        }
      }

      return next;
    });
  }, [basis, M, log, doCalibrate]);

  const startGenerator = useCallback((genKey) => {
    stopGenerator();
    const genDef = GENERATORS[genKey];
    if (!genDef) return;

    setSpins([]);
    setPredictions([]);
    setParams(null);
    setCalStatus(null);
    paramsRef.current = null;
    spinsRef.current = [];

    const gen = genDef.create(M);
    genRef.current = gen;
    setActiveGen(genKey);
    log(`Generator: ${genDef.name} (M=${gen.params?.M || M})`, "info");
    if (gen.params?.a !== undefined) log(`True params: a=${gen.params.a}, c=${gen.params.c}`, "info");

    setIsRunning(true);
    let count = 0;
    intervalRef.current = setInterval(() => {
      const val = gen.next();
      addSpin(val);
      count++;
      if (count >= 150) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
        setIsRunning(false);
        log("Generator run complete (150 spins)", "info");
      }
    }, speed);
  }, [M, speed, addSpin, log]);

  const stopGenerator = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    setIsRunning(false);
  }, []);

  const manualCalibrate = useCallback(() => {
    if (spins.length >= 3) {
      doCalibrate(spins.slice(-Math.min(spins.length, 20)), basis, M);
    } else {
      log("Need at least 3 spins to calibrate", "error");
    }
  }, [spins, basis, M, doCalibrate, log]);

  const reset = useCallback(() => {
    stopGenerator();
    setSpins([]);
    setPredictions([]);
    setParams(null);
    setCalStatus(null);
    setActiveGen(null);
    paramsRef.current = null;
    spinsRef.current = [];
    log("System reset", "info");
  }, [stopGenerator, log]);

  const accuracy = useMemo(() => {
    if (predictions.length === 0) return { pct: 0, correct: 0, total: 0 };
    const correct = predictions.filter(p => p.correct).length;
    return { pct: (correct / predictions.length * 100), correct, total: predictions.length };
  }, [predictions]);

  const syndromeAccuracy = useMemo(() => {
    if (predictions.length === 0) return 0;
    return predictions.filter(p => p.allMatch).length / predictions.length * 100;
  }, [predictions]);

  return (
    <CRAMContext.Provider value={{
      mode, setMode, basis, M,
      params, spins, predictions, logs, calStatus,
      activeGen, isRunning, speed, setSpeed,
      addSpin, startGenerator, stopGenerator, manualCalibrate, reset, log,
      accuracy, syndromeAccuracy
    }}>
      {children}
    </CRAMContext.Provider>
  );
}

function useCRAM() { return useContext(CRAMContext); }

// ─────────────────────────────────────────────
// UI COMPONENTS
// ─────────────────────────────────────────────

const COLORS = {
  bg: "#06090f",
  surface: "#0c1118",
  border: "#1a2233",
  borderLight: "#243044",
  cyan: "#00e5a0",
  cyanDim: "rgba(0,229,160,0.12)",
  purple: "#8b5cf6",
  purpleDim: "rgba(139,92,246,0.12)",
  red: "#ef4444",
  redDim: "rgba(239,68,68,0.12)",
  amber: "#f59e0b",
  amberDim: "rgba(245,158,11,0.12)",
  text: "#e2e8f0",
  textDim: "#64748b",
  textMuted: "#475569",
};

function Card({ children, style, ...props }) {
  return (
    <div style={{
      background: COLORS.surface,
      border: `1px solid ${COLORS.border}`,
      borderRadius: 12,
      padding: 20,
      ...style
    }} {...props}>
      {children}
    </div>
  );
}

function Badge({ children, color = "cyan" }) {
  const c = COLORS[color] || color;
  const bg = COLORS[color + "Dim"] || "rgba(255,255,255,0.08)";
  return (
    <span style={{
      display: "inline-flex", alignItems: "center", gap: 4,
      padding: "3px 10px", borderRadius: 6,
      fontSize: 11, fontWeight: 600, letterSpacing: "0.05em",
      color: c, background: bg,
      border: `1px solid ${c}33`,
      textTransform: "uppercase"
    }}>{children}</span>
  );
}

function Btn({ children, onClick, active, disabled, color = "cyan", size = "md", style: sx }) {
  const c = COLORS[color] || color;
  const pad = size === "sm" ? "6px 12px" : "10px 20px";
  const fs = size === "sm" ? 12 : 13;
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        padding: pad, borderRadius: 8,
        fontSize: fs, fontWeight: 600, cursor: disabled ? "not-allowed" : "pointer",
        border: `1px solid ${active ? c : COLORS.border}`,
        background: active ? `${c}18` : "transparent",
        color: active ? c : disabled ? COLORS.textMuted : COLORS.textDim,
        opacity: disabled ? 0.4 : 1,
        transition: "all 0.15s",
        ...sx
      }}
    >{children}</button>
  );
}

// ── HEADER ──
function Header() {
  const { mode, setMode, M, isRunning, reset } = useCRAM();
  return (
    <div style={{
      display: "flex", alignItems: "center", justifyContent: "space-between",
      padding: "16px 24px",
      borderBottom: `1px solid ${COLORS.border}`,
      background: `${COLORS.bg}ee`,
      backdropFilter: "blur(12px)",
      position: "sticky", top: 0, zIndex: 50
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        <div style={{
          width: 32, height: 32, borderRadius: 8,
          background: COLORS.cyanDim, border: `1px solid ${COLORS.cyan}44`,
          display: "flex", alignItems: "center", justifyContent: "center",
          fontSize: 14, fontWeight: 900, color: COLORS.cyan
        }}>C8</div>
        <span style={{ fontSize: 18, fontWeight: 800, color: COLORS.text, letterSpacing: "-0.02em" }}>
          CRAM <span style={{ color: COLORS.cyan }}>MVP</span>
        </span>
        <Badge color={isRunning ? "cyan" : "textMuted"}>{isRunning ? "LIVE" : "IDLE"}</Badge>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <Btn active={mode === "standard"} onClick={() => setMode("standard")} size="sm">
          Standard (M={SAFE_M.toLocaleString()})
        </Btn>
        <Btn active={mode === "shadow-zero"} color="purple"
          onClick={() => setMode("shadow-zero")} size="sm">
          Shadow-Zero (M={SHADOW_M.toLocaleString()})
        </Btn>
        <Btn onClick={reset} color="red" size="sm">Reset</Btn>
      </div>
    </div>
  );
}

// ── MANUAL INPUT ──
function ManualInput() {
  const { addSpin, manualCalibrate, spins, params, M, log } = useCRAM();
  const [val, setVal] = useState("");
  const [batch, setBatch] = useState("");
  const [tab, setTab] = useState("single"); // single | batch

  const handleSubmit = () => {
    const n = parseInt(val, 10);
    if (isNaN(n) || n < 0) { log("Invalid spin value", "error"); return; }
    addSpin(n);
    setVal("");
  };

  const handleBatch = () => {
    const nums = batch.split(/[\s,;]+/).map(s => parseInt(s.trim(), 10)).filter(n => !isNaN(n) && n >= 0);
    if (nums.length === 0) { log("No valid numbers in batch", "error"); return; }
    nums.forEach((n, i) => setTimeout(() => addSpin(n), i * 30));
    setBatch("");
    log(`Batch input: ${nums.length} spins`, "info");
  };

  return (
    <Card>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 16 }}>
        <div>
          <div style={{ fontSize: 13, fontWeight: 700, color: COLORS.text, marginBottom: 2 }}>
            Manual Input
          </div>
          <div style={{ fontSize: 11, color: COLORS.textDim }}>
            Enter observed spin values from camera / log
          </div>
        </div>
        <div style={{ display: "flex", gap: 4 }}>
          <Btn active={tab === "single"} onClick={() => setTab("single")} size="sm">Single</Btn>
          <Btn active={tab === "batch"} onClick={() => setTab("batch")} size="sm">Batch</Btn>
        </div>
      </div>

      {tab === "single" ? (
        <div style={{ display: "flex", gap: 8 }}>
          <input
            type="number" min="0" value={val}
            onChange={e => setVal(e.target.value)}
            onKeyDown={e => e.key === "Enter" && handleSubmit()}
            placeholder={`0 – ${M - 1}`}
            style={{
              flex: 1, padding: "10px 14px", borderRadius: 8,
              background: COLORS.bg, border: `1px solid ${COLORS.border}`,
              color: COLORS.text, fontSize: 15, fontFamily: "monospace",
              outline: "none"
            }}
          />
          <Btn onClick={handleSubmit} active>Add Spin</Btn>
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          <textarea
            value={batch} onChange={e => setBatch(e.target.value)}
            placeholder="Paste spin values separated by commas, spaces, or newlines"
            rows={3}
            style={{
              padding: "10px 14px", borderRadius: 8,
              background: COLORS.bg, border: `1px solid ${COLORS.border}`,
              color: COLORS.text, fontSize: 13, fontFamily: "monospace",
              outline: "none", resize: "vertical"
            }}
          />
          <Btn onClick={handleBatch} active>Load Batch</Btn>
        </div>
      )}

      <div style={{
        display: "flex", alignItems: "center", justifyContent: "space-between",
        marginTop: 12, paddingTop: 12, borderTop: `1px solid ${COLORS.border}`
      }}>
        <span style={{ fontSize: 11, color: COLORS.textDim }}>
          {spins.length} spins recorded{params ? " · Calibrated" : spins.length >= 3 ? " · Ready to calibrate" : ""}
        </span>
        <Btn onClick={manualCalibrate} size="sm" active={spins.length >= 3}
          disabled={spins.length < 3}>
          Force Calibrate
        </Btn>
      </div>
    </Card>
  );
}

// ── GENERATOR SELECTOR ──
function GeneratorPanel() {
  const { startGenerator, stopGenerator, isRunning, activeGen, speed, setSpeed } = useCRAM();

  return (
    <Card>
      <div style={{ fontSize: 13, fontWeight: 700, color: COLORS.text, marginBottom: 4 }}>
        Test Generators
      </div>
      <div style={{ fontSize: 11, color: COLORS.textDim, marginBottom: 16 }}>
        Run against known PRNG implementations to validate CRAM recovery
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
        {Object.entries(GENERATORS).map(([key, gen]) => {
          const isActive = activeGen === key && isRunning;
          return (
            <button key={key} onClick={() => isActive ? stopGenerator() : startGenerator(key)}
              style={{
                display: "flex", alignItems: "flex-start", gap: 12,
                padding: "10px 14px", borderRadius: 8, textAlign: "left",
                background: isActive ? COLORS.cyanDim : "transparent",
                border: `1px solid ${isActive ? COLORS.cyan + "44" : COLORS.border}`,
                cursor: "pointer", transition: "all 0.15s"
              }}>
              <div style={{
                width: 8, height: 8, borderRadius: 4, marginTop: 4, flexShrink: 0,
                background: isActive ? COLORS.cyan : COLORS.textMuted,
                boxShadow: isActive ? `0 0 8px ${COLORS.cyan}` : "none"
              }} />
              <div>
                <div style={{ fontSize: 12, fontWeight: 600, color: isActive ? COLORS.cyan : COLORS.text }}>
                  {gen.name}
                </div>
                <div style={{ fontSize: 10, color: COLORS.textDim, marginTop: 2, lineHeight: 1.4 }}>
                  {gen.desc}
                </div>
              </div>
            </button>
          );
        })}
      </div>

      <div style={{ marginTop: 16, paddingTop: 12, borderTop: `1px solid ${COLORS.border}` }}>
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 6 }}>
          <span style={{ fontSize: 11, color: COLORS.textDim }}>Speed</span>
          <span style={{ fontSize: 11, color: COLORS.cyan, fontFamily: "monospace" }}>{speed}ms</span>
        </div>
        <input type="range" min={30} max={1000} step={10} value={speed}
          onChange={e => setSpeed(Number(e.target.value))}
          style={{ width: "100%", accentColor: COLORS.cyan }}
        />
      </div>
    </Card>
  );
}

// ── METRICS BAR ──
function Metrics() {
  const { spins, predictions, accuracy, syndromeAccuracy, params, M, calStatus } = useCRAM();
  const items = [
    { label: "Spins", value: spins.length, color: COLORS.text },
    { label: "Predictions", value: predictions.length, color: COLORS.text },
    { label: "Exact Match", value: `${accuracy.pct.toFixed(1)}%`, color: accuracy.pct > 90 ? COLORS.cyan : accuracy.pct > 0 ? COLORS.amber : COLORS.textDim },
    { label: "Syndrome Pass", value: `${syndromeAccuracy.toFixed(1)}%`, color: syndromeAccuracy > 90 ? COLORS.cyan : syndromeAccuracy > 0 ? COLORS.amber : COLORS.textDim },
    { label: "Enrichment", value: params ? `${M.toLocaleString()}×` : "—", color: COLORS.purple },
  ];

  return (
    <div style={{
      display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: 8
    }}>
      {items.map(it => (
        <Card key={it.label} style={{ padding: 14, textAlign: "center" }}>
          <div style={{ fontSize: 10, color: COLORS.textDim, marginBottom: 4, textTransform: "uppercase", letterSpacing: "0.06em" }}>
            {it.label}
          </div>
          <div style={{ fontSize: 22, fontWeight: 800, color: it.color, fontFamily: "monospace" }}>
            {it.value}
          </div>
        </Card>
      ))}
    </div>
  );
}

// ── CRT LANE DISPLAY ──
function LaneDisplay() {
  const { basis, spins, calStatus } = useCRAM();
  const last = spins.length > 0 ? spins[spins.length - 1] : null;

  return (
    <Card style={{ padding: 16 }}>
      <div style={{ fontSize: 12, fontWeight: 700, color: COLORS.text, marginBottom: 12 }}>
        CRT Lane Decomposition
      </div>
      <div style={{ display: "flex", gap: 8 }}>
        {basis.map((m, i) => {
          const laneInfo = calStatus?.lanes?.find(l => l.mod === m);
          const status = laneInfo?.status || "idle";
          const residue = last !== null ? last % m : "—";
          const borderColor = status === "ok" ? COLORS.cyan : status === "fail" ? COLORS.red : COLORS.border;

          return (
            <div key={m} style={{
              flex: 1, textAlign: "center", padding: "10px 8px",
              borderRadius: 8, background: COLORS.bg,
              border: `1px solid ${borderColor}`
            }}>
              <div style={{ fontSize: 9, color: COLORS.textDim, marginBottom: 2 }}>mod {m}</div>
              <div style={{ fontSize: 20, fontWeight: 800, fontFamily: "monospace", color: COLORS.cyan }}>
                {residue}
              </div>
              {laneInfo && status === "ok" && (
                <div style={{ fontSize: 9, color: COLORS.textMuted, marginTop: 4 }}>
                  a={laneInfo.a} c={laneInfo.c}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </Card>
  );
}

// ── PREDICTION STREAM ──
function PredictionStream() {
  const { predictions, spins } = useCRAM();
  const scrollRef = useRef(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [predictions]);

  const recent = predictions.slice(-30);

  return (
    <Card style={{ padding: 0, overflow: "hidden" }}>
      <div style={{
        padding: "12px 16px",
        borderBottom: `1px solid ${COLORS.border}`,
        display: "flex", justifyContent: "space-between", alignItems: "center"
      }}>
        <span style={{ fontSize: 12, fontWeight: 700, color: COLORS.text }}>Prediction Stream</span>
        <span style={{ fontSize: 10, color: COLORS.textDim }}>
          Last {recent.length} of {predictions.length}
        </span>
      </div>

      <div ref={scrollRef} style={{
        height: 280, overflowY: "auto", padding: "8px 12px",
        fontFamily: "monospace", fontSize: 11
      }}>
        {recent.length === 0 ? (
          <div style={{ color: COLORS.textMuted, textAlign: "center", padding: 40 }}>
            {spins.length < 20
              ? `Collecting spins... ${spins.length}/20 for auto-calibration`
              : "Awaiting predictions..."}
          </div>
        ) : (
          recent.map((p, i) => (
            <div key={i} style={{
              display: "flex", gap: 8, padding: "4px 8px", borderRadius: 4,
              marginBottom: 2,
              background: p.correct ? `${COLORS.cyan}08` : `${COLORS.red}08`
            }}>
              <span style={{ color: COLORS.textMuted, width: 32 }}>#{p.spinNum}</span>
              <span style={{ color: COLORS.textDim, width: 20 }}>
                {p.correct ? "✓" : "✗"}
              </span>
              <span style={{ color: COLORS.text }}>
                obs=<span style={{ color: COLORS.cyan }}>{p.observed}</span>
              </span>
              <span style={{ color: COLORS.text }}>
                pred=<span style={{ color: p.correct ? COLORS.cyan : COLORS.red }}>{p.predicted}</span>
              </span>
              <span style={{ color: p.delta === 0 ? COLORS.cyan : COLORS.red }}>
                Δ={p.delta}
              </span>
              <span style={{
                marginLeft: "auto",
                color: p.allMatch ? COLORS.cyan : COLORS.amber
              }}>
                {p.allMatch ? "SYN:OK" : "SYN:FAIL"}
              </span>
            </div>
          ))
        )}
      </div>
    </Card>
  );
}

// ── SYNDROME MATRIX ──
function SyndromeMatrix() {
  const { predictions, basis } = useCRAM();
  const last = predictions.length > 0 ? predictions[predictions.length - 1] : null;

  if (!last) return (
    <Card>
      <div style={{ fontSize: 12, fontWeight: 700, color: COLORS.text, marginBottom: 8 }}>
        K-Elimination Syndrome Matrix
      </div>
      <div style={{ color: COLORS.textMuted, fontSize: 11, textAlign: "center", padding: 20 }}>
        Awaiting first prediction...
      </div>
    </Card>
  );

  return (
    <Card>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
        <span style={{ fontSize: 12, fontWeight: 700, color: COLORS.text }}>
          K-Elimination Syndromes
        </span>
        <Badge color={last.allMatch ? "cyan" : "red"}>
          {last.allMatch ? "ALL PASS" : "MISMATCH"}
        </Badge>
      </div>

      <div style={{ display: "grid", gridTemplateColumns: `repeat(${Math.min(last.syndromes.length, 5)}, 1fr)`, gap: 6 }}>
        {last.syndromes.map((s, i) => (
          <div key={i} style={{
            padding: "8px 6px", borderRadius: 6, textAlign: "center",
            background: s.match ? COLORS.cyanDim : COLORS.redDim,
            border: `1px solid ${s.match ? COLORS.cyan + "33" : COLORS.red + "33"}`
          }}>
            <div style={{ fontSize: 9, color: COLORS.textDim, marginBottom: 2 }}>{s.pair}</div>
            <div style={{ fontSize: 13, fontWeight: 700, fontFamily: "monospace", color: s.match ? COLORS.cyan : COLORS.red }}>
              {s.kObs}={s.kPred}
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
}

// ── CALIBRATION STATUS ──
function CalibrationInfo() {
  const { params, calStatus, activeGen } = useCRAM();
  const genDef = activeGen ? GENERATORS[activeGen] : null;

  return (
    <Card>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
        <span style={{ fontSize: 12, fontWeight: 700, color: COLORS.text }}>Calibration</span>
        <Badge color={params ? "cyan" : "amber"}>{params ? "LOCKED" : "PENDING"}</Badge>
      </div>

      {params ? (
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
          <div>
            <div style={{ fontSize: 9, color: COLORS.textDim, textTransform: "uppercase", marginBottom: 2 }}>
              Recovered a
            </div>
            <div style={{ fontSize: 18, fontWeight: 800, fontFamily: "monospace", color: COLORS.cyan }}>
              {params.a.toLocaleString()}
            </div>
          </div>
          <div>
            <div style={{ fontSize: 9, color: COLORS.textDim, textTransform: "uppercase", marginBottom: 2 }}>
              Recovered c
            </div>
            <div style={{ fontSize: 18, fontWeight: 800, fontFamily: "monospace", color: COLORS.purple }}>
              {params.c.toLocaleString()}
            </div>
          </div>
        </div>
      ) : (
        <div style={{ fontSize: 11, color: COLORS.textMuted }}>
          {genDef
            ? `Running ${genDef.name} — collecting calibration window...`
            : "Input spins manually or select a test generator"}
        </div>
      )}

      {genDef && (
        <div style={{
          marginTop: 12, paddingTop: 12, borderTop: `1px solid ${COLORS.border}`,
          fontSize: 10, color: COLORS.textDim
        }}>
          <span style={{ fontWeight: 600 }}>Active: </span>{genDef.name}
          <br />{genDef.desc}
        </div>
      )}
    </Card>
  );
}

// ── LOG PANEL ──
function LogPanel() {
  const { logs } = useCRAM();
  const scrollRef = useRef(null);

  useEffect(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = 0;
  }, [logs]);

  const levelStyle = {
    info: { color: COLORS.textDim, icon: "›" },
    success: { color: COLORS.cyan, icon: "✓" },
    warn: { color: COLORS.amber, icon: "⚠" },
    error: { color: COLORS.red, icon: "✗" },
  };

  return (
    <Card style={{ padding: 0, overflow: "hidden" }}>
      <div style={{
        padding: "10px 16px", borderBottom: `1px solid ${COLORS.border}`,
        display: "flex", justifyContent: "space-between"
      }}>
        <span style={{ fontSize: 12, fontWeight: 700, color: COLORS.text }}>System Log</span>
        <span style={{ fontSize: 10, color: COLORS.textDim }}>{logs.length} entries</span>
      </div>
      <div ref={scrollRef} style={{
        height: 200, overflowY: "auto", padding: "6px 12px",
        fontFamily: "monospace", fontSize: 10
      }}>
        {logs.slice(0, 80).map(l => {
          const ls = levelStyle[l.level] || levelStyle.info;
          return (
            <div key={l.id} style={{ display: "flex", gap: 8, padding: "3px 0" }}>
              <span style={{ color: COLORS.textMuted, flexShrink: 0, width: 60 }}>
                {l.ts.toLocaleTimeString()}
              </span>
              <span style={{ color: ls.color, width: 12 }}>{ls.icon}</span>
              <span style={{ color: ls.color }}>{l.msg}</span>
            </div>
          );
        })}
      </div>
    </Card>
  );
}

// ── ACCURACY CHART (simple bar viz) ──
function AccuracyViz() {
  const { predictions } = useCRAM();
  if (predictions.length < 2) return null;

  // Show last 50 predictions as a hit/miss bar
  const recent = predictions.slice(-50);
  const barW = Math.max(2, Math.floor(100 / recent.length));

  return (
    <Card style={{ padding: 14 }}>
      <div style={{ fontSize: 12, fontWeight: 700, color: COLORS.text, marginBottom: 10 }}>
        Prediction Accuracy — Last {recent.length}
      </div>
      <div style={{ display: "flex", alignItems: "flex-end", gap: 1, height: 40 }}>
        {recent.map((p, i) => (
          <div key={i} style={{
            flex: 1, maxWidth: barW + "%",
            height: p.correct ? 40 : 12,
            background: p.correct ? COLORS.cyan : COLORS.red,
            borderRadius: "2px 2px 0 0",
            opacity: p.correct ? 0.8 : 0.5,
            transition: "height 0.2s"
          }} />
        ))}
      </div>
      <div style={{
        display: "flex", justifyContent: "space-between", marginTop: 6,
        fontSize: 9, color: COLORS.textDim
      }}>
        <span>← older</span>
        <span>
          <span style={{ color: COLORS.cyan }}>■</span> correct
          {" "}
          <span style={{ color: COLORS.red }}>■</span> miss
        </span>
        <span>newer →</span>
      </div>
    </Card>
  );
}

// ── EXPORT ──
function ExportPanel() {
  const { spins, predictions, params, calStatus, activeGen, M, basis, logs } = useCRAM();

  const handleExport = () => {
    const data = {
      timestamp: new Date().toISOString(),
      config: { basis, M, generator: activeGen },
      calibration: params ? { a: params.a, c: params.c, lanes: calStatus?.lanes } : null,
      spins,
      predictions: predictions.map(p => ({
        spin: p.spinNum, observed: p.observed, predicted: p.predicted,
        correct: p.correct, syndromePass: p.allMatch
      })),
      accuracy: predictions.length > 0
        ? (predictions.filter(p => p.correct).length / predictions.length * 100).toFixed(2) + "%"
        : "N/A",
      logs: logs.slice(0, 100).map(l => ({ ts: l.ts.toISOString(), level: l.level, msg: l.msg }))
    };
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `cram-session-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <Btn onClick={handleExport} size="sm" active style={{ width: "100%" }}>
      Export Session Data (JSON)
    </Btn>
  );
}

// ─────────────────────────────────────────────
// MAIN LAYOUT
// ─────────────────────────────────────────────

function App() {
  return (
    <div style={{
      minHeight: "100vh", background: COLORS.bg, color: COLORS.text,
      fontFamily: "'JetBrains Mono', 'SF Mono', 'Fira Code', monospace"
    }}>
      <Header />

      <div style={{ maxWidth: 1280, margin: "0 auto", padding: "20px 16px" }}>
        {/* Metrics row */}
        <Metrics />

        {/* Main grid */}
        <div style={{
          display: "grid",
          gridTemplateColumns: "320px 1fr",
          gap: 16, marginTop: 16
        }}>
          {/* Left panel — inputs */}
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            <ManualInput />
            <GeneratorPanel />
            <ExportPanel />
          </div>

          {/* Right panel — telemetry */}
          <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
              <LaneDisplay />
              <CalibrationInfo />
            </div>
            <PredictionStream />
            <AccuracyViz />
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
              <SyndromeMatrix />
              <LogPanel />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// ── ROOT EXPORT ──
export default function CRAMTestHarness() {
  return (
    <CRAMProvider>
      <App />
    </CRAMProvider>
  );
}
