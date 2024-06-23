use derive_more::{Display, Into};

#[derive(Clone, Debug, Display, Into, PartialEq)]
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
mod username_tests {
    use crate::username::{Username, UsernameError};

    #[test]
    fn rejects_empty_username() {
        let result = Username::new("  ");

        assert_eq!(result, Err(UsernameError("Username cannot be empty".to_string())));
    }

    #[test]
    fn into_string() {
        let value: String = Username::new("mario").unwrap().into();
        assert_eq!(value, "mario");
    }

    #[test]
    fn formats_to_string() {
        assert_eq!(Username::new("mario").unwrap().to_string(), "mario");
    }
}