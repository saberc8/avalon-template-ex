mod response;
mod auth_handler;
mod captcha_handler;
mod common_handler;
mod user_handler;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::AppState;

pub use response::{api_fail, api_ok, ApiJson, PageResult};

/// 构建 Axum 路由树，对齐 backend-go 的路由结构。
pub fn build_router(state: Arc<AppState>, file_root: String) -> Router {
    // CORS 设置：开发阶段允许 localhost:3000，其他配置与 Go 版保持一致。
    let cors = CorsLayer::very_permissive()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any);

    let static_service = ServeDir::new(file_root);

    Router::new()
        // Auth & User
        .route("/auth/login", post(auth_handler::login))
        .route("/auth/user/info", get(user_handler::get_user_info))
        .route("/auth/user/route", get(user_handler::list_user_route))
        // Captcha
        .route("/captcha/image", get(captcha_handler::get_captcha_image))
        // Common
        .route("/common/dict/option/site", get(common_handler::list_site_options))
        .route("/common/tree/menu", get(common_handler::list_menu_tree))
        .route("/common/tree/dept", get(common_handler::list_dept_tree))
        .route("/common/dict/user", get(common_handler::list_user_dict))
        .route("/common/dict/role", get(common_handler::list_role_dict))
        .route(
            "/common/dict/:code",
            get(common_handler::list_dict_by_code),
        )
        // 静态文件访问，与 Go 版 r.Static("/file", fileRoot) 对齐
        .nest_service("/file", static_service)
        .layer(cors)
        .with_state(state)
}

