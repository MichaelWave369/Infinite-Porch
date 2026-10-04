//! Local microbenchmark; no physical-network or provider performance claim.
use anyhow::Result;
use libp2p_identity::Keypair;
use porch_core::Signed;
use serde_json::json;
use std::time::Instant;

fn main() -> Result<()> {
    let key = Keypair::generate_ed25519();
    let payload = json!({"probe":"public receipt baseline","input_digest":"0".repeat(64),"output_digest":"1".repeat(64)});
    let started = Instant::now();
    let mut receipts = Vec::new();
    for _ in 0..100 {
        receipts.push(Signed::new(
            &key,
            "porch.performance.receipt.v1",
            payload.clone(),
        )?);
    }
    let sign_us = started.elapsed().as_micros();
    let started = Instant::now();
    for receipt in receipts {
        receipt.verify("porch.performance.receipt.v1")?;
    }
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"environment":"LOCAL_MICROBENCHMARK","iterations":100,"sign_total_microseconds":sign_us,"verify_total_microseconds":started.elapsed().as_micros(),"physical_performance":"UNVERIFIED","scope":"domain-separated JSON receipt assertions; no model or LAN timing"})
        )?
    );
    Ok(())
}
