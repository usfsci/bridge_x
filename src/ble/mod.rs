pub mod gatt;
pub mod ble_adapter;

/// Re-export the configure function for easy access as `ble::configure()`
pub use ble_adapter::configure;
