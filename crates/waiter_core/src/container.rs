use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::rc::Rc;
use std::marker::PhantomData;
use config::{Config, Environment, File};
use std::env;

pub mod profiles {
    pub struct Default;
    pub struct Dev;
}

pub trait Component {
    fn __waiter_create<P>(container: &mut Container<P>) -> Self;
    fn __waiter_inject_deferred<P>(container: &mut Container<P>, component: Rc<Self>);
}

pub trait Provider<T: ?Sized> {
    fn get(&mut self) -> Rc<T>;
    fn get_ref(&mut self) -> &T;
    fn create(&mut self) -> Box<T>;
}

pub struct Container<P> {
    profile: PhantomData<P>,
    pub config: Config,
    pub components: HashMap<TypeId, Rc<dyn Any>>
}

impl<T> Container<T> {
    pub fn new() -> Container<T> {
        let mut config = Config::new();
        config.merge(File::with_name("config/default").required(false))
            .expect("Failed to read default config file");

        let mut profile: Option<String> = None;
        let env_profile = env::var("PROFILE");
        if env_profile.is_ok() {
            profile = Some(env_profile.unwrap_or_default());
        } else {
            let file_profile = config.get_str("profile");
            if file_profile.is_ok() {
                profile = Some(file_profile.unwrap());
            }
        }
        if profile.is_some() {
            config.merge(File::with_name(&format!("config/{}", profile.unwrap())).required(false))
                .expect("Failed to read profile specify config file");
        }

        config.merge(Environment::new())
            .expect("Failed to load environment");

        Container {
            config,
            profile: PhantomData::<T>,
            components: HashMap::new()
        }
    }
}