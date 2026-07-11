use std::sync::Mutex;

/// Closed session modes enforced by Rust before external work begins.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectivityMode {
    Online,
    Offline,
}

/// Connectivity mode used at the start of every new application session.
pub const DEFAULT_CONNECTIVITY_MODE: ConnectivityMode = ConnectivityMode::Online;

/// Shared Rust-owned connectivity state for the current application session.
#[derive(Default)]
pub struct ConnectivityPolicy {
    mode: Mutex<ConnectivityMode>,
}

impl Default for ConnectivityMode {
    fn default() -> Self {
        DEFAULT_CONNECTIVITY_MODE
    }
}

/// Bounded failures from reading or enforcing connectivity state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectivityPolicyError {
    Offline,
    Unavailable,
}

impl ConnectivityPolicy {
    /// Returns the effective Rust-owned session mode.
    pub fn mode(&self) -> Result<ConnectivityMode, ConnectivityPolicyError> {
        self.mode
            .lock()
            .map(|mode| *mode)
            .map_err(|_| ConnectivityPolicyError::Unavailable)
    }

    /// Replaces the effective mode with one closed value.
    pub fn set_mode(
        &self,
        mode: ConnectivityMode,
    ) -> Result<ConnectivityMode, ConnectivityPolicyError> {
        let mut current = self
            .mode
            .lock()
            .map_err(|_| ConnectivityPolicyError::Unavailable)?;
        *current = mode;
        Ok(mode)
    }

    /// Rejects external work while the session is offline.
    pub fn require_online(&self) -> Result<(), ConnectivityPolicyError> {
        match self.mode()? {
            ConnectivityMode::Online => Ok(()),
            ConnectivityMode::Offline => Err(ConnectivityPolicyError::Offline),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_defaults_online_and_round_trips_closed_modes() {
        let policy = ConnectivityPolicy::default();

        assert_eq!(policy.mode(), Ok(ConnectivityMode::Online));
        assert_eq!(policy.require_online(), Ok(()));
        assert_eq!(
            policy.set_mode(ConnectivityMode::Offline),
            Ok(ConnectivityMode::Offline)
        );
        assert_eq!(policy.mode(), Ok(ConnectivityMode::Offline));
        assert_eq!(
            policy.require_online(),
            Err(ConnectivityPolicyError::Offline)
        );
    }
}
