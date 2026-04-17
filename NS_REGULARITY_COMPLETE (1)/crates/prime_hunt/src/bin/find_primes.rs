use prime_hunt::hunter::full_hunt;
use prime_hunt::multi_hydra::build_multi_gap_hydra;

fn main() {
    let limit: u64 = 1_000_000;
    let cal_end:  u64 = 50_000;
    let hunt_end: u64 = limit;

    println!("=== Hydra Prime Hunt ===");
    println!("Calibration: [2, {})", cal_end);
    println!("Hunt range:  [{}, {})", cal_end, hunt_end);
    println!("Recall target: 100% (zero false negatives)\n");

    let hydra = build_multi_gap_hydra();
    let (hunter, result) = full_hunt(hydra, (2, cal_end), (cal_end, hunt_end), 1.00);

    println!("--- Hunt Results ---");
    println!("Strata targeted / total: {} / {}",
        hunter.targeted_count(), hunter.total_strata());
    println!("Candidates tested:  {}", result.candidates_tested);
    println!("Candidates skipped: {}", result.candidates_skipped);
    println!("Speedup:            {:.3}x", result.speedup);
    println!("Recall:             {:.6}", result.recall);
    println!("False negatives:    {}", result.false_negatives);
    println!("Primes found:       {}\n", result.primes_found.len());

    // Print all found primes
    println!("--- Primes in [2, {}) ---", limit);
    // Include primes below cal_end via simple sieve
    let mut all_primes: Vec<u64> = Vec::new();
    let mut sieve = vec![true; cal_end as usize];
    sieve[0] = false; sieve[1] = false;
    for i in 2..cal_end as usize {
        if sieve[i] {
            all_primes.push(i as u64);
            let mut j = i * 2;
            while j < cal_end as usize { sieve[j] = false; j += i; }
        }
    }
    all_primes.extend_from_slice(&result.primes_found);

    // Print in rows of 10
    for (i, p) in all_primes.iter().enumerate() {
        if i % 10 == 0 { print!("\n  "); }
        print!("{:>8}", p);
    }
    println!("\n");
    println!("Total primes below {}: {}", limit, all_primes.len());
}
