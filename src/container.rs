use crate::{RcAny, Wrc};
use config::{Config, Environment, File};
use lazy_static::lazy_static;
use regex::Regex;
use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::env;
use std::env::args;
use std::marker::PhantomData;

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
    type Impl;
    fn get(&mut self) -> Wrc<Self::Impl>;
    fn create(&mut self) -> Self::Impl;

    fn get_ref(&mut self) -> &Self::Impl {
        // Value under RC is still stored in container, so it can be safely returned as a reference
        // that has the same life as container reference
        unsafe { Wrc::as_ptr(&Self::get(self)).as_ref().unwrap() }
    }
    fn create_boxed(&mut self) -> Box<Self::Impl> {
        Box::new(Self::create(self))
    }
}

pub struct Container<P> {
    profile: PhantomData<P>,
    pub config: Config,
    pub components: HashMap<TypeId, RcAny>,
}

impl<P> Default for Container<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> Container<P> {
    pub fn new() -> Container<P> {
        let mut config = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(Environment::default())
            .add_source(parse_args());

        let profile = profile_name::<P>();
        if profile.ne(&"default".to_string()) {
            config =
                config.add_source(File::with_name(&format!("config/{}", profile)).required(false))
        }
        Container {
            config: config.build().expect("Failed to parse configuration"),
            profile: PhantomData::<P>,
            components: HashMap::new(),
        }
    }
}

lazy_static! {
    pub static ref APP_PROFILE: String = parse_profile();
}

fn parse_profile() -> String {
    let config = Config::builder()
        .add_source(File::with_name("config/default").required(false))
        .build()
        .expect("Failed to read default config file");

    let profile_arg = args()
        .position(|arg| arg.as_str() == "--profile")
        .and_then(|arg_pos| args().nth(arg_pos + 1));

    let parsed_profile = profile_arg
        .or_else(|| env::var("PROFILE").ok())
        .or_else(|| config.get_string("profile").ok())
        .unwrap_or_else(|| "default".to_string());

    log::info!("Using profile: {}", parsed_profile);

    parsed_profile
}

pub fn parse_args() -> Config {
    let mut config = Config::builder();

    let mut args = args().peekable();
    while let Some(arg) = args.next() {
        if let Some(arg_name) = arg.strip_prefix("--") {
            let value = args.peek();
            if value.is_none() || value.unwrap().starts_with("--") {
                config = config.set_override(arg_name, true).unwrap();
            } else {
                config = config.set_override(arg_name, args.next().unwrap()).unwrap();
            }
        }
    }

    config.build().unwrap()
}

pub fn profile_name<T>() -> String {
    let profile_type_name = type_name::<T>().to_lowercase();

    Regex::new(r".*::")
        .unwrap()
        .replace(profile_type_name.as_str(), "")
        .to_string()
}
