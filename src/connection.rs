use std::io;
use std::io::Read;

use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,

    buffer: BytesMut,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),

            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }

    pub async fn read(&mut self) -> crate::Result<Option<&[u8]>> {
        loop {
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                return Ok(Some(&self.buffer));
            }
        }
    }

    pub async fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.stream.write(bytes).await?;
        self.stream.flush().await
    }
}

