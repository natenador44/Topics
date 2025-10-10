use mongodb::bson::Bson;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use utoipa::ToSchema;
use utoipa::openapi::Object;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Clone)]
#[repr(transparent)]
#[schema(value_type = String)]
pub struct TopicId(#[serde(serialize_with = "obj_id_serialize")] ObjectId);

fn obj_id_serialize<S>(id: &ObjectId, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    id.to_hex().serialize(ser)
}

impl TopicId {
    pub fn new(id: ObjectId) -> Self {
        Self(id)
    }
}

impl Deref for TopicId {
    type Target = ObjectId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<TopicId> for Bson {
    fn from(value: TopicId) -> Self {
        value.0.into()
    }
}

impl Display for TopicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SetId {
    #[schema(value_type = String)]
    inner: ObjectId,
}

impl Deref for SetId {
    type Target = ObjectId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for SetId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
#[serde(transparent)]
pub struct EntityId {
    #[schema(value_type = String)]
    inner: ObjectId,
}

impl Deref for EntityId {
    type Target = ObjectId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
#[serde(transparent)]
pub struct IdentifierId {
    #[schema(value_type = String)]
    inner: ObjectId,
}

impl Deref for IdentifierId {
    type Target = ObjectId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for IdentifierId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
