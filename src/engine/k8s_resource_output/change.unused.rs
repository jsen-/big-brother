use std::marker::PhantomData;

use destream::{de, EncodeMap, FromStream};

use crate::k8s_resource::K8sResource;

#[derive(Debug)]
pub enum Change {
    Added(K8sResource),
    Modified(K8sResource),
    Deleted(K8sResource),
    Unknown,
    // Bookmark(Bookmark),
}

#[async_trait::async_trait]
impl de::FromStream for Change {
    type Context = ();
    async fn from_stream<D: destream::Decoder>(_context: Self::Context, decoder: &mut D) -> Result<Self, D::Error> {
        decoder.decode_map(ChangeVisitor).await
    }
}

pub struct ChangeVisitor;

#[async_trait::async_trait]
impl destream::Visitor for ChangeVisitor {
    type Value = Change;
    fn expecting() -> &'static str {
        "Change"
    }

    async fn visit_map<A: destream::MapAccess>(self, mut map: A) -> Result<Self::Value, A::Error> {
        use destream::de::Error;
        enum ChangeType {
            Added,
            Modified,
            Deleted,
            Unknown,
        }
        let mut ty = None;
        let mut object = None;

        while let Some(key) = map.next_key::<String>(()).await? {
            if key == "type" {
                if ty.is_some() {
                    return Err(A::Error::custom("duplicate key \"type\""));
                } else {
                    let ty_val = map.next_value::<String>(()).await?;
                    let ty_enum = if &ty_val == "ADDED" {
                        ChangeType::Added
                    } else if &ty_val == "MODIFIED" {
                        ChangeType::Modified
                    } else if &ty_val == "DELETED" {
                        ChangeType::Deleted
                    } else {
                        ChangeType::Unknown
                    };
                    ty = Some(ty_enum);
                }
            } else if key == "object" {
                if object.is_some() {
                    return Err(A::Error::custom("duplicate key \"object\""));
                } else {
                    object = Some(map.next_value(()).await?);
                }
            } else {
                return Err(A::Error::custom(format!("unexpected key \"{}\"", key)));
            }
        }
        match (ty, object) {
            (Some(ChangeType::Added), Some(object)) => Ok(Change::Added(object)),
            (Some(ChangeType::Modified), Some(object)) => Ok(Change::Modified(object)),
            (Some(ChangeType::Deleted), Some(object)) => Ok(Change::Deleted(object)),
            (Some(ChangeType::Unknown), _) => Ok(Change::Unknown),
            (None, Some(_)) => Err(A::Error::custom("key \"type\" is missing")),
            (Some(_), None) => Err(A::Error::custom("key \"object\" is missing")),
            (None, None) => Err(A::Error::custom("keys \"type\" and \"object\" are missing")),
        }
    }
}
