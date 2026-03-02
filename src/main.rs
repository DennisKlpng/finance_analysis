mod routes;

use std::sync::{Arc, Mutex};

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use finance_analysis::db::Database;
use routes::*;

const DB_PATH: &str = "Z:/Finanzen/finance.db";
const ADDR: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let database = Database::new(DB_PATH)?;
    let state: AppState = Arc::new(Mutex::new(database));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // Serve the SPA
        .route("/", get(serve_index))
        // Singular entries
        .route("/api/singular", get(list_singular).post(create_singular))
        .route(
            "/api/singular/:id",
            get(get_singular).put(update_singular).delete(delete_singular),
        )
        // Regular entries
        .route("/api/regular", get(list_regular).post(create_regular))
        .route(
            "/api/regular/:id",
            get(get_regular_entry).put(update_regular).delete(delete_regular),
        )
        // Monthly summary
        .route("/api/month/:year/:month", get(month_summary))
        // Available months for navigation
        .route("/api/months", get(available_months))
        // Yearly summary
        .route("/api/year/:year", get(year_summary))
        // Expense distribution
        .route("/api/expenses/distribution/:year/:month", get(expense_distribution))
        // Excel import
        .route("/api/import/excel", post(import_excel))
        // Wealth snapshots
        .route("/api/wealth", get(list_wealth_snapshots).post(create_wealth_snapshot))
        .route(
            "/api/wealth/:date",
            get(get_wealth_snapshot_by_date)
                .put(update_wealth_snapshot)
                .delete(delete_wealth_snapshot),
        )
        // Fixed salary
        .route("/api/salary/fixed", get(list_fixed_salaries).post(create_fixed_salary))
        .route(
            "/api/salary/fixed/:id",
            get(get_fixed_salary)
                .put(update_fixed_salary)
                .delete(delete_fixed_salary),
        )
        // Variable salary
        .route("/api/salary/variable", get(list_variable_salaries).post(create_variable_salary))
        .route(
            "/api/salary/variable/:id",
            get(get_variable_salary)
                .put(update_variable_salary)
                .delete(delete_variable_salary),
        )
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(ADDR).await?;
    info!("Finance Analysis running at http://{ADDR}");
    println!("Finance Analysis server started.");
    println!("Open http://{ADDR} in your browser.");
    println!("Database: {DB_PATH}");
    println!("Press Ctrl+C to stop.");

    axum::serve(listener, app).await?;
    Ok(())
}
