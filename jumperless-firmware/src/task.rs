/// Manages state of onboard LEDs
///
pub mod leds;

/// Watchdog task
///
/// Periodically nudges the watchdog to keep alive. If this task isn't scheduled for more than 1.5 seconds, the device resets.
pub mod watchdog;

pub mod net_manager;
