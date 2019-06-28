
use crate::common::{Request, Response};
use crate::{KvsEngine, Result};
use std::net::SocketAddr;
use tokio::prelude::*;

use futures::executor::{self, ThreadPool};
use romio::{TcpListener, TcpStream};
use futures::task::SpawnExt;
use futures::StreamExt;
use futures::io::{AsyncReadExt, AsyncWriteExt};
use byteorder::{ByteOrder, BigEndian};
use futures::compat::Future01CompatExt;

/// The server of a key value store.
pub struct KvsServer<E: KvsEngine> {
    engine: E,
}

impl<E: KvsEngine> KvsServer<E> {
    /// Create a `KvsServer` with a given storage engine.
    pub fn new(engine: E) -> Self {
        KvsServer { engine }
    }

    /// Run the server listening on the given address
    pub fn run(self, addr: SocketAddr) -> Result<()> {
        executor::block_on(async {
            let mut threadpool = ThreadPool::new()?;
            let mut listener = TcpListener::bind(&addr)?;
            let mut incoming = listener.incoming();

            while let Some(stream) = incoming.next().await {
                let stream = stream?;
                let addr = stream.peer_addr()?;
                let engine = self.engine.clone();

                threadpool.spawn(async move {
                    debug!("Accepting stream from: {}", addr);

                    serve(engine, stream).await.map_err(|e| {
                        error!("Error on serving client: {}", e);
                        //KvsError::StringError(e.to_string())
                    }).ok();

                    debug!("Closing stream from: {}", addr);
                }).ok();
            }
            Ok(())
        })
    }
}

async fn serve<E: KvsEngine>(engine: E, tcp: TcpStream) -> Result<()> {
    let (mut read_half, mut write_half) = tcp.split();

    let mut header = [0u8; 4];
    loop {
        read_half.read_exact(&mut header).await?;
        let len = BigEndian::read_u32(&header);
        debug!("header: {}", len);
        let mut payload = vec![0u8; len as usize];
        read_half.read_exact(&mut payload).await?;
        debug!("read {} {}", payload.len(), std::str::from_utf8(&payload).unwrap());
        //debug!("result {} {}", result.len(), std::str::from_utf8(&result).unwrap());
        let res = match serde_json::from_slice(&payload).unwrap() {
            Request::Get { key } => engine.get(key).map(Response::Get).compat().await,
            Request::Set { key, value } => {
                engine.set(key, value).map(|_| Response::Set).compat().await
            }
            Request::Remove { key } => {
                engine.remove(key).map(|_| Response::Remove).compat().await
            } 
            Request::Scan { key } => {
                engine.scan(key).map(Response::Scan)
            }
        };
        let resp = match res {
            Ok(r) => r,
            Err(e) => Response::Err(format!("{}", e)),
        };
        let jresp = serde_json::to_vec(&resp)?;
        BigEndian::write_u32(&mut header, jresp.len() as u32);
        write_half.write_all(&header).await?;
        write_half.write_all(&jresp).await?;
    }
}
