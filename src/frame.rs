use std::io::Cursor;
use std::num::TryFromIntError;
use std::string::FromUtf8Error;
use bytes::{Buf, Bytes};

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>)
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    
    Other(crate::Error)
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<(), Error> {
        match get_u8(src)? {
            b'+' => {
                get_line(src)?;
                Ok(())
            }
            b'-' => {
                get_line(src)?;
                Ok(())
            }
            b':' => {
                get_decimal(src)?;
                Ok(())
            }
            b'$' => {
                if b'-' == peek_u8(src)? {
                    skip(src, 4)
                } else { 
                    let len = get_decimal(src)?;
                    skip(src, (len + 4) as usize)
                }
            }
            b'*' => {
                let len = get_decimal(src)?;
                
                for i in 0..len {
                    Frame::check(src)?;
                }
                
                Ok(())
            }
            actual => Err(format!("protocol error; invalid frame type byte `{}`", actual).into())
        }
    }
    
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Frame, Error> {
        match get_u8(src)? {
            b'+' => {
                let line = get_line(src)?.to_vec();
                let str = String::from_utf8(line)?;
                Ok(Frame::Simple(str))
            }
            b'-' => {
                let line = get_line(src)?.to_vec();
                let str = String::from_utf8(line)?;
                Ok(Frame::Error(str))
            }
            b':' => {
                let num = get_decimal(src)?;
                Ok(Frame::Integer(num))
            }
            b'$' => {
                if b'-' == peek_u8(src)? {
                    let line = get_line(src)?;
                    if line != b"-1" {
                        return Err("protocol error; invalid frame format".into())
                    }
                    Ok(Frame::Null)
                } else {
                    let len: usize = get_decimal(src)?.try_into()?;
                    let n = len + 2;
                    
                    if src.remaining() < n as usize {
                        return Err(Error::Incomplete);
                    }
                    
                    let data = Bytes::copy_from_slice(&src.chunk()[..len]);
                    
                    Ok(Frame::Bulk(data))
                }
            }
            b'*' => {
                let len: usize = get_decimal(src)?.try_into()?;
                let mut out = Vec::with_capacity(len);

                for i in 0..len {
                    out.push(Frame::parse(src)?);
                }

                Ok(Frame::Array(out))
            }
            _ => unimplemented!(),
        }
    }
}

fn get_line<'a>(src: & mut Cursor<&'a [u8]>) -> Result<&'a [u8], Error> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;
    
    for i in start .. end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i+1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Ok(&src.get_ref()[start..i]);
        }
    }
    
    Err(Error::Incomplete)
}

fn get_decimal(src: & mut Cursor<&[u8]>) -> Result<u64, Error> {
    use atoi::atoi;
    
    let line = get_line(src)?;
    
    atoi::<u64>(line).ok_or_else(|| "protocol error; invalid frame format".into())
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }
    
    Ok(src.get_u8())
}

fn peek_u8(src: &mut Cursor<&[u8]>) -> Result<u8, Error> {
    if !src.has_remaining() {
        return Err(Error::Incomplete);
    }

    Ok(src.chunk()[0])
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<(), Error> {
    if !src.remaining() < n {
        return Err(Error::Incomplete);
    }

    Ok(src.advance(n))
}


impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_src: FromUtf8Error) -> Error {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for Error {
    fn from(_src: TryFromIntError) -> Error {
        "protocol error; invalid frame format".into()
    }
}
