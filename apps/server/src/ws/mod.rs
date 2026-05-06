//! WebSocket real-time sync
//!
//! Clients connect to /ws with a valid JWT token.
//! When any client uploads a new vault version, all other clients
//! for the same user are notified to pull the latest version.

pub mod handler;
