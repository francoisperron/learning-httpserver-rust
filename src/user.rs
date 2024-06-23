use std::time::{SystemTime, UNIX_EPOCH};
use derive_more::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: Id,
    pub username: Username,
}

impl User {
    pub fn new(raw_username: &str) -> Result<User, UserError> {
        let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_micros() as Id;
        let username = Username::new(raw_username)?;
        Ok(User { id, username })
    }
}

#[derive(Debug, PartialEq, Display)]
pub struct  UserError(String);

impl From<UsernameError> for UserError {
    fn from(error: UsernameError) -> Self {
        UserError(error.to_string())
    }
}

pub type Id = u64;

#[derive(Debug, Clone, PartialEq, Display)]
pub struct Username(String);

impl Username {
    pub fn new(raw_username: &str) -> Result<Username, UsernameError> {
        if raw_username.trim().is_empty() {
            return Err(UsernameError("Username cannot be empty".to_string()));
        }

        Ok(Username(raw_username.to_string()))
    }
}

#[derive(Debug, PartialEq, Display)]
pub struct UsernameError(String);


#[cfg(test)]
mod user_tests {
    use crate::user::{User, UserError};

    #[test]
    fn rejects_user_with_invalid_username() {
        let result = User::new("  ");

        assert_eq!(result, Err(UserError("Username cannot be empty".to_string())));
    }
}

#[cfg(test)]
mod username_tests {
    use crate::user::{Username, UsernameError};

    #[test]
    fn rejects_empty_username() {
        let result = Username::new("  ");

        assert_eq!(result, Err(UsernameError("Username cannot be empty".to_string())));
    }

    #[test]
    fn formats_to_string() {
        assert_eq!(Username::new("mario").unwrap().to_string(), "mario");
    }
}
