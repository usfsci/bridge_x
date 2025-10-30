use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{io, usize};
use tokio_util::codec::{Decoder, Encoder};

/// Maximum frame payload size (tune as needed).
pub const MAX_FRAME_SIZE: usize = 64 * 1024; // 64KB

/// codec: reads 2-byte start-of-frame, then 2-byte BE length, then payload.
pub struct TwoByteLenSkipReserved {
    max_frame: usize,
}

impl TwoByteLenSkipReserved {
    pub fn new(max_frame: usize) -> Self {
        Self { max_frame }
    }
}

impl Decoder for TwoByteLenSkipReserved {
    type Item = Bytes; // payload bytes (without len/reserved)
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Need at least 2 bytes for len
        if src.len() < 2 {
            return Ok(None);
        }

        // Peek initial 2 bytes until start of frame is seen
        // don't advance; read from the buffer directly
        if src[0] != 0xff || src[1] != 0xff {
            return Ok(None);
        }

        // Peek 2 bytes for length (big endian)
        let len = {
            // don't advance; read from the buffer directly
            let b2 = src[2];
            let b3 = src[3];
            u16::from_be_bytes([b2, b3]) as usize
        };

        // total frame = 2 (len field itself) + len
        let total_frame = 4usize + len;

        // Check max frame length
        if len > self.max_frame {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("frame length {} exceeds max {}", len, self.max_frame),
            ));
        }

        // Wait until whole frame arrives
        if src.len() < total_frame {
            return Ok(None);
        }

        // We have the full frame. Remove the length field first.
        // split_to(total_frame) will give us a BytesMut containing len field + payload (including reserved).
        let mut frame = src.split_to(total_frame);

        // Advance past length field (2 bytes)
        frame.advance(2);

        // Now frame contains: reserved(2) + payload(len-2)
        if frame.len() < 2 {
            // malformed: length claims there's reserved bytes but they're not present
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "missing reserved bytes"));
        }

        // discard reserved bytes
        frame.advance(2);

        // the remaining bytes are the payload
        let payload = frame.freeze(); // Bytes

        Ok(Some(payload))
    }
}

impl Encoder<Bytes> for TwoByteLenSkipReserved {
    type Error = io::Error;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let payload_len = item.len();

        if payload_len > self.max_frame {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("payload length {} exceeds max {}", payload_len, self.max_frame),
            ));
        }

        // length field includes the 2 reserved bytes
        let len_field = payload_len as u16;

        // Reserve space and append
        dst.reserve(2 + 2 + payload_len);
        dst.put_slice(&[0xffu8, 0xffu8]);
        dst.put_slice(&len_field.to_be_bytes());
        //dst.put_slice(&[0u8, 0u8]); // reserved bytes
        dst.put_slice(&item);

        Ok(())
    }
}
