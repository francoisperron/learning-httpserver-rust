use derive_more::Display;

use crate::id::Id;
use crate::username::{Username, UsernameError};

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: Id,
    pub username: Username,
}

impl User {
    pub fn new(raw_username: &str) -> Result<User, UserError> {
        let id = Id::new();
        let username = Username::new(raw_username)?;
        Ok(User { id, username })
    }
}

#[derive(Debug, PartialEq, Display)]
pub struct UserError(String);

impl From<UsernameError> for UserError {
    fn from(error: UsernameError) -> Self {
        UserError(error.to_string())
    }
}


#[cfg(test)]
mod user_tests {
    use crate::user::{User, UserError};

    #[test]
    fn rejects_user_with_invalid_username() {
        let result = User::new("  ");

        assert_eq!(result, Err(UserError("Username cannot be empty".to_string())));
    }
}
