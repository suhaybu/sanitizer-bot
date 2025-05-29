use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(i32)]
pub enum SanitizerMode {
    Automatic = 0,
    ManualEmote = 1,
    ManualMention = 2,
    ManualBoth = 3,
}

impl Default for SanitizerMode {
    fn default() -> Self {
        SanitizerMode::Automatic
    }
}

impl From<i32> for SanitizerMode {
    fn from(value: i32) -> Self {
        match value {
            0 => SanitizerMode::Automatic,
            1 => SanitizerMode::ManualEmote,
            2 => SanitizerMode::ManualMention,
            3 => SanitizerMode::ManualBoth,
            _ => SanitizerMode::Automatic, // Default
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(i32)]
pub enum DeletePermission {
    AuthorAndMods = 0,
    Everyone = 1,
    Disabled = 2,
}

impl Default for DeletePermission {
    fn default() -> Self {
        DeletePermission::AuthorAndMods
    }
}

impl From<i32> for DeletePermission {
    fn from(value: i32) -> Self {
        match value {
            0 => DeletePermission::AuthorAndMods,
            1 => DeletePermission::Everyone,
            2 => DeletePermission::Disabled,
            _ => DeletePermission::AuthorAndMods, // Default
        }
    }
}
