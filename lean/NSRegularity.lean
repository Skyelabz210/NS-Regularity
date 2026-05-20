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
