use std::sync::{Arc, Mutex};

use chrono::Datelike;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

use finance_analysis::db::Database;
use finance_analysis::models::{RegularEntry, SingularEntry};

pub type AppState = Arc<Mutex<Database>>;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn db_err(e: rusqlite::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": e.to_string() })),
    )
        .into_response()
}

fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": "not found" })),
    )
        .into_response()
}

// ─── Root ─────────────────────────────────────────────────────────────────────

pub async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}

// ─── Singular entries ─────────────────────────────────────────────────────────

pub async fn list_singular(State(db): State<AppState>) -> Response {
    let db = db.lock().unwrap();
    match db.get_all_singular() {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn create_singular(
    State(db): State<AppState>,
    Json(entry): Json<SingularEntry>,
) -> Response {
    let db = db.lock().unwrap();
    match db.add_singular(&entry) {
        Ok(id) => (StatusCode::CREATED, Json(json!({ "id": id }))).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn get_singular(State(db): State<AppState>, Path(id): Path<i64>) -> Response {
    let db = db.lock().unwrap();
    match db.get_singular(id) {
        Ok(Some(entry)) => Json(entry).into_response(),
        Ok(None) => not_found(),
        Err(e) => db_err(e),
    }
}

pub async fn update_singular(
    State(db): State<AppState>,
    Path(id): Path<i64>,
    Json(mut entry): Json<SingularEntry>,
) -> Response {
    entry.id = Some(id);
    let db = db.lock().unwrap();
    match db.update_singular(&entry) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn delete_singular(State(db): State<AppState>, Path(id): Path<i64>) -> Response {
    let db = db.lock().unwrap();
    match db.delete_singular(id) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}

// ─── Regular entries ──────────────────────────────────────────────────────────

pub async fn list_regular(State(db): State<AppState>) -> Response {
    let db = db.lock().unwrap();
    match db.get_all_regular() {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn create_regular(
    State(db): State<AppState>,
    Json(entry): Json<RegularEntry>,
) -> Response {
    let db = db.lock().unwrap();
    match db.add_regular(&entry) {
        Ok(id) => (StatusCode::CREATED, Json(json!({ "id": id }))).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn get_regular_entry(State(db): State<AppState>, Path(id): Path<i64>) -> Response {
    let db = db.lock().unwrap();
    match db.get_regular(id) {
        Ok(Some(entry)) => Json(entry).into_response(),
        Ok(None) => not_found(),
        Err(e) => db_err(e),
    }
}

pub async fn update_regular(
    State(db): State<AppState>,
    Path(id): Path<i64>,
    Json(mut entry): Json<RegularEntry>,
) -> Response {
    entry.id = Some(id);
    let db = db.lock().unwrap();
    match db.update_regular(&entry) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn delete_regular(State(db): State<AppState>, Path(id): Path<i64>) -> Response {
    let db = db.lock().unwrap();
    match db.delete_regular(id) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}

// ─── Monthly summary ──────────────────────────────────────────────────────────

pub async fn month_summary(
    State(db): State<AppState>,
    Path((year, month)): Path<(i32, u32)>,
) -> Response {
    if month < 1 || month > 12 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "month must be 1-12" })),
        )
            .into_response();
    }
    let db = db.lock().unwrap();
    match db.get_month_summary(year, month) {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => db_err(e),
    }
}

// ─── Available months (for navigation) ───────────────────────────────────────

pub async fn available_months(State(db): State<AppState>) -> Response {
    let db = db.lock().unwrap();

    // Collect distinct year-month combinations from both tables
    let singular = db.get_all_singular().unwrap_or_default();
    let regular = db.get_all_regular().unwrap_or_default();

    let mut months: std::collections::BTreeSet<(i32, u32)> = std::collections::BTreeSet::new();

    for e in &singular {
        months.insert((e.date.year(), e.date.month()));
    }
    for e in &regular {
        // Include every month in the validity range
        let mut current = e.start_date.with_day(1).unwrap();
        let end = e.end_date.with_day(1).unwrap();
        while current <= end {
            months.insert((current.year(), current.month()));
            // Advance one month
            let next_month = if current.month() == 12 {
                chrono::NaiveDate::from_ymd_opt(current.year() + 1, 1, 1).unwrap()
            } else {
                chrono::NaiveDate::from_ymd_opt(current.year(), current.month() + 1, 1).unwrap()
            };
            current = next_month;
        }
    }

    // Always include current month
    let today = chrono::Local::now().date_naive();
    months.insert((today.year(), today.month()));

    let result: Vec<Value> = months
        .into_iter()
        .map(|(y, m)| json!({ "year": y, "month": m }))
        .collect();

    Json(result).into_response()
}

// ─── Yearly summary ───────────────────────────────────────────────────────────

pub async fn year_summary(State(db): State<AppState>, Path(year): Path<i32>) -> Response {
    let db = db.lock().unwrap();
    match db.get_year_summary(year) {
        Ok(summaries) => Json(summaries).into_response(),
        Err(e) => db_err(e),
    }
}

// ─── Expense distribution ─────────────────────────────────────────────────────

pub async fn expense_distribution(
    State(db): State<AppState>,
    Path((year, month)): Path<(i32, u32)>,
) -> Response {
    if month < 1 || month > 12 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "month must be 1-12" })),
        )
            .into_response();
    }
    let db = db.lock().unwrap();
    match db.get_expense_distribution(year, month) {
        Ok((type_dist, necessity_dist)) => Json(json!({
            "type_distribution": type_dist,
            "necessity_distribution": necessity_dist
        }))
        .into_response(),
        Err(e) => db_err(e),
    }
}

// ─── Excel Import ─────────────────────────────────────────────────────────────

pub async fn import_excel(
    State(db): State<AppState>,
    mut multipart: Multipart,
) -> Response {
    // Extract file and year from multipart form
    let mut file_data: Option<Vec<u8>> = None;
    let mut year: Option<i32> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let data = field.bytes().await.unwrap_or_default();
            file_data = Some(data.to_vec());
        } else if name == "year" {
            let text = field.text().await.unwrap_or_default();
            year = text.parse().ok();
        }
    }

    let file_data = match file_data {
        Some(d) => d,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "No file uploaded" })),
            )
                .into_response();
        }
    };

    let year = match year {
        Some(y) => y,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Year parameter missing" })),
            )
                .into_response();
        }
    };

    // Save temp file
    let temp_path = std::env::temp_dir().join(format!("finance_import_{}.xlsx", year));
    if let Err(e) = std::fs::write(&temp_path, &file_data) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to save temp file: {}", e) })),
        )
            .into_response();
    }

    // Import
    let db_guard = db.lock().unwrap();
    let result = finance_analysis::import::import_excel(&temp_path, "excel_mapping.json", &db_guard, year);
    drop(db_guard);

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    match result {
        Ok(stats) => Json(json!({
            "success": true,
            "regular_expenses": stats.regular_expenses,
            "singular_expenses": stats.singular_expenses,
            "regular_incomes": stats.regular_incomes,
            "singular_incomes": stats.singular_incomes,
            "errors": stats.errors
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// ─── Wealth Snapshots ─────────────────────────────────────────────────────────

pub async fn list_wealth_snapshots(State(db): State<AppState>) -> Response {
    let db = db.lock().unwrap();
    match db.get_all_wealth_snapshots() {
        Ok(snapshots) => Json(snapshots).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn create_wealth_snapshot(
    State(db): State<AppState>,
    Json(snapshot): Json<finance_analysis::models::WealthSnapshot>,
) -> Response {
    let db = db.lock().unwrap();
    match db.add_wealth_snapshot(&snapshot) {
        Ok(id) => (StatusCode::CREATED, Json(json!({ "id": id }))).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn get_wealth_snapshot_by_date(
    State(db): State<AppState>,
    Path(date_str): Path<String>,
) -> Response {
    let date = match chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid date format, use YYYY-MM-DD" })),
            )
                .into_response()
        }
    };

    let db = db.lock().unwrap();
    match db.get_wealth_snapshot(&date) {
        Ok(Some(snapshot)) => Json(snapshot).into_response(),
        Ok(None) => not_found(),
        Err(e) => db_err(e),
    }
}

pub async fn update_wealth_snapshot(
    State(db): State<AppState>,
    Path(date_str): Path<String>,
    Json(mut snapshot): Json<finance_analysis::models::WealthSnapshot>,
) -> Response {
    let date = match chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid date format, use YYYY-MM-DD" })),
            )
                .into_response()
        }
    };

    // Get existing snapshot to obtain ID
    let db = db.lock().unwrap();
    let existing = match db.get_wealth_snapshot(&date) {
        Ok(Some(s)) => s,
        Ok(None) => return not_found(),
        Err(e) => return db_err(e),
    };

    snapshot.id = existing.id;
    snapshot.date = date;

    match db.update_wealth_snapshot(&snapshot) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn delete_wealth_snapshot(
    State(db): State<AppState>,
    Path(date_str): Path<String>,
) -> Response {
    let date = match chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
        Ok(d) => d,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid date format, use YYYY-MM-DD" })),
            )
                .into_response()
        }
    };

    let db = db.lock().unwrap();
    match db.delete_wealth_snapshot(&date) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}

// ─── Fixed Salary ─────────────────────────────────────────────────────────────

pub async fn list_fixed_salaries(State(db): State<AppState>) -> Response {
    let db = db.lock().unwrap();
    match db.get_all_fixed_salaries() {
        Ok(salaries) => Json(salaries).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn create_fixed_salary(
    State(db): State<AppState>,
    Json(salary): Json<finance_analysis::models::FixedSalary>,
) -> Response {
    let db = db.lock().unwrap();
    match db.add_fixed_salary(&salary) {
        Ok(id) => (StatusCode::CREATED, Json(json!({ "id": id }))).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn get_fixed_salary(State(db): State<AppState>, Path(id): Path<i64>) -> Response {
    let db = db.lock().unwrap();
    match db.get_fixed_salary(id) {
        Ok(Some(salary)) => Json(salary).into_response(),
        Ok(None) => not_found(),
        Err(e) => db_err(e),
    }
}

pub async fn update_fixed_salary(
    State(db): State<AppState>,
    Path(id): Path<i64>,
    Json(mut salary): Json<finance_analysis::models::FixedSalary>,
) -> Response {
    salary.id = Some(id);
    let db = db.lock().unwrap();
    match db.update_fixed_salary(&salary) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn delete_fixed_salary(State(db): State<AppState>, Path(id): Path<i64>) -> Response {
    let db = db.lock().unwrap();
    match db.delete_fixed_salary(id) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}

// ─── Variable Salary ──────────────────────────────────────────────────────────

pub async fn list_variable_salaries(State(db): State<AppState>) -> Response {
    let db = db.lock().unwrap();
    match db.get_all_variable_salaries() {
        Ok(salaries) => Json(salaries).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn create_variable_salary(
    State(db): State<AppState>,
    Json(salary): Json<finance_analysis::models::VariableSalary>,
) -> Response {
    let db = db.lock().unwrap();
    match db.add_variable_salary(&salary) {
        Ok(id) => (StatusCode::CREATED, Json(json!({ "id": id }))).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn get_variable_salary(State(db): State<AppState>, Path(id): Path<i64>) -> Response {
    let db = db.lock().unwrap();
    match db.get_variable_salary(id) {
        Ok(Some(salary)) => Json(salary).into_response(),
        Ok(None) => not_found(),
        Err(e) => db_err(e),
    }
}

pub async fn update_variable_salary(
    State(db): State<AppState>,
    Path(id): Path<i64>,
    Json(mut salary): Json<finance_analysis::models::VariableSalary>,
) -> Response {
    salary.id = Some(id);
    let db = db.lock().unwrap();
    match db.update_variable_salary(&salary) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}

pub async fn delete_variable_salary(State(db): State<AppState>, Path(id): Path<i64>) -> Response {
    let db = db.lock().unwrap();
    match db.delete_variable_salary(id) {
        Ok(_) => Json(json!({ "ok": true })).into_response(),
        Err(e) => db_err(e),
    }
}
