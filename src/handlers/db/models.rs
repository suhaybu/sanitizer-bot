use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(u32)]
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

impl From<u32> for SanitizerMode {
    fn from(value: u32) -> Self {
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
#[repr(u32)]
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

impl From<u32> for DeletePermission {
    fn from(value: u32) -> Self {
        match value {
            0 => DeletePermission::AuthorAndMods,
            1 => DeletePermission::Everyone,
            2 => DeletePermission::Disabled,
            _ => DeletePermission::AuthorAndMods, // Default
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub guild_id: u64,
    pub sanitizer_mode: SanitizerMode,
    pub delete_permission: DeletePermission,
    pub hide_original_embed: bool,
}

impl ServerConfig {
    pub fn default(guild_id: u64) -> Self {
        Self {
            guild_id,
            sanitizer_mode: SanitizerMode::Automatic,
            delete_permission: DeletePermission::AuthorAndMods,
            hide_original_embed: false,
        }
    }
}
