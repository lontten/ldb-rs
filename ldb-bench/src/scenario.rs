//! 固定业务场景输入（各 ORM 后端必须对齐同一语义）。

/// 种子行数。
pub const SEED_N: usize = 500;

/// 搜索过滤：部分字段为 `None` 表示不进入 WHERE。
#[derive(Debug, Clone)]
pub struct Filter {
    pub name_like: Option<&'static str>,
    pub age_min: Option<i32>,
    pub age_max: Option<i32>,
    pub status: Option<i16>,
    pub city: Option<&'static str>,
}

/// 分页参数。
#[derive(Debug, Clone, Copy)]
pub struct Page {
    pub limit: u64,
    pub offset: u64,
}

/// 基准使用的过滤条件。
pub const FILTER: Filter = Filter {
    name_like: Some("user_%"),
    age_min: Some(18),
    age_max: None,
    status: Some(1),
    city: None,
};

/// 第 3 页：limit 20、offset 40。
pub const PAGE: Page = Page {
    limit: 20,
    offset: 40,
};

/// 部分更新写入的 age。
pub const PATCH_AGE: i32 = 99;

/// 部分更新写入的 city。
pub const PATCH_CITY: &str = "bench_city";

/// upsert / 冲突键对应的 name（种子中必存在）。
pub const UPSERT_NAME: &str = "user_0";

/// get_or_insert 查找的已存在 name。
pub const GET_OR_INSERT_NAME: &str = "user_1";

/// 批量删除的 id 列表（种子后为 1..=20）。
pub fn delete_id_list() -> [i64; 20] {
    std::array::from_fn(|i| (i + 1) as i64)
}

/// 种子行字段（id 从 1 起时 name 为 `user_{id-1}`）。
pub fn seed_row(index: usize) -> SeedRow {
    const CITIES: [&str; 4] = ["shanghai", "beijing", "guangzhou", "shenzhen"];
    SeedRow {
        name: format!("user_{index}"),
        age: (index % 60) as i32 + 10,
        status: (index % 3) as i16,
        city: Some(CITIES[index % CITIES.len()].to_string()),
    }
}

/// 单行种子数据。
#[derive(Debug, Clone)]
pub struct SeedRow {
    pub name: String,
    pub age: i32,
    pub status: i16,
    pub city: Option<String>,
}
