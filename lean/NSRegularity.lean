-- Root of the NSRegularity formalization.
-- Each leaf file contains one or more `theorem ... := by sorry` stubs.
-- Agents close sorries; we recompose upward.
import NSRegularity.Shift
import NSRegularity.Kill113          -- M1
import NSRegularity.DKAM              -- M2
import NSRegularity.HSquared          -- M3
import NSRegularity.AubinLions        -- M4
import NSRegularity.Adelic            -- M5
import NSRegularity.Continuum         -- M6
import NSRegularity.NonlinearPassage  -- M7
import NSRegularity.BKM               -- M8
import NSRegularity.ThreeStream       -- M9
import NSRegularity.GlobalRegularity  -- top-level composition
import NSRegularity.GlobalRegularityGeneric  -- generic L∞-preserving step closure
import NSRegularity.NS.FullStep  -- full NS step (diffusion + advection + stretching)
import NSRegularity.Adelic.LaplacianAsPolyOp  -- worked example: Laplacian descent
import NSRegularity.Shift.SumGrid  -- 3D shift-sum invariance lemmas
import NSRegularity.HSquared.SBP  -- 3D SBP cross-term reindex lemmas
import NSRegularity.HSquared.ResolutionA  -- L3.4 Resolution A H² monotonicity
import NSRegularity.AubinLions.ResolutionA  -- L4.2 Resolution A L² monotonicity
import NSRegularity.NS.BiotSavart  -- discrete curl + Biot-Savart skeleton (axiomatic)
import NSRegularity.NS.Trajectory  -- NS trajectory generator + global regularity skeleton
