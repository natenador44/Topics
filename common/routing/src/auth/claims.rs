use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth::{roles::Roles, user::AuthedUser};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    email: Option<String>,
    // apparently this can be a string or an array
    #[serde(default)]
    aud: Value,
    /// expiration
    exp: usize,
    /// issued?
    iss: String,
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

impl Claims {
    pub fn into_authed_user<R>(self, roles_path: &str) -> AuthedUser<R>
    where
        R: Roles,
        R::Err: Debug,
    {
        let parts = roles_path.split('.');

        let mut current = Value::Object(self.extra);

        for part in parts {
            current = match current.get(part) {
                Some(val) => val.clone(),
                None => {
                    return AuthedUser {
                        id: self.sub.into(),
                        // email: self.email.map(|e| e.into()),
                        roles: R::none(),
                    };
                }
            }
        }

        let roles = match current {
            Value::Array(roles) => roles.into_iter().fold(R::none(), |mut r, next| {
                r.add(
                    next.to_string()
                        .parse()
                        .expect("roles flag parse is infallible"),
                );
                r
            }),
            _ => R::none(),
        };

        AuthedUser {
            id: self.sub.into(),
            // email: self.email.map(|e| e.into()),
            roles,
        }
    }
}
