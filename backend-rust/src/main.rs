mod config;
mod db;
mod security;
mod application;
mod interfaces;
mod id;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;

use crate::config::AppConfig;
use crate::db::create_pool;
use crate::interfaces::http::build_router;
use crate::security::{BcryptVerifier, RsaDecryptor, TokenService};

/// 全局应用状态，供各个 Handler 通过状态注入共享。
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL 连接池
    pub db: db::DbPool,
    /// JWT Token 服务
    pub token_svc: TokenService,
    /// RSA 解密器（用于前端加密密码解密）
    pub rsa_decryptor: RsaDecryptor,
    /// BCrypt 密码校验器
    pub pwd_verifier: BcryptVerifier,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载 .env（如果存在），保持与其他后端实现一致的环境变量配置方式
    let _ = dotenvy::dotenv();

    // 读取环境变量配置（DB_*、AUTH_*、HTTP_PORT 等）
    let cfg = AppConfig::from_env();

    // 初始化数据库连接池
    let db_pool = create_pool(&cfg).await?;

    // 初始化安全组件：RSA 解密 + BCrypt 校验 + JWT 生成
    let rsa_decryptor = RsaDecryptor::from_base64(&cfg.auth_rsa_private_key)?;
    let pwd_verifier = BcryptVerifier::new();
    let token_svc = TokenService::new(cfg.auth_jwt_secret.clone(), cfg.auth_jwt_ttl_hours);

    let state = Arc::new(AppState {
        db: db_pool,
        token_svc,
        rsa_decryptor,
        pwd_verifier,
    });

    // 构建 HTTP 路由（对齐 backend-go：auth/user、captcha、common 等）
    let app: Router = build_router(state.clone(), cfg.file_storage_dir.clone());

    // 启动 HTTP 服务，端口与 Go/Java 默认值保持一致（可通过 HTTP_PORT 配置）
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.http_port));
    println!("backend-rust listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
