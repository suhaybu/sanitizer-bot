use anyhow::Context;
use libsql::params;
use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::Message,
    id::{Id, marker::MessageMarker},
};

use super::connection::{get_connection, sync_database};
use crate::discord::models::{DeletePermission, SanitizerMode};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub guild_id: u64,
    pub sanitizer_mode: SanitizerMode,
    pub delete_permission: DeletePermission,
    pub hide_original_embed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ResponseMap {
    pub user_message_id: u64,
    pub bot_message_id: u64,
    pub guild_id: Option<u64>,
    pub channel_id: u64,
}

impl ResponseMap {
    pub fn new(user_message: &Message, bot_message_id: Id<MessageMarker>) -> Self {
        Self {
            user_message_id: user_message.id.get(),
            bot_message_id: bot_message_id.get(),
            guild_id: user_message.guild_id.map(|id| id.get()),
            channel_id: user_message.channel_id.into(),
        }
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let conn = get_connection()?;

        let sql = r#"
            INSERT OR REPLACE INTO response_map
            (user_message_id, bot_message_id, guild_id, channel_id)
            VALUES (?, ?, ?, ?)
        "#;

        conn.execute(
            sql,
            params![
                self.user_message_id as i64,
                self.bot_message_id as i64,
                self.guild_id.map(|id| id as i64),
                self.channel_id as i64,
            ],
        )
        .await
        .context("Failed to save response map")?;

        tracing::debug!(
            "Saved response map: user_msg={}, bot_msg={}, guild_id={:?}, channel_id={}",
            self.user_message_id,
            self.bot_message_id,
            self.guild_id,
            self.channel_id
        );

        tokio::spawn(async move {
            if let Err(e) = sync_database().await {
                tracing::warn!("Failed to sync database after write: {:?}", e);
            }
        });

        Ok(())
    }

    pub async fn find_match(deleted_message_id: Id<MessageMarker>) -> anyhow::Result<Option<Self>> {
        let conn = get_connection()?;

        let sql = r#"
            SELECT user_message_id, bot_message_id, guild_id, channel_id
            FROM response_map
            WHERE user_message_id = ?
        "#;

        let stmt = conn
            .prepare(sql)
            .await
            .context("Failed to prepare SELECT statement")?;

        let mut rows = stmt
            .query(params![deleted_message_id.get() as i64])
            .await
            .context("Failed to execute SELECT statement")?;

        if let Some(row) = rows.next().await.context("Failed to fetch row")? {
            tracing::debug!(
                "Found response map for user_message_id={}",
                deleted_message_id
            );
            Ok(Some(Self {
                user_message_id: deleted_message_id.get(),
                bot_message_id: row.get::<i64>(1)? as u64,
                guild_id: row.get::<Option<i64>>(2)?.map(|id| id as u64),
                channel_id: row.get::<i64>(3)? as u64,
            }))
        } else {
            tracing::debug!(
                "No response map found for user_message_id={}",
                deleted_message_id
            );
            Ok(None)
        }
    }

    pub async fn delete_entry(user_message_id: u64) -> anyhow::Result<()> {
        let conn = get_connection()?;

        let sql = "DELETE FROM response_map WHERE user_message_id = ?";

        conn.execute(sql, params![user_message_id as i64])
            .await
            .context("Failed to delete from response map")?;

        tracing::debug!(
            "Deleted response map for user_message_id={}",
            user_message_id
        );

        tokio::spawn(async move {
            if let Err(e) = sync_database().await {
                tracing::warn!("Failed to sync database after delete: {:?}", e);
            }
        });

        Ok(())
    }
}

impl ServerConfig {
    pub async fn save(&self) -> anyhow::Result<()> {
        let conn = get_connection()?;

        let sql = r#"
            INSERT OR REPLACE INTO server_configs
            (guild_id, sanitizer_mode, delete_permission, hide_original_embed)
            VALUES (?, ?, ?, ?)
        "#;

        conn.execute(
            sql,
            params![
                self.guild_id as i64,
                self.sanitizer_mode as i32,
                self.delete_permission as i32,
                self.hide_original_embed
            ],
        )
        .await
        .context("Failed to save server config")?;

        tracing::debug!("Saved config for guild {}", self.guild_id);
        tracing::debug!("{:?}", self);

        tokio::spawn(async move {
            if let Err(e) = sync_database().await {
                tracing::warn!("Failed to sync database after write: {:?}", e);
            }
        });

        Ok(())
    }

    pub async fn get_or_default(guild_id: u64) -> anyhow::Result<Self> {
        match Self::get(guild_id).await? {
            Some(config) => Ok(config),
            None => {
                tracing::debug!("Using default config for guild ({})", guild_id);
                Ok(Self::new(guild_id))
            }
        }
    }

    fn new(guild_id: u64) -> Self {
        Self {
            guild_id,
            sanitizer_mode: SanitizerMode::default(),
            delete_permission: DeletePermission::default(),
            hide_original_embed: true,
        }
    }

    async fn get(guild_id: u64) -> anyhow::Result<Option<Self>> {
        let conn = get_connection()?;

        let sql = r#"
            SELECT guild_id, sanitizer_mode, delete_permission, hide_original_embed
            FROM server_configs
            WHERE guild_id = ?
        "#;

        let stmt = conn
            .prepare(sql)
            .await
            .context("Failed to prepare SELECT statement")?;

        let mut rows = stmt
            .query(params![guild_id as i64])
            .await
            .context("Failed to execute SELECT query")?;

        if let Some(row) = rows.next().await.context("Failed to fetch row")? {
            tracing::debug!("Found existing config for guild {}", guild_id);
            Ok(Some(Self {
                guild_id,
                sanitizer_mode: row.get::<i32>(1)?.into(),
                delete_permission: row.get::<i32>(2)?.into(),
                hide_original_embed: row.get::<bool>(3)?,
            }))
        } else {
            tracing::debug!("No config found for guild {}, returning default", guild_id);
            Ok(None)
        }
    }

    // pub async fn delete(guild_id: u64) -> anyhow::Result<()> {
    //    let conn = get_connection()
    //        .context("Failed to get database connection")?;

    //     let sql = "DELETE FROM Sanitizer WHERE guild_id = ?";

    //     conn.execute(sql, params![guild_id as i64])
    //         .await
    //         .context("Failed to delete server config")?;

    //     debug!("Deleted config for guild {}", guild_id);
    //     Ok(())
    // }
}
