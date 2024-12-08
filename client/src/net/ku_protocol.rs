use async_trait::async_trait;
use bincode;
use futures::io::{AsyncRead, AsyncWrite};
use libp2p::request_response::Codec;
use libp2p::StreamProtocol;
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRequest {
    pub file_name: String,
    pub target_path: String,
    pub save_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResponse {
    // pub is_success: bool,
    // pub message: String,
    pub file_name: String,
    pub src_path: String,
    pub tgt_path: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct KuFileTransferCodec();

#[async_trait]
impl Codec for KuFileTransferCodec {
    type Protocol = StreamProtocol;
    type Request = FileRequest;
    type Response = FileResponse;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        let bytes_read = futures::io::AsyncReadExt::read_to_end(io, &mut buf).await?;
        tracing::debug!("Bytes read: {}", bytes_read);
        bincode::deserialize(&buf).map_err(|e| {
            tracing::error!("Deserialization error: {:?}", e);
            io::Error::new(io::ErrorKind::InvalidData, "Failed to deserialize request")
        })
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        let bytes_read = futures::io::AsyncReadExt::read_to_end(io, &mut buf).await?;
        tracing::debug!("Bytes read: {}", bytes_read);

        match bincode::deserialize(&buf) {
            Ok(response) => Ok(response),
            Err(e) => {
                tracing::error!("Deserialization failed: {:?}", e);
                Err(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            }
        }
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        request: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = bincode::serialize(&request).unwrap();
        tracing::debug!("Serialized request size: {}", data.len());
        futures::io::AsyncWriteExt::write_all(io, &data).await?;
        tracing::debug!("Request written successfully.");
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        response: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = bincode::serialize(&response).unwrap();
        tracing::debug!("Serialized response size: {}", data.len());
        futures::io::AsyncWriteExt::write_all(io, &data).await?;
        tracing::debug!("Response written successfully.");
        Ok(())
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Message(pub String);

// #[derive(Clone)]
// pub struct MessagingCodec;

// #[async_trait]
// impl Codec for MessagingCodec {
//     type Protocol = StreamProtocol;
//     type Request = Message;
//     type Response = Message;

//     async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
//     where
//         T: futures::AsyncRead + Unpin + Send,
//     {
//         let mut buf = Vec::new();
//         futures::io::AsyncReadExt::read_to_end(io, &mut buf).await?;
//         let request: Message = bincode::deserialize(&buf).unwrap();
//         Ok(request)
//     }

//     async fn read_response<T>(
//         &mut self,
//         _: &Self::Protocol,
//         io: &mut T,
//     ) -> io::Result<Self::Response>
//     where
//         T: futures::AsyncRead + Unpin + Send,
//     {
//         let mut buf = Vec::new();
//         futures::io::AsyncReadExt::read_to_end(io, &mut buf).await?;
//         let response: Message = bincode::deserialize(&buf).unwrap();
//         Ok(response)
//     }

//     async fn write_request<T>(
//         &mut self,
//         _: &Self::Protocol,
//         io: &mut T,
//         request: Self::Request,
//     ) -> io::Result<()>
//     where
//         T: futures::AsyncWrite + Unpin + Send,
//     {
//         let buf = bincode::serialize(&request).unwrap();
//         futures::io::AsyncWriteExt::write_all(io, &buf).await?;
//         Ok(())
//     }

//     async fn write_response<T>(
//         &mut self,
//         _: &Self::Protocol,
//         io: &mut T,
//         response: Self::Response,
//     ) -> io::Result<()>
//     where
//         T: futures::AsyncWrite + Unpin + Send,
//     {
//         let buf = bincode::serialize(&response).unwrap();
//         futures::io::AsyncWriteExt::write_all(io, &buf).await?;
//         Ok(())
//     }
// }
