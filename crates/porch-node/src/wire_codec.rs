//! Same protocol-1 JSON/EOF wire format, with a strict byte ceiling in both directions.
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use libp2p::{StreamProtocol, request_response::Codec};
use porch_core::{Signed, WireRequest, WireResponse};
use serde::{Serialize, de::DeserializeOwned};
use std::io;
#[derive(Clone)]
pub struct BoundedJsonCodec {
    limit: usize,
}
impl BoundedJsonCodec {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }
    async fn read<T: AsyncRead + Unpin + Send, V: DeserializeOwned>(
        &self,
        io: &mut T,
    ) -> io::Result<V> {
        let mut bytes = Vec::new();
        io.take(self.limit as u64 + 1)
            .read_to_end(&mut bytes)
            .await?;
        if bytes.len() > self.limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "FRAME_TOO_LARGE",
            ));
        }
        serde_json::from_slice(&bytes).map_err(io::Error::other)
    }
    async fn write<T: AsyncWrite + Unpin + Send, V: Serialize>(
        &self,
        io: &mut T,
        value: V,
    ) -> io::Result<()> {
        let bytes = serde_json::to_vec(&value).map_err(io::Error::other)?;
        if bytes.len() > self.limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "FRAME_TOO_LARGE",
            ));
        }
        io.write_all(&bytes).await
    }
}
#[async_trait::async_trait]
impl Codec for BoundedJsonCodec {
    type Protocol = StreamProtocol;
    type Request = Signed<WireRequest>;
    type Response = Signed<WireResponse>;
    async fn read_request<T: AsyncRead + Unpin + Send>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Request> {
        self.read(io).await
    }
    async fn read_response<T: AsyncRead + Unpin + Send>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response> {
        self.read(io).await
    }
    async fn write_request<T: AsyncWrite + Unpin + Send>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        value: Self::Request,
    ) -> io::Result<()> {
        self.write(io, value).await
    }
    async fn write_response<T: AsyncWrite + Unpin + Send>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        value: Self::Response,
    ) -> io::Result<()> {
        self.write(io, value).await
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn strict_limit_rejects_padding_and_oversized_writes() {
        let codec = BoundedJsonCodec::new(8);
        let mut exact = futures::io::Cursor::new(b"{}      ".to_vec());
        assert!(codec.read::<_, serde_json::Value>(&mut exact).await.is_ok());
        let mut padded = futures::io::Cursor::new(b"{}       ".to_vec());
        assert_eq!(
            codec
                .read::<_, serde_json::Value>(&mut padded)
                .await
                .unwrap_err()
                .to_string(),
            "FRAME_TOO_LARGE"
        );
        let mut out = futures::io::Cursor::new(Vec::new());
        assert!(
            codec
                .write(&mut out, serde_json::json!({"oversized":true}))
                .await
                .is_err()
        );
        assert!(out.into_inner().is_empty());
    }
}
