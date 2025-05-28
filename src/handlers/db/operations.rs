use anyhow::{Context, Result};
use libsql::params;
use tracing::debug;

use super::{connection::get_connection, models::ServerConfig};

impl ServerConfig {
    pub async fn get_or_default(guild_id: u64) -> Result<Self> {
        let conn = get_connection()?;

        let sql = "SELECT guild_id, sanitizer_mode, delete_permission, hide_original_embed
                           FROM Sanitizer WHERE guild_id = ?";

        let mut rows = conn
            .prepare(sql)
            .await
            .context("Failed to prepare SELECT statement")?
            .query(params![guild_id as i64])
            .await
            .context("Failed to execute SELECT query")?;

        if let Some(row) = rows.next().await.context("Failed to fetch row")? {
            debug!("Found existing config for guild {}", guild_id);
            Ok(ServerConfig {
                guild_id,
                sanitizer_mode: row.get::<u32>(1)?.into(),
                delete_permission: row.get::<u32>(2)?.into(),
                hide_original_embed: row.get::<bool>(3)?,
            })
        } else {
            debug!("No config found for guild {}, returning default", guild_id);
            Ok(ServerConfig::default(guild_id))
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
                self.sanitizer_mode as u32,
                self.delete_permission as u32,
                self.hide_original_embed
            ],
        )
        .await
        .context("Failed to save server config")?;

        debug!("Saved cofnig for guild {}", self.guild_id);
        Ok(())
    }

    pub async fn delete(guild_id: u64) -> Result<()> {
        let conn = get_connection()?;

        let sql = "DELETE FROM Sanitizer WHERE guild_id = ?";

        conn.execute(sql, params![guild_id as i64])
            .await
            .context("Failed to delete server config")?;

        debug!("Deleted config for guild {}", guild_id);
        Ok(())
    }
}
