use std::sync::Arc;

use crate::auth::roles::Roles;

#[derive(Debug, Clone)]
pub struct AuthedUser<R> {
    pub id: Arc<str>,
    pub email: Option<Arc<str>>,
    pub roles: R,
}

impl<R> AuthedUser<R>
where
    R: Roles,
{
    pub fn has_roles(&self, role: R) -> bool {
        self.roles.contains(role)
    }
}
