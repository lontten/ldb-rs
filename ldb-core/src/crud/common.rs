//! CRUD Builder 共用字段。

#[derive(Debug, Clone, Default)]
pub struct BuilderFlags {
    pub table_name: Option<String>,
    pub show_sql: bool,
    pub dry_run: bool,
    pub skip_soft_delete: bool,
}

impl BuilderFlags {
    pub fn table_name(mut self, name: impl Into<String>) -> Self {
        self.table_name = Some(name.into());
        self
    }

    pub fn show_sql(mut self, enabled: bool) -> Self {
        self.show_sql = enabled;
        self
    }

    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.dry_run = enabled;
        self
    }

    pub fn skip_soft_delete(mut self, enabled: bool) -> Self {
        self.skip_soft_delete = enabled;
        self
    }

    pub fn resolve_table_name<M: crate::model::LdbModel>(&self) -> String {
        self.table_name
            .clone()
            .unwrap_or_else(|| M::table_conf().table_name.to_string())
    }
}

pub(crate) fn apply_soft_delete_filter<M: crate::model::LdbModel>(
    mut where_builder: crate::where_builder::WhereBuilder,
    skip: bool,
) -> crate::where_builder::WhereBuilder {
    if !skip && let Some(column) = M::table_conf().soft_delete_column {
        where_builder = where_builder.is_null(column);
    }
    where_builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TestUser;

    #[test]
    fn resolve_table_name_uses_override() {
        let flags = BuilderFlags::default().table_name("custom");
        assert_eq!(flags.resolve_table_name::<TestUser>(), "custom");
    }
}
