//! Parlance - P2P Messaging Library
//!
//! This library provides the core functionality for a peer-to-peer
//! messaging application over local networks.

pub mod app;
pub mod core;
pub mod network;

pub use core::{error, peer};
pub use network::{discovery, messaging};
