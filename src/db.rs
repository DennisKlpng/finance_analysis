use crate::models::*;
use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let is_new = std::fs::metadata(path)
            .map(|m| m.len() == 0)
            .unwrap_or(true); // file doesn't exist → treat as new

        let conn = Connection::open(path)?;
        let db = Database { conn };
        if is_new {
            db.initialize()?;
        }
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

            CREATE TABLE IF NOT EXISTS wealth_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS wealth_components (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                snapshot_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                amount REAL NOT NULL,
                FOREIGN KEY (snapshot_id) REFERENCES wealth_snapshots(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS fixed_salary (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                effective_date TEXT NOT NULL,
                monthly_amount REAL NOT NULL,
                payments_per_year INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS variable_salary (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                amount REAL NOT NULL,
                description TEXT NOT NULL
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

    /// Get summary for all 12 months of a given year
    pub fn get_year_summary(&self, year: i32) -> Result<Vec<MonthSummary>> {
        let mut summaries = Vec::new();
        for month in 1..=12 {
            summaries.push(self.get_month_summary(year, month)?);
        }
        Ok(summaries)
    }

    /// Get expense distribution by category for a given month
    pub fn get_expense_distribution(&self, year: i32, month: u32) -> Result<(Vec<(String, f64)>, Vec<(String, f64)>)> {
        let month_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
        let next_month_start = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };
        let month_end = next_month_start.pred_opt().unwrap();

        // Type distribution from singular expenses
        let mut type_dist: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        let mut necessity_dist: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

        // Singular expenses
        let mut stmt = self.conn.prepare(
            "SELECT amount, type_category, necessity_category FROM singular_entries
             WHERE date >= ?1 AND date <= ?2 AND is_income = 0",
        )?;
        let rows = stmt.query_map(
            params![month_start.to_string(), month_end.to_string()],
            |row| {
                let amount: f64 = row.get(0)?;
                let type_cat: String = row.get(1)?;
                let nec_cat: String = row.get(2)?;
                Ok((amount, type_cat, nec_cat))
            },
        )?;
        for row in rows {
            let (amount, type_cat, nec_cat) = row?;
            *type_dist.entry(type_cat).or_insert(0.0) += amount;
            *necessity_dist.entry(nec_cat).or_insert(0.0) += amount;
        }

        // Regular expenses
        let mut stmt = self.conn.prepare(
            "SELECT amount, periodicity, type_category, necessity_category FROM regular_entries
             WHERE start_date <= ?2 AND end_date >= ?1 AND is_income = 0",
        )?;
        let rows = stmt.query_map(
            params![month_start.to_string(), month_end.to_string()],
            |row| {
                let amount: f64 = row.get(0)?;
                let periodicity: String = row.get(1)?;
                let type_cat: String = row.get(2)?;
                let nec_cat: String = row.get(3)?;
                Ok((amount, periodicity, type_cat, nec_cat))
            },
        )?;
        for row in rows {
            let (amount, periodicity, type_cat, nec_cat) = row?;
            let monthly_amount = if periodicity == "yearly" {
                amount / 12.0
            } else {
                amount
            };
            *type_dist.entry(type_cat).or_insert(0.0) += monthly_amount;
            *necessity_dist.entry(nec_cat).or_insert(0.0) += monthly_amount;
        }

        let type_vec: Vec<(String, f64)> = type_dist.into_iter().collect();
        let necessity_vec: Vec<(String, f64)> = necessity_dist.into_iter().collect();

        Ok((type_vec, necessity_vec))
    }

    // ─── Wealth Snapshots ─────────────────────────────────────────────────────

    pub fn add_wealth_snapshot(&self, snapshot: &WealthSnapshot) -> Result<i64> {
        // Insert snapshot
        self.conn.execute(
            "INSERT INTO wealth_snapshots (date) VALUES (?1)",
            params![snapshot.date.to_string()],
        )?;
        let snapshot_id = self.conn.last_insert_rowid();

        // Insert components
        for component in &snapshot.components {
            self.conn.execute(
                "INSERT INTO wealth_components (snapshot_id, name, amount) VALUES (?1, ?2, ?3)",
                params![snapshot_id, component.name, component.amount],
            )?;
        }

        Ok(snapshot_id)
    }

    pub fn get_all_wealth_snapshots(&self) -> Result<Vec<WealthSnapshot>> {
        let mut snapshots = Vec::new();
        let mut stmt = self.conn.prepare(
            "SELECT id, date FROM wealth_snapshots ORDER BY date ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        for row in rows {
            let (id, date_str) = row?;
            let components = self.get_wealth_components(id)?;
            let total: f64 = components.iter().map(|c| c.amount).sum();
            snapshots.push(WealthSnapshot {
                id: Some(id),
                date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .expect("Invalid date in DB"),
                components,
                total,
            });
        }

        Ok(snapshots)
    }

    pub fn get_wealth_snapshot(&self, date: &NaiveDate) -> Result<Option<WealthSnapshot>> {
        let mut stmt = self.conn.prepare(
            "SELECT id FROM wealth_snapshots WHERE date = ?1"
        )?;
        let id: Option<i64> = stmt.query_row(params![date.to_string()], |row| row.get(0)).ok();

        match id {
            Some(id) => {
                let components = self.get_wealth_components(id)?;
                let total: f64 = components.iter().map(|c| c.amount).sum();
                Ok(Some(WealthSnapshot {
                    id: Some(id),
                    date: *date,
                    components,
                    total,
                }))
            }
            None => Ok(None),
        }
    }

    fn get_wealth_components(&self, snapshot_id: i64) -> Result<Vec<WealthComponent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, amount FROM wealth_components WHERE snapshot_id = ?1"
        )?;
        let rows = stmt.query_map(params![snapshot_id], |row| {
            Ok(WealthComponent {
                id: Some(row.get(0)?),
                snapshot_id: Some(snapshot_id),
                name: row.get(1)?,
                amount: row.get(2)?,
            })
        })?;

        rows.collect()
    }

    pub fn update_wealth_snapshot(&self, snapshot: &WealthSnapshot) -> Result<()> {
        let id = snapshot.id.expect("Snapshot must have ID for update");

        // Delete existing components
        self.conn.execute(
            "DELETE FROM wealth_components WHERE snapshot_id = ?1",
            params![id],
        )?;

        // Insert new components
        for component in &snapshot.components {
            self.conn.execute(
                "INSERT INTO wealth_components (snapshot_id, name, amount) VALUES (?1, ?2, ?3)",
                params![id, component.name, component.amount],
            )?;
        }

        Ok(())
    }

    pub fn delete_wealth_snapshot(&self, date: &NaiveDate) -> Result<()> {
        self.conn.execute(
            "DELETE FROM wealth_snapshots WHERE date = ?1",
            params![date.to_string()],
        )?;
        Ok(())
    }

    // ─── Fixed Salary ─────────────────────────────────────────────────────────

    pub fn add_fixed_salary(&self, salary: &FixedSalary) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO fixed_salary (effective_date, monthly_amount, payments_per_year) VALUES (?1, ?2, ?3)",
            params![salary.effective_date.to_string(), salary.monthly_amount, salary.payments_per_year],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_fixed_salaries(&self) -> Result<Vec<FixedSalary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, effective_date, monthly_amount, payments_per_year FROM fixed_salary ORDER BY effective_date ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            let date_str: String = row.get(1)?;
            Ok(FixedSalary {
                id: Some(row.get(0)?),
                effective_date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .expect("Invalid date in DB"),
                monthly_amount: row.get(2)?,
                payments_per_year: row.get(3)?,
            })
        })?;

        rows.collect()
    }

    pub fn get_fixed_salary(&self, id: i64) -> Result<Option<FixedSalary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, effective_date, monthly_amount, payments_per_year FROM fixed_salary WHERE id = ?1"
        )?;
        match stmt.query_row(params![id], |row| {
            let date_str: String = row.get(1)?;
            Ok(FixedSalary {
                id: Some(row.get(0)?),
                effective_date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .expect("Invalid date in DB"),
                monthly_amount: row.get(2)?,
                payments_per_year: row.get(3)?,
            })
        }) {
            Ok(salary) => Ok(Some(salary)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn update_fixed_salary(&self, salary: &FixedSalary) -> Result<()> {
        self.conn.execute(
            "UPDATE fixed_salary SET effective_date = ?1, monthly_amount = ?2, payments_per_year = ?3 WHERE id = ?4",
            params![salary.effective_date.to_string(), salary.monthly_amount, salary.payments_per_year, salary.id],
        )?;
        Ok(())
    }

    pub fn delete_fixed_salary(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM fixed_salary WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ─── Variable Salary ──────────────────────────────────────────────────────

    pub fn add_variable_salary(&self, salary: &VariableSalary) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO variable_salary (date, amount, description) VALUES (?1, ?2, ?3)",
            params![salary.date.to_string(), salary.amount, salary.description],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_variable_salaries(&self) -> Result<Vec<VariableSalary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, date, amount, description FROM variable_salary ORDER BY date ASC"
        )?;
        let rows = stmt.query_map([], |row| {
            let date_str: String = row.get(1)?;
            Ok(VariableSalary {
                id: Some(row.get(0)?),
                date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .expect("Invalid date in DB"),
                amount: row.get(2)?,
                description: row.get(3)?,
            })
        })?;

        rows.collect()
    }

    pub fn get_variable_salary(&self, id: i64) -> Result<Option<VariableSalary>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, date, amount, description FROM variable_salary WHERE id = ?1"
        )?;
        match stmt.query_row(params![id], |row| {
            let date_str: String = row.get(1)?;
            Ok(VariableSalary {
                id: Some(row.get(0)?),
                date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .expect("Invalid date in DB"),
                amount: row.get(2)?,
                description: row.get(3)?,
            })
        }) {
            Ok(salary) => Ok(Some(salary)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn update_variable_salary(&self, salary: &VariableSalary) -> Result<()> {
        self.conn.execute(
            "UPDATE variable_salary SET date = ?1, amount = ?2, description = ?3 WHERE id = ?4",
            params![salary.date.to_string(), salary.amount, salary.description, salary.id],
        )?;
        Ok(())
    }

    pub fn delete_variable_salary(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM variable_salary WHERE id = ?1", params![id])?;
        Ok(())
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
