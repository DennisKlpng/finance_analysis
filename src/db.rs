use crate::models::*;
use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS singular_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                amount REAL NOT NULL,
                description TEXT NOT NULL,
                date TEXT NOT NULL,
                type_category TEXT NOT NULL,
                necessity_category TEXT NOT NULL,
                is_income INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS regular_entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                amount REAL NOT NULL,
                description TEXT NOT NULL,
                periodicity TEXT NOT NULL,
                start_date TEXT NOT NULL,
                end_date TEXT NOT NULL,
                type_category TEXT NOT NULL,
                necessity_category TEXT NOT NULL,
                is_income INTEGER NOT NULL
            );
        ",
        )?;
        Ok(())
    }

    // ─── Singular entries ────────────────────────────────────────────────────

    pub fn add_singular(&self, entry: &SingularEntry) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO singular_entries
             (amount, description, date, type_category, necessity_category, is_income)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                entry.amount,
                entry.description,
                entry.date.to_string(),
                entry.type_category.as_str(),
                entry.necessity_category.as_str(),
                entry.is_income as i32
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_singular(&self, id: i64) -> Result<Option<SingularEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, description, date, type_category, necessity_category, is_income
             FROM singular_entries WHERE id = ?1",
        )?;
        let result = stmt.query_row(params![id], row_to_singular);
        match result {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_all_singular(&self) -> Result<Vec<SingularEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, description, date, type_category, necessity_category, is_income
             FROM singular_entries ORDER BY date DESC",
        )?;
        let rows = stmt.query_map([], row_to_singular)?;
        rows.collect()
    }

    pub fn update_singular(&self, entry: &SingularEntry) -> Result<()> {
        self.conn.execute(
            "UPDATE singular_entries
             SET amount=?1, description=?2, date=?3,
                 type_category=?4, necessity_category=?5, is_income=?6
             WHERE id=?7",
            params![
                entry.amount,
                entry.description,
                entry.date.to_string(),
                entry.type_category.as_str(),
                entry.necessity_category.as_str(),
                entry.is_income as i32,
                entry.id.unwrap()
            ],
        )?;
        Ok(())
    }

    pub fn delete_singular(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM singular_entries WHERE id=?1", params![id])?;
        Ok(())
    }

    // ─── Regular entries ─────────────────────────────────────────────────────

    pub fn add_regular(&self, entry: &RegularEntry) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO regular_entries
             (amount, description, periodicity, start_date, end_date,
              type_category, necessity_category, is_income)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.amount,
                entry.description,
                entry.periodicity.as_str(),
                entry.start_date.to_string(),
                entry.end_date.to_string(),
                entry.type_category.as_str(),
                entry.necessity_category.as_str(),
                entry.is_income as i32
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_regular(&self, id: i64) -> Result<Option<RegularEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, description, periodicity, start_date, end_date,
                    type_category, necessity_category, is_income
             FROM regular_entries WHERE id=?1",
        )?;
        let result = stmt.query_row(params![id], row_to_regular);
        match result {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn get_all_regular(&self) -> Result<Vec<RegularEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, amount, description, periodicity, start_date, end_date,
                    type_category, necessity_category, is_income
             FROM regular_entries ORDER BY start_date DESC",
        )?;
        let rows = stmt.query_map([], row_to_regular)?;
        rows.collect()
    }

    pub fn update_regular(&self, entry: &RegularEntry) -> Result<()> {
        self.conn.execute(
            "UPDATE regular_entries
             SET amount=?1, description=?2, periodicity=?3,
                 start_date=?4, end_date=?5,
                 type_category=?6, necessity_category=?7, is_income=?8
             WHERE id=?9",
            params![
                entry.amount,
                entry.description,
                entry.periodicity.as_str(),
                entry.start_date.to_string(),
                entry.end_date.to_string(),
                entry.type_category.as_str(),
                entry.necessity_category.as_str(),
                entry.is_income as i32,
                entry.id.unwrap()
            ],
        )?;
        Ok(())
    }

    pub fn delete_regular(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM regular_entries WHERE id=?1", params![id])?;
        Ok(())
    }

    // ─── Monthly summary ─────────────────────────────────────────────────────

    /// Calculate income and expenses for a given year/month.
    ///
    /// * Singular entries are booked to their exact date's month.
    /// * Monthly regular entries contribute their full amount to every month
    ///   within [start_date, end_date].
    /// * Yearly regular entries contribute amount/12 to every month within
    ///   [start_date, end_date].
    pub fn get_month_summary(&self, year: i32, month: u32) -> Result<MonthSummary> {
        let month_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next_month_start = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };
        let month_end = next_month_start.pred_opt().unwrap();

        // Singular entries in this month
        let mut stmt = self.conn.prepare(
            "SELECT amount, is_income FROM singular_entries
             WHERE date >= ?1 AND date <= ?2",
        )?;
        let (mut singular_income, mut singular_expenses) = (0.0f64, 0.0f64);
        let rows = stmt.query_map(
            params![month_start.to_string(), month_end.to_string()],
            |row| {
                let amount: f64 = row.get(0)?;
                let is_income: i32 = row.get(1)?;
                Ok((amount, is_income != 0))
            },
        )?;
        for row in rows {
            let (amount, is_income) = row?;
            if is_income {
                singular_income += amount;
            } else {
                singular_expenses += amount;
            }
        }

        // Regular entries active during this month
        let mut stmt = self.conn.prepare(
            "SELECT amount, periodicity, is_income FROM regular_entries
             WHERE start_date <= ?2 AND end_date >= ?1",
        )?;
        let (mut regular_income, mut regular_expenses) = (0.0f64, 0.0f64);
        let rows = stmt.query_map(
            params![month_start.to_string(), month_end.to_string()],
            |row| {
                let amount: f64 = row.get(0)?;
                let periodicity: String = row.get(1)?;
                let is_income: i32 = row.get(2)?;
                Ok((amount, periodicity, is_income != 0))
            },
        )?;
        for row in rows {
            let (amount, periodicity, is_income) = row?;
            let monthly_amount = if periodicity == "yearly" {
                amount / 12.0
            } else {
                amount
            };
            if is_income {
                regular_income += monthly_amount;
            } else {
                regular_expenses += monthly_amount;
            }
        }

        let total_income = singular_income + regular_income;
        let total_expenses = singular_expenses + regular_expenses;
        Ok(MonthSummary {
            year,
            month,
            singular_income,
            singular_expenses,
            regular_income,
            regular_expenses,
            total_income,
            total_expenses,
            net: total_income - total_expenses,
        })
    }
}

// ─── Row mapping helpers ──────────────────────────────────────────────────────

fn row_to_singular(row: &rusqlite::Row) -> rusqlite::Result<SingularEntry> {
    let date_str: String = row.get(3)?;
    let type_cat: String = row.get(4)?;
    let nec_cat: String = row.get(5)?;
    let is_income: i32 = row.get(6)?;
    Ok(SingularEntry {
        id: Some(row.get(0)?),
        amount: row.get(1)?,
        description: row.get(2)?,
        date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .expect("Invalid date in DB"),
        type_category: TypeCategory::from_str(&type_cat).expect("Invalid type_category in DB"),
        necessity_category: NecessityCategory::from_str(&nec_cat)
            .expect("Invalid necessity_category in DB"),
        is_income: is_income != 0,
    })
}

fn row_to_regular(row: &rusqlite::Row) -> rusqlite::Result<RegularEntry> {
    let periodicity_str: String = row.get(3)?;
    let start_str: String = row.get(4)?;
    let end_str: String = row.get(5)?;
    let type_cat: String = row.get(6)?;
    let nec_cat: String = row.get(7)?;
    let is_income: i32 = row.get(8)?;
    Ok(RegularEntry {
        id: Some(row.get(0)?),
        amount: row.get(1)?,
        description: row.get(2)?,
        periodicity: Periodicity::from_str(&periodicity_str)
            .expect("Invalid periodicity in DB"),
        start_date: NaiveDate::parse_from_str(&start_str, "%Y-%m-%d")
            .expect("Invalid start_date in DB"),
        end_date: NaiveDate::parse_from_str(&end_str, "%Y-%m-%d")
            .expect("Invalid end_date in DB"),
        type_category: TypeCategory::from_str(&type_cat).expect("Invalid type_category in DB"),
        necessity_category: NecessityCategory::from_str(&nec_cat)
            .expect("Invalid necessity_category in DB"),
        is_income: is_income != 0,
    })
}
