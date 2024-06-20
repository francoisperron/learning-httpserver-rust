pub type Id = u64;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Id,
    pub username: String,
}