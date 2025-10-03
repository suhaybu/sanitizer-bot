use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsMenuType {
    SanitizerMode,
    DeletePermission,
    HideOriginalEmbed,
}

impl AsRef<str> for SettingsMenuType {
    fn as_ref(&self) -> &str {
        match self {
            Self::SanitizerMode => "sanitizer_mode",
            Self::DeletePermission => "delete_permission",
            Self::HideOriginalEmbed => "hide_original_embed",
        }
    }
}

impl FromStr for SettingsMenuType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sanitizer_mode" => Ok(Self::SanitizerMode),
            "delete_permission" => Ok(Self::DeletePermission),
            "hide_original_embed" => Ok(Self::HideOriginalEmbed),
            _ => Err(anyhow::anyhow!("Unknown settings menu type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(i32)]
#[derive(Default)]
pub enum SanitizerMode {
    #[default]
    Automatic = 0,
    ManualEmote = 1,
    ManualMention = 2,
    ManualBoth = 3,
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

impl AsRef<str> for SanitizerMode {
    fn as_ref(&self) -> &str {
        match self {
            Self::Automatic => "automatic",
            Self::ManualEmote => "manual_emote",
            Self::ManualMention => "manual_mention",
            Self::ManualBoth => "manual_both",
        }
    }
}

impl FromStr for SanitizerMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "automatic" => Ok(Self::Automatic),
            "manual_emote" => Ok(Self::ManualEmote),
            "manual_mention" => Ok(Self::ManualMention),
            "manual_both" => Ok(Self::ManualBoth),
            _ => Err(anyhow::anyhow!("Unknown sanitizer mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[repr(i32)]
#[derive(Default)]
pub enum DeletePermission {
    #[default]
    AuthorAndMods = 0,
    Everyone = 1,
    Disabled = 2,
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

impl AsRef<str> for DeletePermission {
    fn as_ref(&self) -> &str {
        match self {
            Self::AuthorAndMods => "author_and_mods",
            Self::Everyone => "everyone",
            Self::Disabled => "disabled",
        }
    }
}

impl FromStr for DeletePermission {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "author_and_mods" => Ok(Self::AuthorAndMods),
            "everyone" => Ok(Self::Everyone),
            "disabled" => Ok(Self::Disabled),
            _ => Err(anyhow::anyhow!("Unknown delete permission: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum HideOriginalEmbed {
    #[default]
    On,
    Off,
}


impl AsRef<str> for HideOriginalEmbed {
    fn as_ref(&self) -> &str {
        match self {
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

impl FromStr for HideOriginalEmbed {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err(anyhow::anyhow!("Unknown hide original embed setting: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizer_mode_component_ids() {
        // Test AsRef<str> conversion
        assert_eq!(SanitizerMode::Automatic.as_ref(), "automatic");
        assert_eq!(SanitizerMode::ManualEmote.as_ref(), "manual_emote");
        assert_eq!(SanitizerMode::ManualMention.as_ref(), "manual_mention");
        assert_eq!(SanitizerMode::ManualBoth.as_ref(), "manual_both");

        // Test parsing from component IDs using FromStr
        assert_eq!("automatic".parse::<SanitizerMode>().unwrap(), SanitizerMode::Automatic);
        assert_eq!("manual_emote".parse::<SanitizerMode>().unwrap(), SanitizerMode::ManualEmote);
        assert_eq!("manual_mention".parse::<SanitizerMode>().unwrap(), SanitizerMode::ManualMention);
        assert_eq!("manual_both".parse::<SanitizerMode>().unwrap(), SanitizerMode::ManualBoth);
    }

    #[test]
    fn test_delete_permission_component_ids() {
        // Test AsRef<str> conversion
        assert_eq!(DeletePermission::AuthorAndMods.as_ref(), "author_and_mods");
        assert_eq!(DeletePermission::Everyone.as_ref(), "everyone");
        assert_eq!(DeletePermission::Disabled.as_ref(), "disabled");

        // Test parsing from component IDs using FromStr
        assert_eq!("author_and_mods".parse::<DeletePermission>().unwrap(), DeletePermission::AuthorAndMods);
        assert_eq!("everyone".parse::<DeletePermission>().unwrap(), DeletePermission::Everyone);
        assert_eq!("disabled".parse::<DeletePermission>().unwrap(), DeletePermission::Disabled);
    }

    #[test]
    fn test_hide_original_embed_component_ids() {
        // Test AsRef<str> conversion
        assert_eq!(HideOriginalEmbed::On.as_ref(), "on");
        assert_eq!(HideOriginalEmbed::Off.as_ref(), "off");

        // Test parsing from component IDs using FromStr
        assert_eq!("on".parse::<HideOriginalEmbed>().unwrap(), HideOriginalEmbed::On);
        assert_eq!("off".parse::<HideOriginalEmbed>().unwrap(), HideOriginalEmbed::Off);
    }

    #[test]
    fn test_settings_menu_type_component_ids() {
        // Test AsRef<str> conversion
        assert_eq!(SettingsMenuType::SanitizerMode.as_ref(), "sanitizer_mode");
        assert_eq!(SettingsMenuType::DeletePermission.as_ref(), "delete_permission");
        assert_eq!(SettingsMenuType::HideOriginalEmbed.as_ref(), "hide_original_embed");

        // Test parsing from component IDs using FromStr
        assert_eq!("sanitizer_mode".parse::<SettingsMenuType>().unwrap(), SettingsMenuType::SanitizerMode);
        assert_eq!("delete_permission".parse::<SettingsMenuType>().unwrap(), SettingsMenuType::DeletePermission);
        assert_eq!("hide_original_embed".parse::<SettingsMenuType>().unwrap(), SettingsMenuType::HideOriginalEmbed);
    }

    #[test]
    fn test_roundtrip_serialization() {
        // Test that AsRef<str> and FromStr are consistent
        
        // SanitizerMode
        for variant in [SanitizerMode::Automatic, SanitizerMode::ManualEmote, SanitizerMode::ManualMention, SanitizerMode::ManualBoth] {
            let id = variant.as_ref();
            let parsed = id.parse::<SanitizerMode>().unwrap();
            assert_eq!(variant, parsed, "Failed roundtrip for SanitizerMode::{:?}", variant);
        }
        
        // DeletePermission
        for variant in [DeletePermission::AuthorAndMods, DeletePermission::Everyone, DeletePermission::Disabled] {
            let id = variant.as_ref();
            let parsed = id.parse::<DeletePermission>().unwrap();
            assert_eq!(variant, parsed, "Failed roundtrip for DeletePermission::{:?}", variant);
        }
        
        // HideOriginalEmbed
        for variant in [HideOriginalEmbed::On, HideOriginalEmbed::Off] {
            let id = variant.as_ref();
            let parsed = id.parse::<HideOriginalEmbed>().unwrap();
            assert_eq!(variant, parsed, "Failed roundtrip for HideOriginalEmbed::{:?}", variant);
        }
    }

    #[test]
    fn test_enum_default_values() {
        // Test that default values are correctly set
        assert_eq!(SanitizerMode::default(), SanitizerMode::Automatic);
        assert_eq!(DeletePermission::default(), DeletePermission::AuthorAndMods);
        assert_eq!(HideOriginalEmbed::default(), HideOriginalEmbed::On);
    }

    #[test]
    fn test_from_i32_conversion() {
        // Test that i32 conversion works correctly for database values
        assert_eq!(SanitizerMode::from(0), SanitizerMode::Automatic);
        assert_eq!(SanitizerMode::from(1), SanitizerMode::ManualEmote);
        assert_eq!(SanitizerMode::from(2), SanitizerMode::ManualMention);
        assert_eq!(SanitizerMode::from(3), SanitizerMode::ManualBoth);
        assert_eq!(SanitizerMode::from(999), SanitizerMode::Automatic); // Invalid value should default

        assert_eq!(DeletePermission::from(0), DeletePermission::AuthorAndMods);
        assert_eq!(DeletePermission::from(1), DeletePermission::Everyone);
        assert_eq!(DeletePermission::from(2), DeletePermission::Disabled);
        assert_eq!(DeletePermission::from(999), DeletePermission::AuthorAndMods); // Invalid value should default
    }

    #[test]
    fn test_as_ref_methods() {
        // Test that AsRef<str> implementations work correctly
        assert_eq!(SanitizerMode::Automatic.as_ref(), "automatic");
        assert_eq!(DeletePermission::Everyone.as_ref(), "everyone");
        assert_eq!(HideOriginalEmbed::Off.as_ref(), "off");
        assert_eq!(SettingsMenuType::SanitizerMode.as_ref(), "sanitizer_mode");
    }

    #[test]
    fn test_error_cases() {
        // Test invalid component IDs return errors
        assert!("invalid".parse::<SanitizerMode>().is_err());
        assert!("invalid".parse::<DeletePermission>().is_err());
        assert!("invalid".parse::<HideOriginalEmbed>().is_err());
        assert!("invalid".parse::<SettingsMenuType>().is_err());
    }
}