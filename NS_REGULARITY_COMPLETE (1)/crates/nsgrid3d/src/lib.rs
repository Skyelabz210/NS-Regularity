pub mod ns_grid_3d;
pub mod biot_savart;
pub mod schema_sweep;
pub mod dual_tray;
pub mod schema_program;
pub mod fused_solver;
pub mod inline_oracles;
pub mod cram_ns_bridge;
pub mod regularity_tracker;

// Wave3 additions (OP-3 + diagnostics)
pub mod biot_savart_ntt;
pub mod biot_savart_exact;
pub mod dkam_monitor;
pub mod sobolev_winding;
pub mod cram_amplified;
