//! Welds 后端。

use welds::prelude::*;

use super::{reset_and_seed, reset_empty};
use crate::{CrudOp, DbKind};

#[derive(Debug, Clone, WeldsModel)]
#[welds(table = "t_user")]
pub struct BenchUser {
    #[welds(primary_key)]
    pub id: i64,
    pub name: String,
    pub age: i32,
}

async fn client(
    db: DbKind,
) -> Result<welds::connections::any::AnyClient, Box<dyn std::error::Error + Send + Sync>> {
    let url = db.url().expect("database url");
    Ok(welds::connections::connect(url).await?)
}

pub async fn run(
    db: DbKind,
    op: CrudOp,
    n: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = client(db).await?;
    let client = client.as_ref();
    match op {
        CrudOp::Insert => {
            reset_empty(db).await?;
            for i in 0..n {
                let mut user = BenchUser::new();
                user.name = format!("user_{i}");
                user.age = i as i32;
                user.save(client).await?;
            }
        }
        CrudOp::Update => {
            reset_and_seed(db, n).await?;
            BenchUser::where_col(|u| u.id.gt(0))
                .set(|u| u.age, 99)
                .run(client)
                .await?;
        }
        CrudOp::Delete => {
            reset_and_seed(db, n).await?;
            BenchUser::where_col(|u| u.id.gt(0)).delete(client).await?;
        }
        CrudOp::First => {
            reset_and_seed(db, n).await?;
            let _ = BenchUser::where_col(|u| u.id.gt(0))
                .limit(1)
                .run(client)
                .await?;
        }
        CrudOp::List => {
            reset_and_seed(db, n).await?;
            let _ = BenchUser::where_col(|u| u.id.gt(0))
                .limit(n as i64)
                .run(client)
                .await?;
        }
        CrudOp::Count => {
            reset_and_seed(db, n).await?;
            let _ = BenchUser::where_col(|u| u.id.gt(0)).count(client).await?;
        }
    }
    Ok(())
}
