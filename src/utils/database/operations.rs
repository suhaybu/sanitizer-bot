use anyhow::Context;
use libsql::params;
use serde::{Deserialize, Serialize};

use super::connection::{get_connection, sync_database};
use crate::discord::models::{DeletePermission, SanitizerMode};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct ServerConfig {
    pub guild_id: u64,
    pub sanitizer_mode: SanitizerMode,
    pub delete_permission: DeletePermission,
    pub hide_original_embed: bool,
}

impl ServerConfig {
    pub fn new(guild_id: u64) -> Self {
        Self {
            guild_id,
            sanitizer_mode: SanitizerMode::default(),
            delete_permission: DeletePermission::default(),
            hide_original_embed: true,
        }
    }

    pub async fn get_or_default(guild_id: u64) -> anyhow::Result<Self> {
        Self::get(guild_id)
            .await
            .or_else(|_| Ok(Self::new(guild_id)))
    }

    async fn get(guild_id: u64) -> anyhow::Result<Self> {
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
            Ok(Self {
                guild_id,
                sanitizer_mode: row.get::<i32>(1)?.into(),
                delete_permission: row.get::<i32>(2)?.into(),
                hide_original_embed: row.get::<bool>(3)?,
            })
        } else {
            tracing::debug!("No config found for guild {}, returning default", guild_id);
            Ok(ServerConfig::new(guild_id))
        }
    }

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
