// src/response.rs (新文件)
// 定义统一的 API 响应结构，模仿 Java 的 R<T>

use serde::Serialize;

/// 统一的 API 响应结构
/// 泛型 T 代表 "data" 字段的类型
#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize, // 约束 T 必须是可序列化的
{
    // 状态码 (e.g., 200, 500)
    code: u16,
    
    // 消息 (e.g., "success", "Item not found")
    msg: String,
    
    // 数据 (T)
    // 使用 Option<T> 使得 "data" 字段在错误时可以为 null
    // serde(skip_serializing_if = "Option::is_none") 会在 data 为 None 时，
    // 从 JSON 序列化中完全省略 data 字段，更干净。
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

// --- 构造函数 ---

impl<T: Serialize> ApiResponse<T> {
    /// 创建一个成功的响应 (HTTP 200, data 为 Some(T))
    pub fn success(data: T) -> Self {
        Self {
            code: 0, // 默认成功码 (你也可以用 0 或其他)
            msg: "success".to_string(),
            data: Some(data),
        }
    }

    /// 创建一个失败的响应 (data 为 None)
    // --- 修改点 ---
    // 接收一个自定义的 业务 code (u16)，而不是 StatusCode
    pub fn error(code: u16, msg: String) -> ApiResponse<()> {
        ApiResponse {
            code, // 使用传入的自定义业务码
            msg,
            data: None,
        }
    }
}