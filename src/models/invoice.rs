//! `invoices` table.
//!
//! Columns for the future BTCPay Server integration are nullable and stay
//! `NULL` until that work lands — no schema change will be required then.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::MySql;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: i64,
    pub order_id: i64,
    pub invoice_number: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
    pub btcpay_invoice_id: Option<String>,
    pub payment_url: Option<String>,
    pub expires_at: Option<NaiveDateTime>,
    pub paid_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn list_by_order<'e, E>(executor: E, order_id: i64) -> Result<Vec<Invoice>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE order_id = ? ORDER BY id ASC",
    )
    .bind(order_id)
    .fetch_all(executor)
    .await
}
