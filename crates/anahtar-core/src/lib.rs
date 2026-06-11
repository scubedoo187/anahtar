//! Core KDBX operations for Anahtar.
//!
//! This crate intentionally exposes safe output structures for CLI/GUI use.
//! Password fields are not included in summaries and are only returned by
//! explicit detail requests with `reveal_password = true`.

mod internal;

pub mod audit;
pub mod credentials;
pub mod entries;
pub mod errors;
pub mod groups;
pub mod inspect;
pub mod selectors;
pub mod totp;
pub mod types;
pub mod write;

pub use audit::*;
pub use credentials::*;
pub use entries::*;
pub use errors::*;
pub use groups::*;
pub use inspect::*;
pub use selectors::*;
pub use totp::*;
pub use types::*;
pub use write::*;
