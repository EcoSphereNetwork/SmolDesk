// mod.rs - Main module definition for input forwarding system

// Re-export public modules
pub mod types;
pub mod error;
pub mod forwarder_trait;
pub mod x11;
pub mod wayland;
pub mod factory;
pub mod utils;

// Re-export public items for easier access
pub use types::*;
pub use error::*;
pub use forwarder_trait::*;
pub use factory::*;

// This allows importing the most common elements directly:
// use crate::input_forwarding::*;
