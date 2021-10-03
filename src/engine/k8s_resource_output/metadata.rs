use destream::{de, EncodeMap};

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub name: String,
    pub namespace: Option<String>,
    pub resource_version: String,
}

impl<'en> destream::ToStream<'en> for Metadata {
    fn to_stream<E: destream::Encoder<'en>>(&'en self, encoder: E) -> Result<E::Ok, E::Error> {
        let mut map = encoder.encode_map(Some(3))?;
        map.encode_entry("name", &self.name)?;
        map.encode_entry("namespace", &self.namespace)?;
        map.encode_entry("resourceVersion", &self.resource_version)?;
        map.end()
    }
}

#[async_trait::async_trait]
impl de::FromStream for Metadata {
    type Context = ();
    async fn from_stream<D: destream::Decoder>(_context: Self::Context, decoder: &mut D) -> Result<Self, D::Error> {
        decoder.decode_map(MetadataVisitor).await
    }
}

pub struct MetadataVisitor;

#[async_trait::async_trait]
impl destream::Visitor for MetadataVisitor {
    type Value = Metadata;
    fn expecting() -> &'static str {
        "Metadata"
    }

    async fn visit_map<A: destream::MapAccess>(self, mut map: A) -> Result<Self::Value, A::Error> {
        use destream::de::Error;

        let mut name = None;
        let mut namespace = None;
        let mut resource_version = None;

        while let Some(key) = map.next_key::<String>(()).await? {
            if key == "name" {
                if name.is_some() {
                    return Err(A::Error::custom("duplicate key \"name\""));
                } else {
                    name = Some(map.next_value::<String>(()).await?);
                }
            } else if key == "namespace" {
                if namespace.is_some() {
                    return Err(A::Error::custom("duplicate key \"namespace\""));
                } else {
                    namespace = Some(map.next_value::<String>(()).await?);
                }
            } else if key == "resourceVersion" {
                if resource_version.is_some() {
                    return Err(A::Error::custom("duplicate key \"resourceVersion\""));
                } else {
                    resource_version = Some(map.next_value::<String>(()).await?);
                }
            } else {
                // ignore unknown keys
                map.next_value::<destream_json::Value>(()).await?;
            }
        }

        match (name, namespace, resource_version) {
            (Some(name), namespace, Some(resource_version)) => Ok(Metadata {
                name,
                namespace,
                resource_version,
            }),
            _ => Err(A::Error::custom("some key is missing")),
        }
    }
}
