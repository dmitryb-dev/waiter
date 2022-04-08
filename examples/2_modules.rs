extern crate config;
extern crate serde;
extern crate waiter_di;

use std::collections::HashMap;
use waiter_di::*;

// Simple demo of dependency inversion, constructors and modules

trait UserRepository {
    fn find(&self, id: i64) -> Option<&String>;
    fn save(&mut self, id: i64, username: String);
}

struct HashMapUserRepository {
    users: HashMap<i64, String>,
}

#[component]
impl HashMapUserRepository {
    #[provides]
    fn new() -> Self {
        HashMapUserRepository {
            users: HashMap::new(),
        }
    }
}

#[provides]
impl UserRepository for HashMapUserRepository {
    fn find(&self, id: i64) -> Option<&String> {
        self.users.get(&id)
    }

    fn save(&mut self, id: i64, username: String) {
        self.users.insert(id, username);
    }
}

#[module]
struct UserModule {
    repository: Box<dyn UserRepository>,
}

#[module]
struct RootModule {
    user_module: UserModule,
}

fn main() {
    let mut container = Container::<profiles::Dev>::new();

    let mut user_repository = Provider::<dyn UserRepository>::create(&mut container);

    user_repository.save(12, "John".to_string());

    println!("Found user with id = 12: {:?}", user_repository.find(12));
}
