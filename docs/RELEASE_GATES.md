# 0.1.1 release gates

Engineering candidate completion requires operational tooling and passing local
software gates. It does not complete external physical or production qualification.

| Software gate | Evidence |
| --- | --- |
| Format, clippy, Rust builds/tests on tested minimum compiler | qualification/command-gates.json and raw logs |
| SDK, CLI, UI and original 33 peer checks | executed local receipts |
| Security, migration, canonicalization, mock/provider-contract regressions | Rust tests, qualification acceptance and process failure receipts |
| Qualification host/join/status/run/export/validate | qualification acceptance and example signed sanitized bundle |
| Package manifest, launch/UI/CLI checks | package receipt |
| Platform and physical procedures | Windows/Linux/macOS/Ollama/offline/failure guides |
| Supply-chain review | raw cargo/npm audits plus reviewed applicability gate |

| External evidence gate | Candidate status |
| --- | --- |
| Two separately owned physical nodes and actual LAN | UNVERIFIED |
| Recipient-bound pairing and human fingerprint confirmation on real PCs | UNVERIFIED |
| Real model advertisement, bounded grant, remote Ollama weights, signed receipt | UNVERIFIED |
| Revocation refusal and restart/network interruption on real PCs | UNVERIFIED |
| WAN disconnected with LAN messaging and inference continuing | UNVERIFIED |
| Native Windows build, paths, ACLs, firewall and runtime | UNVERIFIED |
| Native macOS if claiming macOS support | UNVERIFIED |
| Independent physical-network security review | UNVERIFIED |
| Official binary signing / production release authorization | UNVERIFIED / human-owned |

CI is configured for Linux/Windows/macOS and does not pretend to prove physical
networking. Local checks are not remote CI executions. Qualifier labels and node
signatures cannot substitute for the external evidence rows. Keep failed and
skipped results. No production release is authorized by this build.
