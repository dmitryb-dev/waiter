use std::any::Any;

pub use container::*;
pub use deferred::*;
pub use waiter_codegen::*;

pub mod container;
pub mod deferred;

#[macro_use]
pub mod inject;

#[cfg(feature = "async")]
pub type Wrc<T> = std::sync::Arc<T>;

#[cfg(not(feature = "async"))]
pub type Wrc<T> = std::rc::Rc<T>;


#[cfg(feature = "async")]
pub type RcAny = Wrc<dyn Any + Send + Sync>;

#[cfg(not(feature = "async"))]
pub type RcAny = Wrc<dyn Any>;