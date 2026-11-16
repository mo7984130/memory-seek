use std::ops::Deref;

use crate::UserId;

pub struct AdminId(UserId);

impl AdminId {
    pub fn new(user_id: UserId) -> Option<Self> {
        if user_id.0 != 1 {
            None
        } else {
            Some(Self(user_id))
        }
    }

    pub fn inner(&self) -> &UserId {
        &self.0
    }
}

impl Deref for AdminId {
    type Target = UserId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
