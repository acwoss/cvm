//! Test-only helpers shared across `cvm-core`'s test modules.

#[cfg(test)]
use std::sync::Mutex;

/// Serializes every test across the crate that temporarily overrides the
/// process-global `HOME`/`CVM_HOME`/`CVM_USER_HOME` environment variables.
/// A per-module lock would only serialize tests within that one module -
/// tests in different modules that both mutate these same global vars
/// would still race under cargo's default multi-threaded test runner.
#[cfg(test)]
pub(crate) static HOME_LOCK: Mutex<()> = Mutex::new(());
