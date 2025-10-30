pub mod gatt;
pub mod ble_adapter;
pub mod transport;

/// Re-export the configure function for easy access as `ble::configure()`
pub use ble_adapter::configure;
