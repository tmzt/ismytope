pub mod eval;
pub mod idents;
pub mod process;
pub mod session;
pub mod state;

pub trait ScopeParentId {
    fn parent_id(&self) -> Option<&str>;
}

pub use self::eval::*;
pub use self::idents::*;
pub use self::process::*;
pub use self::session::*;
pub use self::state::*;
