use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::rc::Rc;
use std::env;
use std::marker::PhantomData;
use config::{Config, Environment, File};
use regex::Regex;

pub mod profiles {
    pub struct Default;
    pub struct Dev;
    pub struct Test;
}

pub trait Component {
    fn __waiter_create<P>(container: &mut Container<P>) -> Self;
    fn __waiter_inject_deferred<P>(container: &mut Container<P>, component: &Self);
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

impl<P> Container<P> {
    pub fn new() -> Container<P> {
        let mut config = Config::new();
        config.merge(File::with_name("config/default").required(false))
            .expect("Failed to read default config file");

        let profile = profile_name::<P>();
        if profile.ne(&"default".to_string()) {
            config.merge(File::with_name(&format!("config/{}", profile)).required(false))
                .expect(format!("Failed to read {} config file", profile).as_str());
        }

        config.merge(Environment::new())
            .expect("Failed to load environment");

        Container {
            config,
            profile: PhantomData::<P>,
            components: HashMap::new()
        }
    }
}


lazy_static! {
    pub static ref APP_PROFILE: String = parse_profile();
}

fn parse_profile() -> String {
    let mut config = Config::new();

    config.merge(File::with_name("config/default").required(false))
        .expect("Failed to read default config file");

    let parsed_profile = env::var("PROFILE")
        .or(config.get_str("profile"))
        .unwrap_or("default".to_string());

    log::info!("Using profile: {}", parsed_profile);

    parsed_profile
}

pub fn profile_name<T>() -> String {
    let profile_type_name = type_name::<T>().to_lowercase();

    Regex::new(r".*::").unwrap()
        .replace(profile_type_name.as_str(), "")
        .to_string()
}