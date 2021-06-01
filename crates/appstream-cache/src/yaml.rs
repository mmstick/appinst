use futures_codec::{BytesMut, Decoder};
use memchr::memchr;

#[derive(Default)]
pub struct YamlSplitter;

impl Decoder for YamlSplitter {
    type Item = BytesMut;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.starts_with(b"---\n") {
            let _ = src.split_to(4);
        }

        let mut p = 0;
        while let Some(pos) = memchr(b'\n', &src[p..]) {
            p += pos + 1;
            if src[p..].starts_with(b"---\n") {
                let bytes = src.split_to(p);
                let _ = src.split_to(4);
                return Ok(Some(bytes));
            }
        };

        return Ok(None);
    }

    fn decode_eof(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut bytes = src.split();
        return Ok(
            if bytes.is_empty() {
                None
            } else if bytes.ends_with(b"\n") {
                Some(bytes.split_to(bytes.len()-1))
            } else {
                Some(bytes)
            }
        );
    }
}
