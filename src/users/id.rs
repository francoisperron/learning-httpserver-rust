use std::time::{SystemTime, UNIX_EPOCH};
use derive_more::{From, Into};

#[derive(Clone, Debug, Eq, From, Hash, Into, PartialEq)]
pub struct Id(u64);

impl Id {
    pub fn new() -> Id {
        Id(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_micros() as u64)
    }
}

impl Default for Id {
    fn default() -> Self {
        Id::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::users::id::Id;

    #[test]
    fn creates_with_random_id() {
        assert_ne!(Id::new(), Id::from(0));
    }
    
    #[test]
    fn creates_with_default() {
        assert_ne!(Id::default(), Id::from(0));
    }
    
    #[test]
    fn into_u64() {
        let value: u64 = Id::new().into();
        assert_ne!(value, 0);    
    }
}