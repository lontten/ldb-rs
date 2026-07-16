//! SeaORM 后端。

use sea_orm::sea_query::{MysqlQueryBuilder, OnConflict, PostgresQueryBuilder};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, Database, DatabaseConnection,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set, Statement,
};

use crate::scenario::{
    FILTER, GET_OR_INSERT_NAME, PAGE, PATCH_AGE, PATCH_CITY, UPSERT_NAME, delete_id_list,
};
use crate::setup;
use crate::{DbKind, Scenario};

mod entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "t_user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: String,
        pub age: i32,
        pub status: i16,
        pub city: Option<String>,
        pub created_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

use entity::{ActiveModel, Column, Entity};

fn filter_condition() -> Condition {
    let mut cond = Condition::all();
    if let Some(pattern) = FILTER.name_like {
        cond = cond.add(Column::Name.like(pattern));
    }
    if let Some(v) = FILTER.age_min {
        cond = cond.add(Column::Age.gte(v));
    }
    if let Some(v) = FILTER.age_max {
        cond = cond.add(Column::Age.lte(v));
    }
    if let Some(v) = FILTER.status {
        cond = cond.add(Column::Status.eq(v));
    }
    if let Some(v) = FILTER.city {
        cond = cond.add(Column::City.eq(v));
    }
    cond
}

async fn connect(db: DbKind) -> Result<DatabaseConnection, sea_orm::DbErr> {
    Database::connect(db.url().expect("database url")).await
}

pub async fn run(
    db: DbKind,
    scenario: Scenario,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = connect(db).await?;
    match scenario {
        Scenario::FilterPage => {
            let _ = Entity::find()
                .filter(filter_condition())
                .order_by_asc(Column::Id)
                .limit(PAGE.limit)
                .offset(PAGE.offset)
                .all(&conn)
                .await?;
        }
        Scenario::PageWithTotal => {
            let _ = Entity::find()
                .filter(filter_condition())
                .count(&conn)
                .await?;
            let _ = Entity::find()
                .filter(filter_condition())
                .order_by_asc(Column::Id)
                .limit(PAGE.limit)
                .offset(PAGE.offset)
                .all(&conn)
                .await?;
        }
        Scenario::PartialUpdate => {
            Entity::update_many()
                .col_expr(Column::Age, sea_orm::sea_query::Expr::value(PATCH_AGE))
                .col_expr(Column::City, sea_orm::sea_query::Expr::value(PATCH_CITY))
                .filter(Column::Status.eq(1i16))
                .exec(&conn)
                .await?;
        }
        Scenario::Upsert => {
            let mut insert = Entity::insert(ActiveModel {
                id: sea_orm::NotSet,
                name: Set(UPSERT_NAME.to_string()),
                age: Set(PATCH_AGE),
                status: Set(1),
                city: Set(Some("shanghai".to_string())),
                created_at: sea_orm::NotSet,
            });
            match db {
                DbKind::Mysql => {
                    insert = insert.on_conflict(
                        OnConflict::column(Column::Name)
                            .update_column(Column::Age)
                            .to_owned(),
                    );
                }
                DbKind::Postgres => {
                    insert = insert.on_conflict(
                        OnConflict::column(Column::Name)
                            .update_column(Column::Age)
                            .to_owned(),
                    );
                }
            }
            insert.exec_without_returning(&conn).await?;
        }
        Scenario::DeleteByIds => {
            let ids = delete_id_list();
            Entity::delete_many()
                .filter(Column::Id.is_in(ids))
                .exec(&conn)
                .await?;
            setup::restore_deleted_ids(db).await?;
        }
        Scenario::GetOrInsert => {
            let existing = Entity::find()
                .filter(Column::Name.eq(GET_OR_INSERT_NAME))
                .one(&conn)
                .await?;
            if existing.is_none() {
                ActiveModel {
                    id: sea_orm::NotSet,
                    name: Set(GET_OR_INSERT_NAME.to_string()),
                    age: Set(20),
                    status: Set(1),
                    city: Set(Some("shanghai".to_string())),
                    created_at: sea_orm::NotSet,
                }
                .insert(&conn)
                .await?;
            }
        }
    }
    // 避免未使用告警（builder 类型在部分版本需要）
    let _ = (
        MysqlQueryBuilder,
        PostgresQueryBuilder,
        Statement::from_string(conn.get_database_backend(), String::new()),
    );
    conn.close().await?;
    Ok(())
}
