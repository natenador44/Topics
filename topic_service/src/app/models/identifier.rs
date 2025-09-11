use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

pub type IdentifierId = Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Identifier {
    pub id: Uuid,
    pub topic_id: Uuid,
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
