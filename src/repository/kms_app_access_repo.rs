// src/repository/kms_app_access_repo.rs
// 负责 `kms_app_access` 表的数据库访问逻辑

use crate::models::kms_app_access::{self, Entity as KmsAppAccess}; // 导入实体
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, QueryFilter, ColumnTrait};

/// 根据主键 ID 查找 KmsAppAccess
///
/// # Arguments
/// * `db` - `DatabaseConnection` (数据库连接池)
/// * `id` - 要查找的主键 ID
pub async fn find_by_id(
    db: &DatabaseConnection,
    id: i64,
) -> Result<Option<kms_app_access::Model>, DbErr> {
    KmsAppAccess::find_by_id(id).one(db).await
}

/// (示例) 根据 name 查找 KmsAppAccess
///
/// # Arguments
/// * `db` - `DatabaseConnection`
/// * `name` - 要查找的应用名称
#[allow(dead_code)] 
pub async fn find_by_name(
    db: &DatabaseConnection,
    name: &str,
) -> Result<Option<kms_app_access::Model>, DbErr> {
    KmsAppAccess::find()
        .filter(kms_app_access::Column::Name.eq(name))
        .one(db)
        .await
}
