/-
  NSTorus.lean — Root module for the NS Torus formalization

  Imports all submodules in dependency order:
    1. Axioms      — A1–A6: foundational axioms of the CRAM/DKAM system
    2. Definitions — D1–D6: DiscreteVelocityField, discreteEnergy, etc.
    3. Lemmas      — L1–L3: energy identity, uniform bounds, time derivative
    4. Compactness — T1–T4: weak/strong compactness, nonlinear limit, continuum bridge
    5. DKAM        — DKAM subcriticality (deg S < ρ B) and D-030 winding-energy bound

  Build with: lake build
  Known gaps (by design, marked `sorry`):
    D-024: uniform bound transfer to continuum
    D-026: ω_crt = ω_phys identification
    D-030-A/B: curl definition and discrete Cauchy-Schwarz
    D-016: full discrete regularity induction
-/

import NSTorus.Axioms
import NSTorus.Definitions
import NSTorus.Lemmas
import NSTorus.Compactness
import NSTorus.DKAM
