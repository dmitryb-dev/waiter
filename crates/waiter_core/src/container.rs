use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::rc::Rc;
use std::marker::PhantomData;
use config::{Config, Environment, File};

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
        config.merge(File::with_name("config/default.toml").required(false))
            .expect("Failed to read default.toml config file");

        let profile = type_name::<T>().to_lowercase();
        if profile.ne(&"default.toml".to_owned()) {
            config.merge(File::with_name(&format!("config/{}", profile)).required(false))
                .expect(format!("Failed to read {} config file", profile).as_str());
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