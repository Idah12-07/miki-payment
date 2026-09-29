//! `transactions` table — an append-only ledger of money movements.
//!
//! `amount` is signed: refunds and fees are negative, payments positive.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::MySql;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i64,
    pub payment_id: Option<i64>,
    pub order_id: Option<i64>,
    #[sqlx(rename = "txn_type")]
    #[serde(rename = "type")]
    pub kind: String,
    pub amount: i64,
    pub currency: String,
    pub reference: Option<String>,
    pub created_at: NaiveDateTime,
}

pub async fn list_by_order<'e, E>(
    executor: E,
    order_id: i64,
) -> Result<Vec<Transaction>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE order_id = ? ORDER BY id ASC",
    )
    .bind(order_id)
    .fetch_all(executor)
    .await
}
