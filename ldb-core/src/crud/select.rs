//! Select 族 Builder 与入口。

use std::future::Future;
use std::pin::Pin;

use crate::crud::common::BuilderFlags;
use crate::engine::Engine;
use crate::error::LdbError;
use crate::model::LdbModel;
use crate::order::{Order, OrderBy};
use crate::sql_build::{SelectKind, build_select};
use crate::where_builder::WhereBuilder;

struct SelectState {
    flags: BuilderFlags,
    where_builder: Option<WhereBuilder>,
    order_by_list: Vec<OrderBy>,
    limit: Option<u64>,
    offset: Option<u64>,
}

macro_rules! select_builder {
    ($name:ident, $kind:expr) => {
        pub struct $name<'a, E, T> {
            engine: &'a E,
            _marker: std::marker::PhantomData<T>,
            state: SelectState,
        }
    };
}

select_builder!(FirstBuilder, SelectKind::First);
select_builder!(ListBuilder, SelectKind::List);
select_builder!(HasBuilder, SelectKind::Has);
select_builder!(CountBuilder, SelectKind::Count);

pub struct GetOrInsertBuilder<'a, E, T> {
    engine: &'a E,
    candidate: &'a mut T,
    state: SelectState,
    on_conflict: Option<crate::on_conflict::OnConflict>,
}

macro_rules! select_entry {
    ($fn:ident, $builder:ident) => {
        pub fn $fn<'a, T, E>(engine: &'a E) -> $builder<'a, E, T>
        where
            T: LdbModel,
            E: Engine,
        {
            $builder {
                engine,
                _marker: std::marker::PhantomData,
                state: SelectState {
                    flags: BuilderFlags::default(),
                    where_builder: None,
                    order_by_list: vec![],
                    limit: None,
                    offset: None,
                },
            }
        }
    };
}

select_entry!(first, FirstBuilder);
select_entry!(list, ListBuilder);
select_entry!(has, HasBuilder);
select_entry!(count, CountBuilder);

pub fn get_or_insert<'a, T, E>(engine: &'a E, candidate: &'a mut T) -> GetOrInsertBuilder<'a, E, T>
where
    T: LdbModel + Clone + Default,
    E: Engine,
{
    GetOrInsertBuilder {
        engine,
        candidate,
        state: SelectState {
            flags: BuilderFlags::default(),
            where_builder: None,
            order_by_list: vec![],
            limit: None,
            offset: None,
        },
        on_conflict: None,
    }
}

macro_rules! select_methods {
    ($t:ty) => {
        impl<'a, E, T> $t
        where
            T: LdbModel,
            E: Engine,
        {
            pub fn where_(mut self, wb: WhereBuilder) -> Self {
                self.state.where_builder = Some(wb);
                self
            }

            pub fn table_name(mut self, name: impl Into<String>) -> Self {
                self.state.flags = self.state.flags.table_name(name);
                self
            }

            pub fn order_by(mut self, column: &str, order: Order) -> Self {
                self.state.order_by_list.push(OrderBy {
                    column: column.to_string(),
                    order,
                });
                self
            }

            pub fn limit(mut self, n: u64) -> Self {
                self.state.limit = Some(n);
                self
            }

            pub fn offset(mut self, n: u64) -> Self {
                self.state.offset = Some(n);
                self
            }

            pub fn show_sql(mut self, enabled: bool) -> Self {
                self.state.flags = self.state.flags.show_sql(enabled);
                self
            }

            pub fn dry_run(mut self, enabled: bool) -> Self {
                self.state.flags = self.state.flags.dry_run(enabled);
                self
            }
        }
    };
}

select_methods!(FirstBuilder<'a, E, T>);
select_methods!(ListBuilder<'a, E, T>);
select_methods!(HasBuilder<'a, E, T>);
select_methods!(CountBuilder<'a, E, T>);

impl<'a, E, T> GetOrInsertBuilder<'a, E, T>
where
    T: LdbModel + Clone + Default,
    E: Engine,
{
    pub fn where_(mut self, wb: WhereBuilder) -> Self {
        self.state.where_builder = Some(wb);
        self
    }

    pub fn on_conflict(mut self, action: crate::on_conflict::OnConflict) -> Self {
        self.on_conflict = Some(action);
        self
    }

    pub fn show_sql(mut self, enabled: bool) -> Self {
        self.state.flags = self.state.flags.show_sql(enabled);
        self
    }

    pub fn dry_run(mut self, enabled: bool) -> Self {
        self.state.flags = self.state.flags.dry_run(enabled);
        self
    }
}

async fn run_select<E: Engine, T: LdbModel>(
    _engine: &E,
    state: &SelectState,
    kind: SelectKind,
) -> Result<crate::sql_build::BuiltSql, LdbError> {
    let wb = state.where_builder.clone().unwrap_or_default();
    if wb.is_empty() {
        return Err(LdbError::WhereRequired);
    }
    let table = state.flags.resolve_table_name::<T>();
    let built = build_select::<T>(
        &table,
        &wb,
        kind,
        &state.order_by_list,
        state.limit,
        state.offset,
    )?;
    if state.flags.show_sql {
        eprintln!("SQL: {} {:?}", built.sql, built.arg_list);
    }
    Ok(built)
}

impl<'a, E, T> IntoFuture for CountBuilder<'a, E, T>
where
    T: LdbModel,
    E: Engine,
{
    type Output = Result<u64, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let built = run_select::<E, T>(self.engine, &self.state, SelectKind::Count).await?;
            if self.state.flags.dry_run {
                return Ok(0);
            }
            self.engine.query_scalar_u64(&built).await
        })
    }
}

impl<'a, E, T> IntoFuture for HasBuilder<'a, E, T>
where
    T: LdbModel,
    E: Engine,
{
    type Output = Result<bool, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let built = run_select::<E, T>(self.engine, &self.state, SelectKind::Has).await?;
            if self.state.flags.dry_run {
                return Ok(false);
            }
            self.engine.query_exists(&built).await
        })
    }
}

impl<'a, E, T> IntoFuture for FirstBuilder<'a, E, T>
where
    T: LdbModel + Default,
    E: Engine,
{
    type Output = Result<Option<T>, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let built = run_select::<E, T>(self.engine, &self.state, SelectKind::First).await?;
            if self.state.flags.dry_run {
                return Ok(None);
            }
            let rows = self.engine.fetch_models::<T>(&built).await?;
            Ok(rows.into_iter().next())
        })
    }
}

impl<'a, E, T> IntoFuture for ListBuilder<'a, E, T>
where
    T: LdbModel + Default,
    E: Engine,
{
    type Output = Result<Vec<T>, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let built = run_select::<E, T>(self.engine, &self.state, SelectKind::List).await?;
            if self.state.flags.dry_run {
                return Ok(vec![]);
            }
            self.engine.fetch_models::<T>(&built).await
        })
    }
}

impl<'a, E, T> IntoFuture for GetOrInsertBuilder<'a, E, T>
where
    T: LdbModel + Clone + Default,
    E: Engine,
{
    type Output = Result<T, LdbError>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let wb = self.state.where_builder.unwrap_or_default();
            if wb.is_empty() {
                return Err(LdbError::WhereRequired);
            }
            let existing = first::<T, _>(self.engine).where_(wb.clone());
            if let Some(row) = existing.await? {
                return Ok(row);
            }
            crate::crud::insert::insert(self.engine, self.candidate)
                .on_conflict(
                    self.on_conflict
                        .unwrap_or(crate::on_conflict::OnConflict::DoNothing),
                )
                .await?;
            Ok(self.candidate.clone())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exec::MockExecutor;
    use crate::test_util::TestUser;
    use crate::where_builder::w;

    #[tokio::test]
    async fn count_dry_run() {
        let mock = MockExecutor::default();
        let n = count::<TestUser, _>(&mock)
            .where_(w().gt("id", 0))
            .dry_run(true)
            .await
            .unwrap();
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn has_first_list_dry_run() {
        let mock = MockExecutor::default();
        let wb = w().eq("id", 1);
        assert!(
            !has::<TestUser, _>(&mock)
                .where_(wb.clone())
                .dry_run(true)
                .await
                .unwrap()
        );
        assert!(
            first::<TestUser, _>(&mock)
                .where_(wb.clone())
                .dry_run(true)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            list::<TestUser, _>(&mock)
                .where_(wb)
                .dry_run(true)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn get_or_insert_requires_where() {
        let mock = MockExecutor::default();
        let mut user = TestUser::default();
        let err = get_or_insert(&mock, &mut user).await.unwrap_err();
        assert!(matches!(err, LdbError::WhereRequired));
    }

    #[tokio::test]
    async fn count_and_has_execute_on_mock() {
        let mock = MockExecutor::default();
        let wb = w().eq("id", 1);
        let n = count::<TestUser, _>(&mock)
            .where_(wb.clone())
            .await
            .unwrap();
        assert_eq!(n, 1);
        assert!(has::<TestUser, _>(&mock).where_(wb).await.unwrap());
    }

    #[tokio::test]
    async fn first_and_list_execute_on_mock() {
        use crate::sql_value::SqlValue;

        let mock = MockExecutor::default();
        mock.set_mock_rows(vec![vec![
            ("id".into(), SqlValue::I64(1)),
            ("name".into(), SqlValue::String("tom".into())),
            ("age".into(), SqlValue::I64(18)),
        ]]);

        let user = first::<TestUser, _>(&mock)
            .where_(w().eq("id", 1))
            .await
            .unwrap()
            .expect("expected row");
        assert_eq!(user.name.as_deref(), Some("tom"));
        assert_eq!(user.age, Some(18));

        let rows = list::<TestUser, _>(&mock)
            .where_(w().eq("id", 1))
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name.as_deref(), Some("tom"));
    }

    #[tokio::test]
    async fn get_or_insert_returns_existing_on_mock() {
        use crate::sql_value::SqlValue;

        let mock = MockExecutor::default();
        mock.set_mock_rows(vec![vec![
            ("id".into(), SqlValue::I64(9)),
            ("name".into(), SqlValue::String("exists".into())),
            ("age".into(), SqlValue::I64(30)),
        ]]);

        let mut candidate = TestUser {
            id: None,
            name: Some("exists".into()),
            age: Some(99),
        };
        let row = get_or_insert(&mock, &mut candidate)
            .where_(w().eq("name", "exists"))
            .await
            .unwrap();
        assert_eq!(row.age, Some(30));
    }
}
