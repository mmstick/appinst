use super::appstream::Dep11Package;
use crate::yaml::YamlSplitter;
use anyhow::Context;
use futures_codec::{BytesMut, Decoder};

#[derive(Debug, Default, Deserialize)]
pub struct Dep11Header {
    #[serde(rename = "File")]
    pub file: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Origin")]
    pub origin: String,
    #[serde(rename = "MediaBaseUrl")]
    pub media_base_url: Option<String>,
}

#[derive(Default)]
pub struct Dep11Splitter {
    pub header: Option<Dep11Header>
}

impl Decoder for Dep11Splitter {
    type Item = Dep11Package;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match YamlSplitter::default().decode(src) {
                Ok(Some(bytes)) => {
                    if self.header.is_none() {
                        self.header = Some(
                            serde_yaml::from_slice::<Dep11Header>(&bytes)
                                .context("failed to deserialize YAML")
                                .with_context(|| format!("{:?}", std::str::from_utf8(&bytes)))?
                        )
                    } else {
                        let package = serde_yaml::from_slice::<Dep11Package>(&bytes)
                            .context("failed to deserialize YAML")
                            .with_context(|| format!("{:?}", std::str::from_utf8(&bytes)))?;
                        return Ok(Some(package))
                    }

                }
                Ok(None) => return Ok(None),
                Err(why) => return Err(why)
            }
        }
    }

    fn decode_eof(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match YamlSplitter::default().decode_eof(src) {
            Ok(Some(bytes)) => {
                let package = serde_yaml::from_slice::<Dep11Package>(&bytes)
                    .context("failed to deserialize YAML")
                    .with_context(|| format!("{:?}", std::str::from_utf8(&bytes)))?;
                return Ok(Some(package))
            }
            Ok(None) => return Ok(None),
            Err(why) => return Err(why)
        }
    }
}
