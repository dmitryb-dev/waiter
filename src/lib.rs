extern crate proc_macro;
extern crate config;
extern crate regex;
extern crate waiter_codegen;
#[macro_use]
extern crate lazy_static;

pub mod container;
pub mod deferred;

#[macro_use]
pub mod inject;

pub use waiter_codegen::*;

pub use container::*;
pub use deferred::*;
pub use inject::*;