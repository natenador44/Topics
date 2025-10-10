use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TopicId {
    #[schema(value_type = String)]
    inner: ObjectId,
}

impl Deref for TopicId {
    type Target = ObjectId;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for TopicId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
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
