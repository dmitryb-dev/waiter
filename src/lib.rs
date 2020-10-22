pub mod container;
pub mod deferred;

#[macro_use]
pub mod inject;

pub use waiter_codegen::*;

pub use container::*;
pub use deferred::*;
pub use inject::*;
use std::any::Any;


#[cfg(feature = "async")]
pub type Rc<T> = std::sync::Arc<T>;

#[cfg(not(feature = "async"))]
pub type Rc<T> = std::rc::Rc<T>;


#[cfg(feature = "async")]
pub type RcAny = Rc<dyn Any + Send + Sync>;

#[cfg(not(feature = "async"))]
pub type RcAny = Rc<dyn Any>;