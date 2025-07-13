// cashier-app/src-tauri/src/main.rs

use tauri::{command, Manager, State};
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use sqlx::types::BigDecimal;
use dotenvy::dotenv;
use std::env;
mod commands;
use commands::{save_transaction, get_today_transactions};

#[derive(Serialize, sqlx::FromRow)]
struct Product {
    id: i32,
    name: String,
    discount_price: BigDecimal,
    price: Option<BigDecimal>,
    discount: Option<i32>,
    stock: Option<i32>,
    width: Option<i32>,
    depth: Option<i32>,
    height: Option<i32>,
    image: Option<String>,
    category_id: Option<i32>,
    supplier_id: Option<i32>,
    created_at: Option<chrono::NaiveDateTime>,
}

#[command]
async fn get_products(pool: State<'_, PgPool>) -> Result<String, String> {
    let rows = sqlx::query_as!(
        Product,
        r#"
        SELECT
            id, name, discount_price, price, discount, stock,
            width, depth, height, image,
            category_id, supplier_id, created_at
        FROM products
        "#
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    serde_json::to_string(&rows).map_err(|e| e.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    tauri::Builder::default()
        .manage(pool) // simpan pool ke state
        .invoke_handler(tauri::generate_handler![
            get_products,
            save_transaction,
            get_today_transactions
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())?;

    Ok(())
}
