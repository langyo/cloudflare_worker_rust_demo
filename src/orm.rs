use anyhow::{anyhow, Context, Result};
use std::sync::Arc;
use wasm_bindgen::JsValue;

use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, ProxyDatabaseTrait,
    ProxyExecResult, ProxyRow, RuntimeErr, Schema, Statement, Value, Values,
};
use worker::{console_log, Env};

struct ProxyDb {
    env: Arc<Env>,
}

impl std::fmt::Debug for ProxyDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProxyDb").finish()
    }
}

impl ProxyDb {
    async fn do_query(env: Arc<Env>, statement: Statement) -> Result<Vec<ProxyRow>> {
        let sql = statement.sql.clone();
        let values = match statement.values {
            Some(Values(values)) => values
                .iter()
                .map(|val| match &val {
                    Value::BigInt(Some(val)) => JsValue::from(*val),
                    Value::BigUnsigned(Some(val)) => JsValue::from(*val),
                    Value::Int(Some(val)) => JsValue::from(*val),
                    Value::Unsigned(Some(val)) => JsValue::from(*val),
                    Value::SmallInt(Some(val)) => JsValue::from(*val),
                    Value::SmallUnsigned(Some(val)) => JsValue::from(*val),
                    Value::TinyInt(Some(val)) => JsValue::from(*val),
                    Value::TinyUnsigned(Some(val)) => JsValue::from(*val),

                    Value::Float(Some(val)) => JsValue::from_f64(*val as f64),
                    Value::Double(Some(val)) => JsValue::from_f64(*val),

                    Value::Bool(Some(val)) => JsValue::from(*val),
                    Value::Bytes(Some(val)) => JsValue::from(format!(
                        "X'{}'",
                        val.iter()
                            .map(|byte| format!("{:02x}", byte))
                            .collect::<String>()
                    )),
                    Value::Char(Some(val)) => JsValue::from(val.to_string()),
                    Value::Json(Some(val)) => JsValue::from(val.to_string()),
                    Value::String(Some(val)) => JsValue::from(val.to_string()),

                    Value::ChronoDate(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTime(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTimeLocal(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTimeUtc(Some(val)) => JsValue::from(val.to_string()),
                    Value::ChronoDateTimeWithTimeZone(Some(val)) => JsValue::from(val.to_string()),

                    _ => JsValue::NULL,
                })
                .collect(),
            None => Vec::new(),
        };

        let ret = env.d1("ljyys")?.prepare(sql).bind(&values)?.all().await?;
        if let Some(message) = ret.error() {
            return Err(anyhow!(message.to_string()));
        }
        let ret = ret.results::<serde_json::Value>()?;
        for (index, item) in ret.iter().enumerate() {
            console_log!("#{}: {}", index, item.to_string());
        }

        // TODO: 暂时先不返回，我瞅一眼结果长啥样

        Ok(vec![])
    }
}

#[async_trait::async_trait]
impl ProxyDatabaseTrait for ProxyDb {
    async fn query(&self, statement: Statement) -> Result<Vec<ProxyRow>, DbErr> {
        console_log!("SQL query: {:?}", statement);

        let env = self.env.clone();
        let (tx, rx) = oneshot::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let ret = Self::do_query(env, statement).await;
            tx.send(ret).unwrap();
        });

        let ret = rx.await.unwrap();
        ret.map_err(|err| DbErr::Conn(RuntimeErr::Internal(err.to_string())))
    }

    async fn execute(&self, statement: Statement) -> Result<ProxyExecResult, DbErr> {
        console_log!("SQL execute: {:?}", statement);
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
    let db = Database::connect_proxy(DbBackend::Sqlite, Arc::new(Box::new(ProxyDb { env })))
        .await
        .context("Failed to connect to database")?;
    let builder = db.get_database_backend();

    console_log!("Connected to database");

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
