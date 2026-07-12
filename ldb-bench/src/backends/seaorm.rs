//! SeaORM 后端。

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, QuerySelect, Set,
};

use super::{reset_and_seed, reset_empty};
use crate::{CrudOp, DbKind};

mod entity {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "t_user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i64,
        pub name: Option<String>,
        pub age: Option<i32>,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

use entity::{ActiveModel, Column, Entity};

async fn connect(db: DbKind) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let url = db.url().expect("database url");
    Database::connect(url).await
}

pub async fn run(
    db: DbKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = connect(db).await?;
    match op {
        CrudOp::Insert => {
            reset_empty(db).await?;
            for i in 0..n {
                let model = ActiveModel {
                    id: sea_orm::NotSet,
                    name: Set(Some(format!("user_{i}"))),
                    age: Set(Some(i as i32)),
                };
                model.insert(&conn).await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(db, n).await?;
            Entity::update_many()
                .col_expr(Column::Age, sea_orm::sea_query::Expr::value(99))
                .filter(Column::Id.gt(0))
                .exec(&conn)
                .await?;
        }
        CrudOp::Delete => {
            reset_and_seed(db, n).await?;
            Entity::delete_many()
                .filter(Column::Id.gt(0))
                .exec(&conn)
                .await?;
        }
        CrudOp::First => {
            reset_and_seed(db, n).await?;
            let _ = Entity::find().filter(Column::Id.gt(0)).one(&conn).await?;
        }
        CrudOp::List => {
            reset_and_seed(db, n).await?;
            let _ = Entity::find()
                .filter(Column::Id.gt(0))
                .limit(n as u64)
                .all(&conn)
                .await?;
        }
        CrudOp::Count => {
            reset_and_seed(db, n).await?;
            let _ = Entity::find().filter(Column::Id.gt(0)).count(&conn).await?;
        }
    }
    conn.close().await?;
    Ok(())
}
