//! UIkit related helpers

use std::fmt;
use stdweb::js;

/// The UIkit service
pub struct UIkitService;

/// Possible status for notifications
pub enum NotificationStatus {
    Warning,
    Danger,
}

impl fmt::Display for NotificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationStatus::Warning => write!(f, "warning"),
            NotificationStatus::Danger => write!(f, "danger"),
        }
    }
}

impl UIkitService {
    /// Create a new UIkitService instance
    pub fn new() -> Self {
        Self {}
    }

    /// Create a new notification
    pub fn notify(&self, message: &str, status: &NotificationStatus) {
        js! {
            UIkit.notification({
                message: @{message},
                status: @{status.to_string()},
                timeout: 3000,
            });
        };
    }
}
