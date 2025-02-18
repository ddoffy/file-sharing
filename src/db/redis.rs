use deadpool_redis::{Config, Connection, Pool, Runtime};
use once_cell::sync::OnceCell;

pub static REDIS_POOL: OnceCell<Pool> = OnceCell::new();

pub struct RedisConfig {
    pub host: String,
    pub port: String,
}

pub fn init_redis(config: RedisConfig) {
    let redis_url = format!("redis://{}:{}", config.host, config.port);
    let cfg = Config::from_url(redis_url);
    let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
    REDIS_POOL.set(pool).unwrap();
}

pub fn init_redis_with_env() {
    let cfg = get_cfg();
    init_redis(cfg);
}

pub fn get_cfg() -> RedisConfig {
    RedisConfig {
        host: std::env::var("REMOTE_REDIS_ADDR").unwrap_or("jetson.local".to_string()),
        port: std::env::var("REMOTE_REDIS_PORT").unwrap_or("6379".to_string()),
    }
}

pub fn get_pool() -> Pool {
    REDIS_POOL.get().unwrap().clone()
}

pub async fn get_redis_conn() -> Connection {
    let pool = get_pool();
    pool.get().await.unwrap()
}
