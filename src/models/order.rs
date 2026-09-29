//! `orders` table plus the order detail view used by the API.

use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{MySql, MySqlPool};
use uuid::Uuid;

use crate::error::ApiError;

use super::{invoice, payment, transaction, user};

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Order {
    pub id: i64,
    pub order_number: String,
    pub user_id: i64,
    pub amount: i64,
    pub currency: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Validated input for [`create`].
#[derive(Debug, Clone)]
pub struct NewOrder {
    pub email: String,
    pub name: Option<String>,
    pub amount: i64,
    pub currency: String,
    pub description: Option<String>,
}

/// Everything the API returns for a single order.
#[derive(Debug, Serialize)]
pub struct OrderDetail {
    pub order: Order,
    pub user: user::User,
    pub invoices: Vec<invoice::Invoice>,
    pub payments: Vec<payment::Payment>,
    pub transactions: Vec<transaction::Transaction>,
}

fn new_order_number() -> String {
    format!("ORD-{}", Uuid::new_v4().simple())
}

/// Create an order, upserting its user, inside a single transaction so an
/// order can never exist without its user.
pub async fn create(pool: &MySqlPool, input: NewOrder) -> Result<Order, ApiError> {
    let mut tx = pool.begin().await?;

    let user = user::find_or_create(&mut tx, &input.email, input.name.as_deref()).await?;

    let result = sqlx::query(
        "INSERT INTO orders (order_number, user_id, amount, currency, description)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(new_order_number())
    .bind(user.id)
    .bind(input.amount)
    .bind(&input.currency)
    .bind(&input.description)
    .execute(&mut *tx)
    .await?;

    let id = result.last_insert_id() as i64;

    let order = find_by_id(&mut *tx, id)
        .await?
        .ok_or_else(|| ApiError::Internal(format!("order {id} vanished after insert")))?;

    tx.commit().await?;

    Ok(order)
}

pub async fn find_by_id<'e, E>(executor: E, id: i64) -> Result<Option<Order>, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, Order>("SELECT * FROM orders WHERE id = ?")
        .bind(id)
        .fetch_optional(executor)
        .await
}

/// Fetch an order together with its user and all related records.
pub async fn find_detail(pool: &MySqlPool, id: i64) -> Result<OrderDetail, ApiError> {
    let order = find_by_id(pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("order {id} not found")))?;

    let user = user::find_by_id(pool, order.user_id)
        .await?
        .ok_or_else(|| {
            ApiError::Internal(format!("order {id} references missing user {}", order.user_id))
        })?;

    let invoices = invoice::list_by_order(pool, id).await?;
    let payments = payment::list_by_order(pool, id).await?;
    let transactions = transaction::list_by_order(pool, id).await?;

    Ok(OrderDetail {
        order,
        user,
        invoices,
        payments,
        transactions,
    })
}
