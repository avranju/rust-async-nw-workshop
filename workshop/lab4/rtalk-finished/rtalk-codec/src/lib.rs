use std::io::{Error, ErrorKind};

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

const MAGIC_COOKIE: u32 = 0xDEAD_BEEF;

#[derive(Debug)]
pub enum Event {
    RequestJoin(String),
    Joined(String),
    Leave(),
    Left(String),
    MessageSend(String),
    MessageReceived(String, String),
}

impl Event {
    fn discriminant(&self) -> u8 {
        match self {
            Event::RequestJoin(_) => 0,
            Event::Joined(_) => 1,
            Event::Leave() => 2,
            Event::Left(_) => 3,
            Event::MessageSend(_) => 4,
            Event::MessageReceived(_, _) => 5,
        }
    }
}

pub struct EventCodec;

fn put_string(dst: &mut BytesMut, string: &str) {
    let buf = string.as_bytes();
    dst.put_u64(buf.len() as u64);
    dst.put_slice(buf);
}

impl Encoder for EventCodec {
    type Item = Event;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u32(MAGIC_COOKIE);
        dst.put_u8(item.discriminant());

        match &item {
            Event::RequestJoin(user) | Event::Joined(user) | Event::Left(user) => {
                put_string(dst, user);
            }

            Event::MessageSend(msg) => {
                put_string(dst, msg);
            }

            Event::Leave() => {}

            Event::MessageReceived(who, msg) => {
                put_string(dst, who);
                put_string(dst, msg);
            }
        }

        Ok(())
    }
}

impl Decoder for EventCodec {
    type Item = Event;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            src.reserve(5);
            return Ok(None);
        }

        let cookie = src.get_u32();
        if cookie != MAGIC_COOKIE {
            return Err(Error::from(ErrorKind::InvalidInput));
        }

        let discriminant = src.get_u8();
        let evt = match discriminant {
            0 => {
                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let len = src.get_u64() as usize;
                if src.len() < len {
                    src.reserve(len);
                    return Ok(None);
                }

                let user = String::from_utf8(src[0..len].to_vec())
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Bad bytes"))?;
                src.advance(len);

                Event::RequestJoin(user)
            }
            1 => {
                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let len = src.get_u64() as usize;
                if src.len() < len {
                    src.reserve(len);
                    return Ok(None);
                }

                let user = String::from_utf8(src[0..len].to_vec())
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Bad bytes"))?;
                src.advance(len);

                Event::Joined(user)
            }
            2 => Event::Leave(),
            3 => {
                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let len = src.get_u64() as usize;
                if src.len() < len {
                    src.reserve(len);
                    return Ok(None);
                }

                let user = String::from_utf8(src[0..len].to_vec())
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Bad bytes"))?;
                src.advance(len);

                Event::Left(user)
            }
            4 => {
                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let len = src.get_u64() as usize;
                if src.len() < len {
                    src.reserve(len);
                    return Ok(None);
                }

                let msg = String::from_utf8(src[0..len].to_vec())
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Bad bytes"))?;
                src.advance(len);

                Event::MessageSend(msg)
            }
            5 => {
                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let len = src.get_u64() as usize;
                if src.len() < len {
                    src.reserve(len);
                    return Ok(None);
                }

                let user = String::from_utf8(src[0..len].to_vec())
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Bad bytes"))?;
                src.advance(len);

                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let len = src.get_u64() as usize;
                if src.len() < len {
                    src.reserve(len);
                    return Ok(None);
                }

                let msg = String::from_utf8(src[0..len].to_vec())
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Bad bytes"))?;
                src.advance(len);

                Event::MessageReceived(user, msg)
            }
            _ => return Err(Error::new(ErrorKind::InvalidData, "Bad bytes")),
        };

        Ok(Some(evt))
    }
}
