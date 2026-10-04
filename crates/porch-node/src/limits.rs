//! Local operator caps. Peers cannot enlarge them; protocol ceilings stay fixed.
use anyhow::{Result, ensure};
use porch_core::{MAX_BLOB, MAX_FRAME, MAX_INPUT};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LimitsConfig {
    pub request_body_bytes: usize,
    pub response_body_bytes: usize,
    pub peer_frame_bytes: usize,
    pub input_bytes: usize,
    pub blob_bytes: usize,
    pub storage_total_bytes: u64,
    pub message_bytes: usize,
    pub concurrent_jobs: usize,
    pub queued_jobs: usize,
    pub ingress_requests: usize,
    pub connections: u32,
    pub services: usize,
    pub grants: u64,
    pub model_timeout_ms: u64,
    pub idle_peer_seconds: u64,
}
impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            request_body_bytes: MAX_BLOB * 2 + 4096,
            response_body_bytes: MAX_BLOB * 2 + 4096,
            peer_frame_bytes: MAX_FRAME,
            input_bytes: MAX_INPUT,
            blob_bytes: MAX_BLOB,
            storage_total_bytes: 128 * 1024 * 1024,
            message_bytes: 4096,
            concurrent_jobs: 2,
            queued_jobs: 64,
            ingress_requests: 32,
            connections: 64,
            services: 128,
            grants: 1000,
            model_timeout_ms: 30000,
            idle_peer_seconds: 120,
        }
    }
}
impl LimitsConfig {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            (4096..=MAX_BLOB * 2 + 4096).contains(&self.request_body_bytes)
                && (4096..=MAX_BLOB * 2 + 4096).contains(&self.response_body_bytes)
                && (4096..=MAX_FRAME).contains(&self.peer_frame_bytes)
                && (1..=MAX_INPUT).contains(&self.input_bytes)
                && (41..=MAX_BLOB).contains(&self.blob_bytes)
                && (MAX_BLOB as u64..=1024 * 1024 * 1024 * 1024)
                    .contains(&self.storage_total_bytes)
                && (1..=4096).contains(&self.message_bytes)
                && (1..=16).contains(&self.concurrent_jobs)
                && (1..=64).contains(&self.queued_jobs)
                && (1..=64).contains(&self.ingress_requests)
                && (1..=64).contains(&self.connections)
                && (4..=128).contains(&self.services)
                && (1..=1000).contains(&self.grants)
                && (100..=30000).contains(&self.model_timeout_ms)
                && (15..=120).contains(&self.idle_peer_seconds),
            "OPERATOR_LIMITS_OUT_OF_BOUNDS"
        );
        Ok(())
    }
}
