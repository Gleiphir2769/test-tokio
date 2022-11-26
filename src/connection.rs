use std::fs::read;
use std::io;
use std::io::{Cursor, Read};

use bytes::{Buf, Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;



#[derive(Debug)]
pub enum Error {
    /// Not enough data is available to parse a message
    Incomplete,

    /// Invalid message encoding
    Other(crate::Error),
}

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

    pub async fn read(&mut self) -> crate::Result<Option<Vec<u8>>> {
        loop {
            if self.buffer.len() > 0 {
                // 如何在返回前清空buffer，即如何完整的将buffer的值
                // 读出返回出去
                
                let mut buf = Cursor::new(&self.buffer[..]);
                let mut res = Vec::new();
                if let Ok(V) = get_line(&mut buf) {
                    res = V.to_vec();
                }
                
                let len = buf.position() as usize;
                self.buffer.advance(len);
                
                return Ok(Some(res))
            }
            
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this to be a clean
                // shutdown, there should be no data in the read buffer. If
                // there is, this means that the peer closed the socket while
                // sending a frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    pub async fn write(&mut self, bytes: &Vec<u8>) -> io::Result<()> {
        self.stream.write(bytes).await?;
        self.stream.flush().await
    }
}