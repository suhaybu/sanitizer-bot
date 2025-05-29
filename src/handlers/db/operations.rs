use anyhow::{Context, Result};
use libsql::params;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::connection::get_connection;
use super::models::{DeletePermission, SanitizerMode};

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

impl ServerConfig {
    pub async fn get_or_default(guild_id: u64) -> Result<Self> {
        let conn = get_connection()?;

        let sql = "SELECT guild_id, sanitizer_mode, delete_permission, hide_original_embed
                           FROM Sanitizer WHERE guild_id = ?";

        match conn.prepare(sql).await {
            Ok(mut stmt) => {
                let mut rows = stmt
                    .query(params![guild_id as i64])
                    .await
                    .context("Failed to execute SELECT query")?;

                if let Some(row) = rows.next().await.context("Failed to fetch row")? {
                    debug!("Found existing config for guild {}", guild_id);
                    Ok(ServerConfig {
                        guild_id,
                        sanitizer_mode: row.get::<i32>(1)?.into(),
                        delete_permission: row.get::<i32>(2)?.into(),
                        hide_original_embed: row.get::<bool>(3)?,
                    })
                } else {
                    debug!("No config found for guild {}, returning default", guild_id);
                    Ok(ServerConfig::default(guild_id))
                }
            }
            Err(e) => {
                warn!(
                    "Failed to prepare SELECT statement for guild {}: {}",
                    guild_id, e
                );
                warn!("Returning default config for guild {}", guild_id);
                Ok(ServerConfig::default(guild_id))
            }
        }
    }

    pub async fn save(&self) -> Result<()> {
        let conn = get_connection()?;

        let sql = r#"
            INSERT OR REPLACE INTO Sanitizer
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

        debug!("Saved cofnig for guild {}", self.guild_id);
        Ok(())
    }

    // pub async fn delete(guild_id: u64) -> Result<()> {
    //     let conn = get_connection()?;

    //     let sql = "DELETE FROM Sanitizer WHERE guild_id = ?";

    //     conn.execute(sql, params![guild_id as i64])
    //         .await
    //         .context("Failed to delete server config")?;

    //     debug!("Deleted config for guild {}", guild_id);
    //     Ok(())
    // }
}
