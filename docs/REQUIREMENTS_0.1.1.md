# 0.1.1 requirement-to-build register

Authority: unchanged CONTROLLING_SPEC_0.1.1.txt, SHA-256
`37a97328fa947db3f249d26d99da6d47efe8519fb5f6780cb3dddf89a2d2e5c7`.
Each Q identifier maps to the same-numbered section. REQUIREMENTS.md remains the
historical 0.1.0 register. The JSON companion records dependencies and deviations;
deviation details are in receipts/qualification/deviations.json.

“Locally demonstrated” means owned processes and loopback sockets. Provider
fixtures and explicit mock outputs do not prove physical LAN or model weights.
Receipt filenames below are relative to receipts/qualification unless identified
as a guide, UI artifact or final outer release-evidence. Final commit/binaries
and package results are collected after this source register is committed.

| ID / section | Requirement | Implementation / verification | Status |
| --- | --- | --- | --- |
| Q-001 | Inspect and preserve baseline | baseline tag; freeze receipt; continued branch; specification-freeze.json; git history | Executed locally |
| Q-002 | Consistent lowest supported Rust compiler | Cargo, rust-toolchain, README, CI; check_toolchain.py; msrv-1.88.log; rust-release-gate.log; toolchain-consistency.log | Executed locally |
| Q-003 | First-class qualification mode | qualification.rs; CLI qualify; Qualification.tsx; qualification-acceptance.json; ui-receipt.json | Demonstrated locally |
| Q-004 | Two independent physical machines and governed inference | host/join; provider; bounded grants; signed receipts; physical/platform guides; qualification-acceptance.json (mock/loopback); live-ollama.json | Integrated; physical UNVERIFIED |
| Q-005 | Separate live Ollama qualification and availability states | models.rs scan/refresh/verify; provider_runtime.rs; qualify_ollama.py; rust-release-gate.log; live-ollama.json (2 skipped, 3 unverified) | Executed locally with HTTP fixtures; weights UNVERIFIED |
| Q-006 | Privacy classes and no unauthorized prompt forwarding | deterministic route; executor grant boundary; digest-only refusal evidence; peer-acceptance.json; qualification-acceptance.json; Rust privacy/security tests | Demonstrated locally |
| Q-007 | Internet-off LAN independence | offline/restored phases; path checks; WAN attestation; OFFLINE_LAN_TEST.md; qualification-acceptance.json proves label boundaries, not WAN removal | Implemented procedure; physical UNVERIFIED |
| Q-008 | Controlled failure injection | faults.rs; provider_runtime.rs; process_failures.py; recovery.rs; rust-release-gate.log; process-failures.json; process-failures-quic.json | Executed locally |
| Q-009 | Restart preserves authority and uncertain jobs | state lock; persistent identity/trust/grants; pinned routes; UNCERTAIN recovery; peer-acceptance.json; process failure receipts; Rust recovery tests | Demonstrated locally; physical durability UNVERIFIED |
| Q-010 | Authenticated actual network-path diagnostics | network.rs connection IDs/sockets/protocol/RTT/age; advanced UI; transports.json; qualification-acceptance.json | Demonstrated locally; LAN/WAN UNVERIFIED |
| Q-011 | Clear safe first pairing and fingerprint | recipient-bound invite; full SHA-256 comparison; batched pinned dial fallback; qualification-acceptance.json; Rust invitation/fingerprint tests | Demonstrated locally |
| Q-012 | Windows readiness without false qualification | MSVC CI; install/qualify PowerShell helpers; user ACL and firewall guide; platform-contracts.json (CLI contracts only); WINDOWS_QUALIFICATION.md | Implemented; native UNVERIFIED |
| Q-013 | macOS readiness without false qualification | native-source guide; user permissions/firewall; CI; signing limits; MACOS_QUALIFICATION.md; platform-contracts.json | Implemented; native UNVERIFIED |
| Q-014 | Linux complete qualification path | install-user.sh; systemd template; two-PC install/share/storage/revoke/restart guide; LINUX_QUALIFICATION.md; final release-evidence/package.json | Integrated; local native package gate collected post-commit |
| Q-015 | Standard sanitized signed evidence bundle | 6 whitelisted files plus signed manifest; CLI/UI export; offline validator; example-sanitized-bundle/; example-validation.json; ui-bundle-validation.json | Demonstrated locally |
| Q-016 | Six explicit outcome states and skip discipline | scenario schema/counts; UI; validator; no production badge; qualification-acceptance.json; example signed bundle; UI receipt | Demonstrated locally |
| Q-017 | Physical-network security review and regressions | SECURITY.md; THREAT_MODEL.md; exact grants; codec; aliases; caps; safe_error; Rust security/caps/faults tests; qualification/API/UI acceptance | Executed locally; independent review UNVERIFIED |
| Q-018 | Separate and hardened localhost control plane | literal loopback bind/URLs; same origin/Host; strong token; authenticated SSE; qualification-acceptance.json; peer-acceptance.json; UI receipt | Demonstrated locally |
| Q-019 | Safe operator-configurable resource caps | limits.rs; offline configure-limits; queue/frame/body/grant/storage caps; limit-cli.json; Rust caps and wire-codec tests | Demonstrated locally |
| Q-020 | Inspect deterministic routing facts | existing route decision with candidates, reasons, selected peer and grant facts; peer-acceptance.json; example run.json/receipts.json | Demonstrated locally |
| Q-021 | Dedicated honest Qualification UI | Qualification.tsx; 10-page navigation; counts/provider/path/export; CANDIDATE; ui-receipt.json; qualification.png; ui-bundle-validation.json | Demonstrated locally |
| Q-022 | Clear physical CLI workflow | qualify host/join/status/run/export/validate; models alias; grants; doctor; qualification-acceptance.json; platform-contracts.json; CLI.md | Demonstrated locally |
| Q-023 | Fourteen-category doctor with remediation | qualification::doctor; CLI/API; explicit unknown LAN/NAT/platform results; qualification-acceptance.json; CLI smoke; package doctor gate | Demonstrated locally |
| Q-024 | Bounded wall-clock safety and monotonic durations | persistent last security time; 2-second future-created skew; rollback refusal/anomaly; Rust security tests; SECURITY.md; db.rs | Executed locally |
| Q-025 | Schema version, private backup and migration | schema 2; owner check before DDL; schema-1 backup; collision/future failure; Rust qualification/caps migration regressions; rust-release-gate.log | Executed locally |
| Q-026 | Integrity metadata, lockfiles and unsigned package | package.py; verify_package.py; build.rs; package_acceptance.py; final portable manifest and release-evidence/package.json; outer manifest | Implemented; native artifact gate collected post-commit |
| Q-027 | Supply-chain tooling and classified findings | audit_gate.py; raw cargo/npm audit; exact feature/source applicability checks; supply-chain-review.json; SUPPLY_CHAIN_REVIEW.md; audit-complete.log | Executed locally with retained findings |
| Q-028 | Honest performance baseline by environment | qualification harness timers; Python SHA-256 sample; Rust receipt microbenchmark; qualification-acceptance.json; receipt-performance.json; PERFORMANCE.md | Executed locally; idle metrics and physical UNVERIFIED |
| Q-029 | Preserve non-goals | no new federation, public relay, payments, shell, sharding, GPU pooling or marketplace engine; ROADMAP.md; current diff and preserved protocol | Preserved |
| Q-030 | CI software gates without physical claims | 3-platform CI; Linux UI/audit; native package/schema/migration/security gates; ci.yml; local command-gates.json; TESTING.md | Configured; remote execution UNVERIFIED |
| Q-031 | Complete required guides and status distinctions | START_HERE; README; all requested qualification/security/release/roadmap guides; docs/ guides; DELIVERY.md; REQUIREMENTS_0.1.1.md | Implemented |
| Q-032 | Separate software completion and external evidence gates | RELEASE_GATES.md; selected local gates; final post-commit package verification; command-gates.json; final delivery report; release-evidence/ | Software evidence collected; physical UNVERIFIED |
| Q-033 | Complete versioned deliverable with source/history/binaries/evidence | git archive/bundle; native Linux portable/UI; 16-item DELIVERY_REPORT.md; outer REPOSITORY_STATE.json; delivery manifest; archive checksum | Prepared by committed package tooling; final collection post-commit |
| Q-034 | Persist through inspect/build/fix/test/package | preserved failures; repaired source; final selected checks; reviewable package; raw receipts; git history; DELIVERY.md | Executed locally |
| Q-035 | Make physical success criterion testable and falsifiable | fixed public probes; signed grants/receipts; recovery; phase exports; exact physical guides; PHYSICAL_QUALIFICATION.md; RELEASE_GATES.md; final bundle | Integrated; external success UNVERIFIED |

The milestone excludes the section-29 engines. Remaining physical gates are
requirements to prove externally, not removed scope. Software gate results are
in command-gates.json; the final native-package gate and all 16 requested
delivery details are in the outer DELIVERY_REPORT.md. No user release decision,
independent review or physical result is inferred from this register.
