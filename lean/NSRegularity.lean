-- Root of the NSRegularity formalization.
-- Each leaf file contains one or more `theorem ... := by sorry` stubs.
-- Agents close sorries; we recompose upward.
import NSRegularity.Shift
import NSRegularity.Kill113
