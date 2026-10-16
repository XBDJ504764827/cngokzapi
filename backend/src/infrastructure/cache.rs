//! Redis 连接。`Client` 本身不建连，首次命令才连接；无库时启动不受影响。

/// 构造 Redis 客户端。排行榜等读多写少接口后续经此处缓存（TTL 见各模块 service）。
pub fn connect(redis_url: &str) -> redis::Client {
    redis::Client::open(redis_url).expect("REDIS_URL 非法")
}
