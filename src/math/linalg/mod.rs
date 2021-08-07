//! Linear-algebra module.

pub mod dir;
pub mod mat;
pub mod pos;
pub mod rot;
pub mod trans;
pub mod vec;

pub use self::dir::*;
pub use self::mat::*;
pub use self::pos::*;
pub use self::rot::*;
pub use self::trans::*;
pub use self::vec::*;
