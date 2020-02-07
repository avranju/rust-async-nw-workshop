use std::io::{Error, ErrorKind};
use std::str;

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum Event {
    Join(String),
    JoinResponse(u64),
    Leave(u64),
    Message(u64, String),
}

impl Event {
    fn discriminant(&self) -> u8 {
        match self {
            Event::Join(_) => 0,
            Event::JoinResponse(_) => 1,
            Event::Leave(_) => 2,
            Event::Message(_, _) => 3,
        }
    }
}

pub struct EventCodec;

impl Encoder for EventCodec {
    type Item = Event;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u8(item.discriminant());

        match &item {
            Event::Join(user) => {
                let buf = user.as_bytes();
                dst.put_u64(buf.len() as u64);
                dst.put_slice(buf);
            }

            Event::JoinResponse(id) | Event::Leave(id) => {
                dst.put_u64(*id);
            }

            Event::Message(id, msg) => {
                dst.put_u64(*id);
                let buf = msg.as_bytes();
                dst.put_u64(buf.len() as u64);
                dst.put_slice(buf);
            }
        }

        Ok(())
    }
}

impl Decoder for EventCodec {
    type Item = Event;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() == 0 {
            src.reserve(1);
            return Ok(None);
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

                let user = str::from_utf8(&src[0..len])
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Bad bytes"))?;

                Event::Join(user.to_string())
            }
            1 => {
                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let id = src.get_u64();

                Event::JoinResponse(id)
            }
            2 => {
                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let id = src.get_u64();

                Event::Leave(id)
            }
            3 => {
                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let id = src.get_u64();

                if src.len() < 8 {
                    src.reserve(8);
                    return Ok(None);
                }

                let len = src.get_u64() as usize;
                if src.len() < len {
                    src.reserve(len);
                    return Ok(None);
                }

                let msg = str::from_utf8(&src[0..len])
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "Bad bytes"))?;

                Event::Message(id, msg.to_string())
            }
            _ => return Err(Error::new(ErrorKind::InvalidData, "Bad bytes")),
        };

        Ok(Some(evt))
    }
}
