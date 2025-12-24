use std::{
    convert::Infallible,
    fmt::Display,
    ops::{BitOr, BitOrAssign},
    str::FromStr,
};

use routing::Roles;
use tracing::{instrument, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct TopicRoles(u8);
impl TopicRoles {
    const MAX: u8 = 4;
    pub const NONE: TopicRoles = TopicRoles(0);
    pub const TOPIC_READ: TopicRoles = TopicRoles(1);
    pub const TOPIC_WRITE: TopicRoles = TopicRoles(2);
    pub const TOPIC_ADMIN: TopicRoles = TopicRoles(Self::MAX);

    fn iter(&self) -> RolesIter {
        RolesIter::new(*self)
    }
}

impl Roles for TopicRoles {
    fn contains(&self, other: TopicRoles) -> bool {
        self.0 & other.0 != TopicRoles::NONE.0
    }
    fn none() -> Self {
        Self::NONE
    }

    fn is_none(&self) -> bool {
        self.0 == Self::NONE.0
    }

    fn add(&mut self, other: Self) {
        *self |= other;
    }
}

impl Default for TopicRoles {
    fn default() -> Self {
        Self::NONE
    }
}

/// An iterator over the individual roles stored in the `Roles` bitflag.
/// ```rust
/// let roles = Roles::TOPIC_WRITE | Roles::TOPIC_READ;
/// let mut itr = roles.iter();
///
/// assert_eq!(Some(Roles::TOPIC_READ), itr.next());
/// assert_eq!(Some(Roles::TOPIC_WRITE), itr.next());
/// assert_eq!(None, itr.next());
/// ```
struct RolesIter {
    roles: TopicRoles,
    idx: u8,
}

impl RolesIter {
    fn new(roles: TopicRoles) -> Self {
        Self { roles, idx: 0 }
    }
}

impl Iterator for RolesIter {
    type Item = TopicRoles;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let roles = self.roles.0 >> self.idx;

            if roles == 0 {
                return None;
            }

            let role = roles % 2;

            if role == 1 {
                let result = Some(TopicRoles(2u8.pow(self.idx as u32)));
                self.idx += 1;
                return result;
            } else {
                self.idx += 1;
            }
        }
    }
}

impl Display for TopicRoles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == TopicRoles::NONE {
            write!(f, "[]")
        } else {
            write!(f, "[")?;
            for role in self.iter() {
                match role.0 {
                    1 => write!(f, "TOPIC_READ,")?,
                    2 => write!(f, "TOPIC_WRITE,")?,
                    4 => write!(f, "ADMIN,")?,
                    _ => unreachable!("unless new topic added"),
                }
            }
            write!(f, "]")
        }
    }
}

impl BitOr for TopicRoles {
    type Output = TopicRoles;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for TopicRoles {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 | rhs.0);
    }
}

impl FromStr for TopicRoles {
    type Err = Infallible; // unknown roles are ignored

    #[instrument]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim_matches('"') {
            "TOPIC_ADMIN" => Ok(TopicRoles::TOPIC_ADMIN),
            "TOPIC_READ" => Ok(TopicRoles::TOPIC_READ),
            "TOPIC_WRITE" => Ok(TopicRoles::TOPIC_WRITE),
            other => {
                warn!("Unknown role: {other}. Ignoring");
                Ok(TopicRoles::NONE)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use routing::Roles;

    use super::TopicRoles;

    #[test]
    fn roles_contains() {
        let roles = TopicRoles::TOPIC_READ;

        assert_eq!(TopicRoles::TOPIC_READ, roles);

        let roles = TopicRoles::TOPIC_READ | TopicRoles::TOPIC_WRITE;

        assert!(roles.contains(TopicRoles::TOPIC_READ));
        assert!(roles.contains(TopicRoles::TOPIC_WRITE));
        assert!(roles.contains(TopicRoles::TOPIC_WRITE | TopicRoles::TOPIC_READ));
        assert!(
            !roles.contains(TopicRoles::TOPIC_ADMIN),
            "!{:b}.contains({:b})",
            roles.0,
            TopicRoles::TOPIC_ADMIN.0
        );
    }

    #[test]
    fn roles_iter() {
        let roles = TopicRoles::TOPIC_READ | TopicRoles::TOPIC_WRITE;
        let mut iter = roles.iter();

        assert_eq!(
            Some(TopicRoles::TOPIC_READ),
            iter.next(),
            "expecting TOPIC_READ"
        );
        assert_eq!(
            Some(TopicRoles::TOPIC_WRITE),
            iter.next(),
            "expecting TOPIC_WRITE"
        );
        assert_eq!(None, iter.next(), "expecting None");
    }

    #[test]
    fn roles_display() {
        let roles = TopicRoles::TOPIC_READ;

        assert_eq!("[TOPIC_READ,]", &roles.to_string());

        let roles = TopicRoles::TOPIC_READ | TopicRoles::TOPIC_ADMIN;

        assert_eq!("[TOPIC_READ,ADMIN,]", &roles.to_string());

        let roles = TopicRoles::NONE;
        assert_eq!("[]", &roles.to_string());
    }
}
