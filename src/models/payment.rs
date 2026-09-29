//! `payments` table.
//!
//! `txid`, `confirmations` and `external_id` are reserved for the BTCPay
//! Server webhook integration. Nothing writes them while the project is
//! offline-only.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::MySql;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Payment {
    pub id: i64,
    pub invoice_id: i64,
    pub order_id: i64,
    pub method: String,
    pub status: String,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub confirmations: i32,
    pub txid: Option<String>,
    pub external_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn list_by_order<'e, E>(executor: E, order_id: i64) -> Result<Vec<Payment>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, Payment>("SELECT * FROM payments WHERE order_id = ? ORDER BY id ASC")
        .bind(order_id)
        .fetch_all(executor)
        .await
}
