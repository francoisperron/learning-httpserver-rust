use derive_more::{Display, Error, From};

use crate::id::Id;
use crate::username::{Username, UsernameEmptyError};

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Debug, Display, Error, From, PartialEq)]
pub enum UserError {
    UsernameEmptyError { source: UsernameEmptyError }
}


#[cfg(test)]
mod tests {
    use crate::user::{User, UserError};
    use crate::username::UsernameEmptyError;

    #[test]
    fn rejects_user_with_invalid_username() {
        let result = User::new("  ");

        assert_eq!(result, Err(UserError::from(UsernameEmptyError)));
    }
}
