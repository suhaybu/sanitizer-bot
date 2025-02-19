use anyhow::{Context, Result};
use models::ServerConfig;
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, ServerApi, ServerApiVersion},
};
use std::sync::Arc;
use tokio::sync::OnceCell;

pub mod models;

static DB: OnceCell<Arc<DatabaseConnection>> = OnceCell::const_new();

pub struct DatabaseConnection {
    client: mongodb::Client,
    db: mongodb::Database,
}

impl DatabaseConnection {
    pub async fn new(mongodb_uri: &str, database_name: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(mongodb_uri)
            .await
            .context("Failed to parse MongoDB connection sring")?;

        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        let client = mongodb::Client::with_options(client_options)
            .context("Failed to create MongoDB client")?;

        let db = client.database(database_name);
        Ok(Self { client, db })
    }

    pub fn get_collection(&self, name: &str) -> mongodb::Collection<Document> {
        self.db.collection(name)
    }

    pub async fn init(mongodb_uri: &str, database_name: &str) -> Result<Arc<Self>> {
        let db = Self::new(mongodb_uri, database_name).await?;
        Ok(Arc::new(db))
    }

    pub async fn get() -> Result<Arc<Self>> {
        DB.get()
            .context("Database connection not initialized")
            .map(Arc::clone)
    }

    pub async fn initialize(mongodb_uri: &str, database_name: &str) -> Result<()> {
        let db = Self::init(mongodb_uri, database_name).await?;
        DB.set(db)
            .map_err(|_| anyhow::anyhow!("Database already initialized"))?;
        Ok(())
    }
}

pub async fn get_server_config(guild_id: i64) -> Result<ServerConfig> {
    let db = DatabaseConnection::get().await?;
    let collection = db.get_collection("server_configs");

    let filter = doc! { "_id": guild_id };
    let result = collection
        .find_one(filter)
        .await
        .context("Failed to fetch server config")?;

    match result {
        Some(doc) => {
            let config: ServerConfig =
                mongodb::bson::from_document(doc).context("Failed to fetch server config")?;
            Ok(config)
        }
        None => Ok(ServerConfig {
            guild_id,
            sanitizer_mode: models::SanitizerMode::Automatic,
            delete_permission: models::DeletePermission::AuthorAndMods,
            hide_original_embed: true,
        }),
    }
}

pub async fn update_server_config(config: &ServerConfig) -> Result<()> {
    let db = DatabaseConnection::get().await?;
    let collection = db.get_collection("server_configs");

    let filter = doc! { "_id": config.guild_id };
    let update = mongodb::bson::to_document(config).context("Failed to serialize config")?;

    collection
        .replace_one(filter, update)
        .await
        .context("Failed to update server config")?;

    Ok(())
}

pub async fn delete_server_config(guild_id: i64) -> Result<()> {
    let db = DatabaseConnection::get().await?;
    let collection = db.get_collection("server_configs");

    let filter = doc! { "_id": guild_id };
    collection
        .delete_one(filter)
        .await
        .context("Failed to delete server config");

    Ok(())
}
