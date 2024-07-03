use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};

use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, ProxyDatabaseTrait,
    ProxyExecResult, ProxyRow, Schema, Statement,
};
use worker::Env;

struct ProxyDb {
    env: Arc<Env>,
}

impl std::fmt::Debug for ProxyDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyDb").finish()
    }
}

impl ProxyDatabaseTrait for ProxyDb {
    fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        println!("SQL query: {:?}", statement);
        let sql = statement.sql.clone();

        let mut ret: Vec<ProxyRow> = vec![];

        // TODO: 直接与 self.env 交互
        // 可能根据需要得带上 wasm_bindgen_futures + oneshot 以规避传递的堆对象没实现 Send + Sync 导致无法交换的问题

        Ok(ret)
    }

    fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        println!("SQL execute: {:?}", statement);
        let sql = statement.sql.clone();

        // TODO: 直接与 self.env 交互
        // 可能根据需要得带上 wasm_bindgen_futures + oneshot 以规避传递的堆对象没实现 Send + Sync 导致无法交换的问题

        Ok(ProxyExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        })
    }
}

pub async fn init_db(env: Arc<Env>) -> Result<DatabaseConnection> {
    let db = Database::connect_proxy(
        DbBackend::Sqlite,
        Arc::new(Mutex::new(Box::new(ProxyDb { env }))),
    )
    .await
    .context("Failed to connect to database")?;
    let builder = db.get_database_backend();

    db.execute(
        builder.build(
            Schema::new(builder)
                .create_table_from_entity(crate::entity::Entity)
                .if_not_exists(),
        ),
    )
    .await?;

    Ok(db)
}
