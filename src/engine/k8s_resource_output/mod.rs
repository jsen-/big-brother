mod metadata;
pub use metadata::Metadata;

use destream::{de, EncodeMap};

#[derive(Debug, Clone, PartialEq)]
pub struct K8sResourceOutput {
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
}

impl<'en> destream::ToStream<'en> for K8sResourceOutput {
    fn to_stream<E: destream::Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        let mut map = encoder.encode_map(Some(3))?;
        map.encode_entry("apiVersion", &self.api_version)?;
        map.encode_entry("kind", &self.kind)?;
        map.encode_entry("metadata", &self.metadata)?;
        map.end()
    }
}

#[async_trait::async_trait]
impl de::FromStream for K8sResourceOutput {
    type Context = ();
    async fn from_stream<D: destream::Decoder>(_context: Self::Context, decoder: &mut D) -> Result<Self, D::Error> {
        decoder.decode_map(K8sResourceVisitor).await
    }
}

pub struct K8sResourceVisitor;

#[async_trait::async_trait]
impl destream::Visitor for K8sResourceVisitor {
    type Value = K8sResourceOutput;
    fn expecting() -> &'static str {
        "K8sResource"
    }

    async fn visit_map<A: destream::MapAccess>(self, mut map: A) -> Result<Self::Value, A::Error> {
        use destream::de::Error;

        let mut api_version = None;
        let mut kind = None;
        let mut metadata = None;

        while let Some(key) = map.next_key::<String>(()).await? {
            if key == "apiVersion" {
                if api_version.is_some() {
                    return Err(A::Error::custom("duplicate key \"apiVersion\""));
                } else {
                    api_version = Some(map.next_value::<String>(()).await?);
                }
            } else if key == "kind" {
                if kind.is_some() {
                    return Err(A::Error::custom("duplicate key \"kind\""));
                } else {
                    kind = Some(map.next_value::<String>(()).await?);
                }
            } else if key == "metadata" {
                if metadata.is_some() {
                    return Err(A::Error::custom("duplicate key \"metadata\""));
                } else {
                    metadata = Some(map.next_value::<Metadata>(()).await?);
                }
            }
            // ignore unknown keys
        }

        match (api_version, kind, metadata) {
            (Some(api_version), Some(kind), Some(metadata)) => Ok(K8sResourceOutput {
                api_version,
                kind,
                metadata,
            }),
            _ => Err(A::Error::custom("some key is missing")),
        }
    }
}
