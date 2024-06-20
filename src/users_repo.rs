use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::user::{Id, User};

pub trait UsersRepo: Send + Sync {
    fn save_user(&self, user: &User);

    fn delete_user(&self, id: Id) -> Option<User>;

    fn get_user(&self, id: Id) -> Option<User>;

    fn get_users(&self) -> Vec<User>; // @todo replace by iter
}

#[derive(Debug, Clone, Default)]
pub struct UsersRepoInMemory {
    map: Arc<Mutex<HashMap<Id, User>>>,
}

impl UsersRepo for UsersRepoInMemory {
    fn save_user(&self, user: &User) {
        self.map.lock().unwrap().insert(user.id, user.clone());
    }

    fn delete_user(&self, id: Id) -> Option<User> {
        self.map.lock().unwrap().remove(&id)
    }

    fn get_user(&self, id: Id) -> Option<User> {
        self.map.lock().unwrap().get(&id).cloned()
    }

    fn get_users(&self) -> Vec<User> {
        self.map.lock().unwrap().values().cloned().collect()
    }
}