// cashier-app/src-tauri/src/commands.rs
use serde::{Deserialize, Serialize};
use tauri::State;
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use chrono::{Utc, NaiveDateTime};

#[derive(Deserialize)]
pub struct CartItem {
    pub id: i32,
    pub price: BigDecimal,
    pub qty: i32,
}

#[derive(Deserialize)]
pub struct TransactionPayload {
    pub cart: Vec<CartItem>,
    pub payment_method: String,
    pub paid_amount: BigDecimal,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i32,
    pub transaction_code: Option<String>,
    pub total_amount: Option<BigDecimal>,
    pub created_at: Option<NaiveDateTime>,
}

#[tauri::command]
pub async fn save_transaction(
    payload: TransactionPayload,
    pool: State<'_, PgPool>,
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let total = payload
        .cart
        .iter()
        .map(|item| &item.price * BigDecimal::from(item.qty))
        .fold(BigDecimal::from(0), |acc, x| acc + x);

    let today = Utc::now().naive_utc().date();
    let date_str = today.format("%Y%m%d").to_string();
    let count_result = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::INT
        FROM transactions
        WHERE DATE(created_at) = $1
        "#,
        today
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let sequence_number = count_result.unwrap_or(0) + 1;
    let transaction_code = format!("TRX-{}-{:03}", date_str, sequence_number);
    let transaction = sqlx::query!(
        r#"
        INSERT INTO transactions (transaction_code, total_amount, created_at)
        VALUES ($1, $2, NOW())
        RETURNING id
        "#,
        transaction_code,
        total
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    for item in &payload.cart {
        sqlx::query!(
            r#"
            UPDATE products
            SET stock = stock - $1
            WHERE id = $2 AND stock IS NOT NULL
            "#,
            item.qty,
            item.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    sqlx::query!(
        r#"
        INSERT INTO transaction_payments 
        (transaction_id, payment_method, amount)
        VALUES ($1, $2, $3)
        "#,
        transaction.id,
        payload.payment_method,
        &payload.paid_amount
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_today_transactions(pool: State<'_, PgPool>) -> Result<Vec<Transaction>, String> {
    let now = Utc::now().naive_utc();
    let start_of_day = now.date().and_hms_opt(0, 0, 0).unwrap();
    let end_of_day = now.date().and_hms_opt(23, 59, 59).unwrap();

    println!("▶ Fetch trx between {} and {}", start_of_day, end_of_day);

    let transactions = sqlx::query_as!(
        Transaction,
        r#"
        SELECT id, transaction_code, total_amount, created_at
        FROM transactions
        WHERE created_at BETWEEN $1 AND $2
        ORDER BY created_at DESC
        "#,
        start_of_day,
        end_of_day
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| {
        println!("❌ SQL Error: {}", e);
        e.to_string()
    })?;

    println!("✅ Dapat {} transaksi", transactions.len());

    Ok(transactions)
}
