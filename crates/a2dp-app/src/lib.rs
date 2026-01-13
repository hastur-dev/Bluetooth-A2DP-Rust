//! A2DP Source Application
//!
//! Main application crate that orchestrates all components:
//! - USB Audio reception
//! - SBC encoding
//! - Bluetooth A2DP streaming

#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod config;
pub mod state_machine;

pub use bt_classic::a2dp::A2dpState;
pub use config::AppConfig;
pub use state_machine::StateMachine;
