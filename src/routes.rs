use std::sync::{Arc, Mutex};

use chrono::Datelike;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

use crate::db::Database;
use crate::models::{RegularEntry, SingularEntry};

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
