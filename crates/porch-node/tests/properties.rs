//! Seeded bounded property checks; manual CI can expand the same reproducible corpus.
use anyhow::Result;
use porch_core::*;
use porch_node::{Config, Node, field, qualification};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde_json::json;
fn cases() -> usize {
    std::env::var("PORCH_PROPERTY_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(128usize)
        .clamp(1, 65536)
}
#[test]
fn canonical_signatures_and_parser_mutations_fail_closed() -> Result<()> {
    let key = libp2p::identity::Keypair::generate_ed25519();
    let mut rng = StdRng::seed_from_u64(0x369012);
    for i in 0..cases() {
        let v = json!({"z":rng.r#gen::<u64>(),"a":{"case":i,"public":"probe"},"array":[1,2,3]});
        let signed = Signed::new(&key, "porch.property.v1", v.clone())?;
        signed.verify("porch.property.v1")?;
        assert!(signed.verify("porch.invite.v1").is_err());
        let mut tampered = signed.clone();
        tampered.payload["a"]["case"] = json!(i + 1);
        assert!(tampered.verify("porch.property.v1").is_err());
        let mut bytes = serde_json::to_vec(&signed)?;
        let at = rng.gen_range(0..bytes.len());
        bytes[at] ^= rng.gen_range(1..=255);
        if let Ok(mutated) = serde_json::from_slice::<Signed<serde_json::Value>>(&bytes)
            && (canonical(&mutated.payload)? != canonical(&v)?
                || mutated.signature != signed.signature
                || mutated.signer != signed.signer
                || mutated.public_key != signed.public_key
                || mutated.domain != signed.domain)
        {
            assert!(mutated.verify("porch.property.v1").is_err());
        }
    }
    Ok(())
}
#[test]
fn invitation_and_grant_authority_boundaries_hold_for_seeded_cases() -> Result<()> {
    let root = tempfile::tempdir()?;
    let node = Node::open(root.path(), Config::default())?;
    node.create_porch("Property Porch")?;
    let peer = libp2p::identity::Keypair::generate_ed25519()
        .public()
        .to_peer_id()
        .to_string();
    let mut rng = StdRng::seed_from_u64(0x45);
    for _ in 0..cases() {
        let invite = node.invite(&json!({"recipient":peer}))?;
        invite.verify("porch.invite.v1")?;
        let mut forged = invite.clone();
        forged.payload.recipient = Some(node.id.clone());
        assert!(forged.verify("porch.invite.v1").is_err());
        let time = rng.gen_range(100000..=1000000);
        let lifetime = rng.gen_range(1..86400);
        let g = Grant {
            issuer: node.id.clone(),
            recipient: peer.clone(),
            capability: "model.inference".into(),
            resource: "exact-model".into(),
            action: "run".into(),
            porch: None,
            limits: Limits::default(),
            created_at: time,
            expires_at: time + lifetime,
            nonce: nonce(),
            exact_input_digest: None,
        };
        assert!(
            g.validate(
                &node.id,
                &peer,
                "model.inference",
                "exact-model",
                "run",
                time
            )
            .is_ok()
        );
        for wrong in [time - 1, time + lifetime, time + lifetime + 1] {
            assert!(
                g.validate(
                    &node.id,
                    &peer,
                    "model.inference",
                    "exact-model",
                    "run",
                    wrong
                )
                .is_err()
            );
        }
        assert!(
            g.validate(
                &node.id,
                &node.id,
                "model.inference",
                "exact-model",
                "run",
                time
            )
            .is_err()
        );
        assert!(
            g.validate(
                &node.id,
                &peer,
                "model.inference",
                "another-model",
                "run",
                time
            )
            .is_err()
        );
    }
    Ok(())
}
#[tokio::test]
async fn evidence_mutations_and_class_promotion_are_rejected() -> Result<()> {
    let root = tempfile::tempdir()?;
    let node = Node::open(root.path(), Config::default())?;
    qualification::run(&node, &json!({"environment":"NATIVE_HOSTED"})).await?;
    let bundle = qualification::export(&node)?;
    qualification::validate_export(&bundle)?;
    for name in [
        "run.json",
        "node.json",
        "results.json",
        "receipts.json",
        "ledger-verification.json",
        "sanitized-log.json",
    ] {
        let mut v = bundle.clone();
        v["files"][name] = json!({"tampered":true});
        assert!(qualification::validate_export(&v).is_err());
    }
    let snapshot = field::snapshot(
        &node,
        &json!({"session":"property","environment":"NATIVE_HOSTED"}),
    )?;
    let mut p = snapshot["payload"].clone();
    p["environment"] = json!("PHYSICAL_LAN");
    let signed = serde_json::to_value(Signed::new(&node.key, "porch.field.participant.v1", p)?)?;
    assert!(field::validate_snapshot(&signed).is_err());
    assert_eq!(field::correlate(&snapshot, &snapshot)?["state"], "FAILED");
    let mut v = bundle.clone();
    v["manifest"]["payload"]["files"]["../../identity.key"] = json!("x");
    assert!(qualification::validate_export(&v).is_err());
    Ok(())
}
#[test]
fn stored_metadata_rejects_unknown_caps_and_corrupt_content_ids() {
    for n in 0..cases() {
        let s = format!("../blob-{n}");
        assert!(validate_cid(&s).is_err());
    }
    let mut config = serde_json::to_value(Config::default()).unwrap();
    config["limits"]["remote_shell"] = json!(true);
    assert!(serde_json::from_value::<Config>(config).is_err());
}
