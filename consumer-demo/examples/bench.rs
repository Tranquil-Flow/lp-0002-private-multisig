use lp0002_private_multisig_core::{
    prove_threshold, verify_threshold_receipt, MemberSecret, MultisigConfig, Proposal,
    ProposalAction,
};
use lp0002_private_multisig_verifier::VerifierProgram;
use std::time::Instant;

fn main() {
    println!("LP-0002 Private Multisig Benchmarks (safe-lane, SHA-256 mock receipt)");
    println!("====================================================================");
    println!();

    let configs = [
        ("2-of-3", 2u16, 3u16),
        ("3-of-5", 3, 5),
        ("5-of-10", 5, 10),
        ("10-of-20", 10, 20),
        ("25-of-50", 25, 50),
    ];

    // Print header
    println!(
        "{:<12} | {:>12} | {:>12} | {:>14} | {:>14} | {:>10}",
        "Config", "Config::new", "prove_thresh", "verify_receipt", "execute_if_met", "proof_bytes"
    );
    println!(
        "{:-<12}-+-{:-<12}-+-{:-<12}-+-{:-<14}-+-{:-<14}-+-{:-<10}",
        "", "", "", "", "", ""
    );

    for &(label, threshold, member_count) in &configs {
        let iterations: u32 = 1000;

        // Create members
        let members: Vec<MemberSecret> = (0..member_count)
            .map(|i| MemberSecret::from_seed(format!("bench-member-{}-{}", label, i).as_bytes()))
            .collect();

        // Benchmark MultisigConfig::new
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = MultisigConfig::new(label, threshold, &members).unwrap();
        }
        let config_new_time = start.elapsed().as_nanos() as f64 / iterations as f64;

        // Create the actual config and proposal for subsequent benchmarks
        let config = MultisigConfig::new(label, threshold, &members).unwrap();
        let proposal = Proposal::new("bench-proposal", "benchmark action: transfer 100 tokens");

        // Benchmark prove_threshold
        let start = Instant::now();
        let mut proof = None;
        for _ in 0..iterations {
            proof =
                Some(prove_threshold(&config, &proposal, &members[..threshold as usize]).unwrap());
        }
        let prove_time = start.elapsed().as_nanos() as f64 / iterations as f64;
        let proof = proof.unwrap();

        // Benchmark verify_threshold_receipt
        let start = Instant::now();
        for _ in 0..iterations {
            verify_threshold_receipt(&config, &proposal, &proof).unwrap();
        }
        let verify_time = start.elapsed().as_nanos() as f64 / iterations as f64;

        // Benchmark VerifierProgram::execute_if_threshold_met
        let start = Instant::now();
        for _ in 0..iterations {
            let mut verifier = VerifierProgram::default();
            let action = ProposalAction::Transfer {
                to: "logos1_bench_recipient".into(),
                amount: 100,
                denom: "LOGOS".into(),
            };
            verifier
                .execute_if_threshold_met(&config, &proposal, &proof, action.clone())
                .unwrap();
        }
        let execute_time = start.elapsed().as_nanos() as f64 / iterations as f64;

        // Measure proof size
        let proof_size = proof.public_bytes().len();

        fn fmt_time(ns: f64) -> String {
            if ns < 1_000.0 {
                format!("{:.0} ns", ns)
            } else if ns < 1_000_000.0 {
                format!("{:.1} μs", ns / 1_000.0)
            } else {
                format!("{:.1} ms", ns / 1_000_000.0)
            }
        }

        println!(
            "{:<12} | {:>12} | {:>12} | {:>14} | {:>14} | {:>10}",
            label,
            fmt_time(config_new_time),
            fmt_time(prove_time),
            fmt_time(verify_time),
            fmt_time(execute_time),
            format!("{} B", proof_size),
        );
    }

    println!();
    println!("Notes:");
    println!("- Safe-lane benchmarks using SHA-256 mock receipt (no RISC0 proving)");
    println!("- Each operation averaged over 1000 iterations");
    println!("- RISC0 heavy-lane proof measurements are recorded in submission/BENCHMARKS.md");
    println!(
        "- LEZ localnet payload/cost evidence is recorded in submission/LEZ_COST_BENCHMARKS.json"
    );
}
