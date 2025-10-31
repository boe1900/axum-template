// src/models/kms_app_access.rs
// SeaORM 实体 (Entity) 定义，对应 `kms_app_access` 表
// 通常这个文件由 `sea-orm-cli` 自动生成，这里我们根据 SQL 手动编写

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize}; // 导入 Serialize 和 Deserialize

// --- 修改点 ---
// `chrono::NaiveDateTime` 用于映射 DATETIME (无时区)
// 我们需要 `chrono::DateTime` 和 `chrono::Utc` 来映射 TIMESTAMP (带时区)
use chrono::{DateTime, Utc};

/// 对应 `kms_app_access` 表的实体
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "kms_app_access")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)] // ID 通常由数据库生成
    pub id: i64, // 对应 bigint(20) NOT NULL
    pub access_info_id: i64, // 对应 bigint(20) NOT NULL
    pub app_access_key: String, // 对应 varchar(255) NOT NULL
    pub name: String, // 对应 varchar(255) NOT NULL
    pub mark: Option<String>, // 对应 varchar(50) DEFAULT NULL
    pub status: i8, // 对应 tinyint(1) NOT NULL (i8 足够)
    pub description: Option<String>, // 对应 varchar(2048) DEFAULT NULL

    // --- 修改点 ---
    // 映射 `create_time` 和 `update_time`
    // 将 NaiveDateTime 更改为 DateTime<Utc> 以匹配 TIMESTAMP
    pub create_time: DateTime<Utc>,
    pub create_by: String,
    pub update_time: DateTime<Utc>,
    pub update_by: String,

    pub del_flag: String, // 对应 char(1) NOT NULL
    pub show_id: Option<String>, // 对应 varchar(255) DEFAULT NULL
}

/// SeaORM 相关的 ActiveModel 行为
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

