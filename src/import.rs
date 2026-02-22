use crate::db::Database;
use crate::models::{NecessityCategory, Periodicity, RegularEntry, SingularEntry, TypeCategory};
use anyhow::{anyhow, Context, Result};
use calamine::{open_workbook_auto, Data, Reader};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct CategoryMapping {
    pub type_category_recurring: HashMap<String, String>,
    pub type_category_singular: HashMap<String, String>,
    pub necessity_category: HashMap<String, String>,
}

impl CategoryMapping {
    pub fn load(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read mapping file. Please create excel_mapping.json from the template.")?;
        let mapping: CategoryMapping = serde_json::from_str(&content)
            .context("Failed to parse mapping JSON")?;
        Ok(mapping)
    }

    fn map_type_recurring(&self, excel_value: &str) -> Result<TypeCategory> {
        let mapped = self.type_category_recurring
            .get(excel_value)
            .ok_or_else(|| anyhow!("Unknown recurring type category: '{}'", excel_value))?;
        TypeCategory::from_str(mapped)
            .ok_or_else(|| anyhow!("Invalid mapped type category: '{}'", mapped))
    }

    fn map_type_singular(&self, excel_value: &str) -> Result<TypeCategory> {
        let mapped = self.type_category_singular
            .get(excel_value)
            .ok_or_else(|| anyhow!("Unknown singular type category: '{}'", excel_value))?;
        TypeCategory::from_str(mapped)
            .ok_or_else(|| anyhow!("Invalid mapped type category: '{}'", mapped))
    }

    fn map_necessity(&self, excel_value: &str) -> Result<NecessityCategory> {
        let mapped = self.necessity_category
            .get(excel_value)
            .ok_or_else(|| anyhow!("Unknown necessity category: '{}'", excel_value))?;
        NecessityCategory::from_str(mapped)
            .ok_or_else(|| anyhow!("Invalid mapped necessity category: '{}'", mapped))
    }
}

pub fn import_excel(excel_path: &Path, mapping_path: &str, db: &Database, year: i32) -> Result<ImportStats> {
    let mapping = CategoryMapping::load(mapping_path)?;
    let mut workbook = open_workbook_auto(excel_path)
        .context("Failed to open Excel file")?;

    let mut stats = ImportStats::default();

    // Sheet 2 (index 1): Expenses
    if let Some(Ok(sheet)) = workbook.worksheet_range_at(1) {
        import_expenses_sheet(&sheet, &mapping, db, year, &mut stats)?;
    } else {
        return Err(anyhow!("Expenses sheet (sheet 2) not found"));
    }

    // Sheet 3 (index 2): Incomes
    if let Some(Ok(sheet)) = workbook.worksheet_range_at(2) {
        import_incomes_sheet(&sheet, &mapping, db, year, &mut stats)?;
    } else {
        return Err(anyhow!("Incomes sheet (sheet 3) not found"));
    }

    Ok(stats)
}

#[derive(Debug, Default)]
pub struct ImportStats {
    pub regular_expenses: usize,
    pub singular_expenses: usize,
    pub regular_incomes: usize,
    pub singular_incomes: usize,
    pub errors: Vec<String>,
}

fn import_expenses_sheet(
    sheet: &calamine::Range<Data>,
    mapping: &CategoryMapping,
    db: &Database,
    year: i32,
    stats: &mut ImportStats,
) -> Result<()> {
    let mut in_singular_section = false;
    let (height, _) = sheet.get_size();

    for row_idx in 0..height {
        // Check for "Variable Kosten" marker
        if let Some(cell) = sheet.get((row_idx, 0)) {
            if cell.to_string().contains("Variable Kosten") {
                in_singular_section = true;
                continue;
            }
        }

        if in_singular_section {
            // One-time expense: B=date, C=type, D=necessity, G=amount, H=description
            match parse_singular_expense(sheet, row_idx, mapping, year) {
                Ok(Some(entry)) => {
                    if let Err(e) = db.add_singular(&entry) {
                        stats.errors.push(format!("Row {}: {}", row_idx + 1, e));
                    } else {
                        stats.singular_expenses += 1;
                    }
                }
                Ok(None) => {} // Empty row
                Err(e) => stats.errors.push(format!("Row {}: {}", row_idx + 1, e)),
            }
        } else {
            // Regular expense: C=type, D=necessity, E=periodicity, G=amount, H=description, I=start_month, J=end_month
            match parse_regular_expense(sheet, row_idx, mapping, year) {
                Ok(Some(entry)) => {
                    if let Err(e) = db.add_regular(&entry) {
                        stats.errors.push(format!("Row {}: {}", row_idx + 1, e));
                    } else {
                        stats.regular_expenses += 1;
                    }
                }
                Ok(None) => {} // Empty row
                Err(e) => stats.errors.push(format!("Row {}: {}", row_idx + 1, e)),
            }
        }
    }

    Ok(())
}

fn import_incomes_sheet(
    sheet: &calamine::Range<Data>,
    mapping: &CategoryMapping,
    db: &Database,
    year: i32,
    stats: &mut ImportStats,
) -> Result<()> {
    let mut in_singular_section = false;
    let (height, _) = sheet.get_size();

    for row_idx in 0..height {
        // Check for separator (empty description in column F for regular, or specific marker)
        // For now, let's use a simple heuristic: if column B has a date and column C is empty, it's singular
        let has_date_in_b = sheet.get((row_idx, 1)).and_then(|c| parse_excel_date(c, year).ok()).is_some();
        let c_empty = sheet.get((row_idx, 2)).map_or(true, |c| matches!(c, Data::Empty));

        if has_date_in_b && c_empty {
            in_singular_section = true;
        }

        if in_singular_section {
            // One-time income: B=date, D=amount, F=description
            match parse_singular_income(sheet, row_idx, mapping, year) {
                Ok(Some(entry)) => {
                    if let Err(e) = db.add_singular(&entry) {
                        stats.errors.push(format!("Row {}: {}", row_idx + 1, e));
                    } else {
                        stats.singular_incomes += 1;
                    }
                }
                Ok(None) => {}
                Err(e) => stats.errors.push(format!("Row {}: {}", row_idx + 1, e)),
            }
        } else {
            // Regular income: C=periodicity, D=amount, F=description, G=start_month, H=end_month
            match parse_regular_income(sheet, row_idx, mapping, year) {
                Ok(Some(entry)) => {
                    if let Err(e) = db.add_regular(&entry) {
                        stats.errors.push(format!("Row {}: {}", row_idx + 1, e));
                    } else {
                        stats.regular_incomes += 1;
                    }
                }
                Ok(None) => {}
                Err(e) => stats.errors.push(format!("Row {}: {}", row_idx + 1, e)),
            }
        }
    }

    Ok(())
}

// ─── Parsers ─────────────────────────────────────────────────────────────────

fn parse_regular_expense(
    sheet: &calamine::Range<Data>,
    row: usize,
    mapping: &CategoryMapping,
    year: i32,
) -> Result<Option<RegularEntry>> {
    let amount = get_f64(sheet, row, 6)?; // G
    if amount == 0.0 {
        return Ok(None); // Skip empty rows
    }

    let type_cat_str = get_string(sheet, row, 2)?; // C
    let necessity_str = get_string(sheet, row, 3)?; // D
    let periodicity_str = get_string(sheet, row, 4)?; // E
    let description = get_string(sheet, row, 7)?; // H
    let start_month = get_month(sheet, row, 8)?; // I
    let end_month = get_month(sheet, row, 9)?; // J

    let type_category = mapping.map_type_recurring(&type_cat_str)?;
    let necessity_category = mapping.map_necessity(&necessity_str)?;
    let periodicity = parse_periodicity(&periodicity_str)?;

    let start_date = NaiveDate::from_ymd_opt(year, start_month, 1)
        .ok_or_else(|| anyhow!("Invalid start month: {}", start_month))?;
    let end_date = month_end_date(year, end_month)?;

    Ok(Some(RegularEntry {
        id: None,
        amount,
        description,
        periodicity,
        start_date,
        end_date,
        type_category,
        necessity_category,
        is_income: false,
    }))
}

fn parse_singular_expense(
    sheet: &calamine::Range<Data>,
    row: usize,
    mapping: &CategoryMapping,
    year: i32,
) -> Result<Option<SingularEntry>> {
    let amount = get_f64(sheet, row, 6)?; // G
    if amount == 0.0 {
        return Ok(None);
    }

    let date = parse_excel_date(sheet.get((row, 1)).ok_or_else(|| anyhow!("Missing date"))?, year)?; // B
    let type_cat_str = get_string(sheet, row, 2)?; // C
    let necessity_str = get_string(sheet, row, 3)?; // D
    let description = get_string(sheet, row, 7)?; // H

    let type_category = mapping.map_type_singular(&type_cat_str)?;
    let necessity_category = mapping.map_necessity(&necessity_str)?;

    Ok(Some(SingularEntry {
        id: None,
        amount,
        description,
        date,
        type_category,
        necessity_category,
        is_income: false,
    }))
}

fn parse_regular_income(
    sheet: &calamine::Range<Data>,
    row: usize,
    _mapping: &CategoryMapping,
    year: i32,
) -> Result<Option<RegularEntry>> {
    let amount = get_f64(sheet, row, 3)?; // D
    if amount == 0.0 {
        return Ok(None);
    }

    let periodicity_str = get_string(sheet, row, 2)?; // C
    let description = get_string(sheet, row, 5)?; // F
    let start_month = get_month(sheet, row, 6)?; // G
    let end_month = get_month(sheet, row, 7)?; // H

    let periodicity = parse_periodicity(&periodicity_str)?;
    let start_date = NaiveDate::from_ymd_opt(year, start_month, 1)
        .ok_or_else(|| anyhow!("Invalid start month: {}", start_month))?;
    let end_date = month_end_date(year, end_month)?;

    // Incomes don't have categories in the spec, use defaults
    Ok(Some(RegularEntry {
        id: None,
        amount,
        description,
        periodicity,
        start_date,
        end_date,
        type_category: TypeCategory::T10, // Sonstiges
        necessity_category: NecessityCategory::N1, // Notwendig
        is_income: true,
    }))
}

fn parse_singular_income(
    sheet: &calamine::Range<Data>,
    row: usize,
    _mapping: &CategoryMapping,
    year: i32,
) -> Result<Option<SingularEntry>> {
    let amount = get_f64(sheet, row, 3)?; // D
    if amount == 0.0 {
        return Ok(None);
    }

    let date = parse_excel_date(sheet.get((row, 1)).ok_or_else(|| anyhow!("Missing date"))?, year)?; // B
    let description = get_string(sheet, row, 5)?; // F

    Ok(Some(SingularEntry {
        id: None,
        amount,
        description,
        date,
        type_category: TypeCategory::T10, // Sonstiges
        necessity_category: NecessityCategory::N1, // Notwendig
        is_income: true,
    }))
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn get_string(sheet: &calamine::Range<Data>, row: usize, col: usize) -> Result<String> {
    let cell = sheet.get((row, col)).ok_or_else(|| anyhow!("Missing cell"))?;
    let s = cell.to_string().trim().to_string();
    if s.is_empty() {
        Err(anyhow!("Empty cell at row {}, col {}", row + 1, col + 1))
    } else {
        Ok(s)
    }
}

fn get_f64(sheet: &calamine::Range<Data>, row: usize, col: usize) -> Result<f64> {
    let cell = sheet.get((row, col)).ok_or_else(|| anyhow!("Missing cell"))?;
    match cell {
        Data::Float(f) => Ok(*f),
        Data::Int(i) => Ok(*i as f64),
        Data::String(s) => s.parse::<f64>().context("Failed to parse amount"),
        Data::Empty => Ok(0.0),
        _ => Err(anyhow!("Invalid number format")),
    }
}

fn get_month(sheet: &calamine::Range<Data>, row: usize, col: usize) -> Result<u32> {
    let cell = sheet.get((row, col)).ok_or_else(|| anyhow!("Missing month"))?;
    let month = match cell {
        Data::Int(i) => *i as u32,
        Data::Float(f) => *f as u32,
        Data::String(s) => s.parse::<u32>().context("Failed to parse month")?,
        _ => return Err(anyhow!("Invalid month format")),
    };

    if month < 1 || month > 12 {
        Err(anyhow!("Month out of range: {}", month))
    } else {
        Ok(month)
    }
}

fn parse_excel_date(cell: &Data, _year: i32) -> Result<NaiveDate> {
    match cell {
        Data::DateTime(excel_dt) => {
            // ExcelDateTime has as_f64() method
            let days = excel_dt.as_f64();
            let base = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
            let date = base + chrono::Duration::days(days as i64);
            Ok(date)
        }
        Data::String(s) => {
            // Try common formats
            NaiveDate::parse_from_str(s, "%d.%m.%Y")
                .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
                .or_else(|_| NaiveDate::parse_from_str(s, "%d/%m/%Y"))
                .context("Failed to parse date string")
        }
        Data::Float(days) => {
            let base = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
            let date = base + chrono::Duration::days(*days as i64);
            Ok(date)
        }
        _ => Err(anyhow!("Invalid date format")),
    }
}

fn parse_periodicity(s: &str) -> Result<Periodicity> {
    let lower = s.to_lowercase();
    if lower.contains("month") || lower.contains("monat") {
        Ok(Periodicity::Monthly)
    } else if lower.contains("year") || lower.contains("jahr") {
        Ok(Periodicity::Yearly)
    } else {
        Err(anyhow!("Unknown periodicity: '{}'", s))
    }
}

fn month_end_date(year: i32, month: u32) -> Result<NaiveDate> {
    let next_month_start = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }.ok_or_else(|| anyhow!("Invalid end month: {}", month))?;

    Ok(next_month_start.pred_opt().unwrap())
}
