use common_core::{
    ext::result::ToOkExt,
    time::{self, DateTime},
};
use type_auth::LoginResponse;
use type_core::UserId;

use crate::storage::{Storage, StorageError};

const USER_ID_KEY: &str = "MEMORY_SEEK:USER_ID";
const ACCESS_TOKEN_PAIR_KEY: &str = "MEMORY_SEEK:ACCESS_TOKEN_PAIR";
const REFRESH_TOKEN_PAIR_KEY: &str = "MEMORY_SEEK:REFRESH_TOKEN_PAIR";
pub struct AuthStorage<S: Storage> {
    storage: S,
}

type Result<T> = std::result::Result<T, StorageError>;
impl<S: Storage> AuthStorage<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub fn user_id(&self) -> Result<Option<UserId>> {
        self.storage.get::<UserId>(USER_ID_KEY)
    }

    pub fn access_token(&self) -> Result<Option<String>> {
        let v = self
            .storage
            .get::<(String, DateTime)>(ACCESS_TOKEN_PAIR_KEY)?;
        match v {
            None => None,
            Some((access_token, access_token_expire_at)) => {
                if time::now() > access_token_expire_at {
                    None
                } else {
                    Some(access_token)
                }
            }
        }
        .to_ok()
    }

    pub fn set_access_token(
        &self,
        access_token: String,
        access_token_expire_at: DateTime,
    ) -> Result<()> {
        self.storage.set(
            ACCESS_TOKEN_PAIR_KEY,
            (access_token, access_token_expire_at),
        )
    }

    pub fn remove_access_token(&self) -> Result<()> {
        self.storage.remove(ACCESS_TOKEN_PAIR_KEY)
    }

    pub fn refresh_token(&self) -> Result<Option<String>> {
        let v = self
            .storage
            .get::<(String, DateTime)>(REFRESH_TOKEN_PAIR_KEY)?;
        match v {
            None => None,
            Some((access_token, access_token_expire_at)) => {
                if time::now() > access_token_expire_at {
                    None
                } else {
                    Some(access_token)
                }
            }
        }
        .to_ok()
    }

    pub fn login(&self, resp: &LoginResponse) -> Result<()> {
        self.storage.set(USER_ID_KEY, resp.user.id)?;
        self.storage.set(
            ACCESS_TOKEN_PAIR_KEY,
            (resp.access_token.clone(), resp.access_token_expire_at),
        )?;
        self.storage.set(
            REFRESH_TOKEN_PAIR_KEY,
            (resp.refresh_token.clone(), resp.refresh_token_expire_at),
        )?;
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.storage
            .remove_many([USER_ID_KEY, ACCESS_TOKEN_PAIR_KEY, REFRESH_TOKEN_PAIR_KEY])
    }

    pub fn check_login(&self) -> Result<bool> {
        Ok(self.user_id()?.is_some() && self.refresh_token()?.is_some())
    }
}
