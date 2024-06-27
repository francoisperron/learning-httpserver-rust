use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use crate::users::id::Id;
use crate::users::user::User;


pub trait UsersRepo: Clone + Send + Sync + 'static {
    fn save_user(&self, user: &User) -> impl Future<Output=()> + Send;

    fn delete_user(&self, id: Id) -> impl Future<Output=bool> + Send;

    fn get_user(&self, id: Id) -> impl Future<Output=Option<User>> + Send;

    fn get_users(&self) -> impl Future<Output=Vec<User>> + Send;
}

#[derive(Debug, Clone, Default)]
pub struct UsersRepoInMemory {
    map: Arc<Mutex<HashMap<Id, User>>>,
}


impl UsersRepo for UsersRepoInMemory {
    fn save_user(&self, user: &User) -> impl Future<Output=()> + Send {
        self.map.lock().unwrap().insert(user.id.clone(), user.clone());
        async move { }
    }

    fn delete_user(&self, id: Id) -> impl Future<Output=bool> + Send {
        let deleted = self.map.lock().unwrap().remove(&id).is_some();
        async move { deleted }
    }

    fn get_user(&self, id: Id) -> impl Future<Output=Option<User>> + Send {
        let user = self.map.lock().unwrap().get(&id).cloned();
        async move { user }
    }

    fn get_users(&self) -> impl Future<Output=Vec<User>> + Send {
        let users = self.map.lock().unwrap().values().cloned().collect();
        async move { users }
    }
}