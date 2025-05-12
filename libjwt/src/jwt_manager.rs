use config::Config;
use crate::SharedTokenCache;
use crate::renewal::start_renewal_task;

pub struct JwtManager {
    pub token_cache: SharedTokenCache,
}

impl JwtManager {
    pub async fn init() -> Result<Self, Box<dyn std::error::Error>> {
        // Load configuration
        let config = Config::builder()
            .add_source(config::File::with_name("app_config.toml"))
            .build()
            .expect("Failed to load configuration");

        // Get storage configuration
        let storage_type = config.get_string("jwt_storage.storage_type")
            .expect("Missing jwt_storage.storage_type in config");

        println!("Using storage type: {}", storage_type);
        
        // Create token cache based on configuration
        let token_cache = match storage_type.as_str() {
            "local_db" => {
                let db_path = config.get_string("jwt_storage.db_path")
                    .expect("Missing jwt_storage.db_path in config");
                println!("Using SQLite database at: {}", db_path);
                SharedTokenCache::with_sqlite(&db_path).await?
            }
            "in_memory" | _ => SharedTokenCache::new(),
        };

        // Start the token renewal background task
        let renewal_cache = token_cache.clone();
        tokio::spawn(async move {
            start_renewal_task(renewal_cache).await;
        });

        Ok(Self { token_cache })
    }
}