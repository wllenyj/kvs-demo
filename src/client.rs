use crate::common::{Request, Response};
use crate::{KvsError, Result};
use std::net::SocketAddr;
use romio::TcpStream;
use futures::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use byteorder::{ByteOrder, BigEndian};


/// Key value store client
pub struct KvsClient {
    read_half: ReadHalf<TcpStream>,
    write_half: WriteHalf<TcpStream>,
}

impl KvsClient {
    /// Connect to `addr` to access `KvsServer`.
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(&addr).await?;
        let (read_half, write_half) = stream.split();
        
        Ok(KvsClient {
            read_half,
            write_half,
        })
    }

    /// Get the value of a given key from the server.
    pub async fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.send_request(Request::Get { key }).await? {
            Some(Response::Get(value)) => Ok(value),
            Some(Response::Err(msg)) => Err(KvsError::StringError(msg)),
            Some(_) => Err(KvsError::StringError("Invalid response".to_owned())),
            None => Err(KvsError::StringError("No response received".to_owned())),
        }
    }

    /// Set the value of a string key in the server.
    pub async fn set(&mut self, key: String, value: String) -> Result<()> {
        match self.send_request(Request::Set { key, value }).await? {
            Some(Response::Set) => Ok(()),
            Some(Response::Err(msg)) => Err(KvsError::StringError(msg)),
            Some(_) => Err(KvsError::StringError("Invalid response".to_owned())),
            None => Err(KvsError::StringError("No response received".to_owned())),
        }
    }

    /// Remove a string key in the server.
    pub async fn remove(&mut self, key: String) -> Result<()> {
        match self.send_request(Request::Remove { key }).await? {
            Some(Response::Remove) => Ok(()),
            Some(Response::Err(msg)) => Err(KvsError::StringError(msg)),
            Some(_) => Err(KvsError::StringError("Invalid response".to_owned())),
            None => Err(KvsError::StringError("No response received".to_owned())),
        }
    }

    /// Scan all key in the server
    pub async fn scan(&mut self, key: String) -> Result<Vec<String>> {
        match self.send_request(Request::Scan{ key }).await? {
            Some(Response::Scan(value)) => Ok(value),
            Some(Response::Err(msg)) => Err(KvsError::StringError(msg)),
            Some(_) => Err(KvsError::StringError("Invalid response".to_owned())),
            None => Err(KvsError::StringError("No response received".to_owned())),
        }
    }

    async fn send_request(&mut self, req: Request) -> Result<Option<Response>> {
        let jreq = serde_json::to_vec(&req)?;
        let mut header = [0; 4];
        BigEndian::write_u32(&mut header, jreq.len() as u32);
        self.write_half.write_all(&header).await?;
        self.write_half.write_all(&jreq).await?;

        self.read_half.read_exact(&mut header).await?;
        let len = BigEndian::read_u32(&header);
        let mut payload = vec![0u8; len as usize];
        self.read_half.read_exact(&mut payload).await?;
        let resp: Response = serde_json::from_slice(&payload).unwrap();
        Ok(Some(resp))
    }
}
