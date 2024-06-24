use derive_more::{Display, Error, Into};

#[derive(Clone, Debug, Into, PartialEq)]
pub struct Username(String);

impl Username {
    pub fn new(raw_username: &str) -> Result<Username, UsernameEmptyError> {
        if raw_username.trim().is_empty() {
            return Err(UsernameEmptyError);
        }

        Ok(Username(raw_username.to_string()))
    }
}

#[derive(Debug, Display, Error, PartialEq)]
pub struct UsernameEmptyError;


#[cfg(test)]
mod tests {
    use crate::users::username::{Username, UsernameEmptyError};

    #[test]
    fn rejects_empty_username() {
        let result = Username::new("  ");

        assert_eq!(result, Err(UsernameEmptyError));
    }

    #[test]
    fn converts_into_string() {
        let value: String = Username::new("mario").unwrap().into();
        assert_eq!(value, "mario");
    }
}