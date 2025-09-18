use crate::app::models::TopicId;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
#[serde(transparent)]
#[schema(as = uuid::Uuid)]
pub struct IdentifierId(Uuid);
impl IdentifierId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Display for IdentifierId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Identifier {
    pub id: IdentifierId,
    pub topic_id: TopicId,
    // some sort of expression to 'test' data
}

/*
// check range on invoice
"legacyId": {
    "invoiceNumber": {
        {
            "type": "range",
            "start": 100000, // implies integer -- will not compare between differen types
            "end": 500000
        }
    }
},
// deferred core check
{
    "deferredCore": {
        "not": {
            "type": "equals",
            "value": null
        }
    }
}
// payment on account
{
    "for": {
        "basket": {},
    }
}
*/
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub enum Expression {
    And(Vec<Expression>),
    Or(Vec<Expression>),
}
