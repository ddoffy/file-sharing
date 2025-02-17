use crate::db::redis::init_redis_with_env;

pub fn init_config() {
    init_redis_with_env();
}
