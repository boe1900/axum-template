// src/services/kms_app_access_service.rs
// `kms_app_access` 相关的业务逻辑

use crate::errors::AppError; // 导入我们统一的错误类型
use crate::models::kms_app_access; // 导入实体模型
use crate::repository::kms_app_access_repo; // 导入 repository
use crate::state::AppState; // 导入共享状态

/// 根据 ID 获取 App Access
///
/// # Arguments
/// * `state` - 共享的 AppState，包含数据库连接池
/// * `id` - 要查找的 ID
pub async fn get_app_access_by_id(
    state: &AppState,
    id: i64,
) -> Result<kms_app_access::Model, AppError> {
    // 从 AppState 中获取数据库连接池
    let db = &state.db_pool;

    // 调用 repository 层
    let app_access = kms_app_access_repo::find_by_id(db, id)
        .await
        .map_err(AppError::DatabaseError)?; // 将 DbErr 转换为 AppError

    // 处理业务逻辑：如果未找到，返回 AppError::NotFound
    match app_access {
        Some(app) => Ok(app),
        None => Err(AppError::NotFound(format!("AppAccess with ID {} not found", id))),
    }
}
