use chrono::NaiveDate;
use std::path::Path;

// Import from the main crate
use finance_analysis::db::Database;
use finance_analysis::import::import_excel;
use finance_analysis::models::{NecessityCategory, Periodicity, TypeCategory};

#[test]
fn test_ods_import() {
    // Setup paths
    let test_file = Path::new("test/test.ods");
    let mapping_file = "test/excel_mapping_test.json";
    let db_path = "test_output.db";

    // Clean up any existing test database
    let _ = std::fs::remove_file(db_path);

    // Create database
    let db = Database::new(db_path).expect("Failed to create test database");

    // Import
    let result = import_excel(test_file, mapping_file, &db, 2025);
    assert!(result.is_ok(), "Import failed: {:?}", result.err());

    let stats = result.unwrap();

    // Print stats for debugging
    println!("Import stats:");
    println!("  Regular expenses: {}", stats.regular_expenses);
    println!("  Singular expenses: {}", stats.singular_expenses);
    println!("  Regular incomes: {}", stats.regular_incomes);
    println!("  Singular incomes: {}", stats.singular_incomes);
    if !stats.errors.is_empty() {
        println!("  Errors/Warnings:");
        for err in &stats.errors {
            println!("    - {}", err);
        }
    }

    // Verify counts
    assert_eq!(
        stats.regular_expenses, 1,
        "Expected 1 regular expense, got {}",
        stats.regular_expenses
    );
    assert_eq!(
        stats.singular_expenses, 1,
        "Expected 1 one-time expense, got {}",
        stats.singular_expenses
    );
    assert_eq!(
        stats.regular_incomes, 1,
        "Expected 1 regular income, got {}",
        stats.regular_incomes
    );
    assert_eq!(
        stats.singular_incomes, 1,
        "Expected 1 one-time income, got {}",
        stats.singular_incomes
    );

    // Report errors if any
    if !stats.errors.is_empty() {
        println!("Import warnings:");
        for err in &stats.errors {
            println!("  - {}", err);
        }
    }

    // Verify specific entries
    // 1. Recurring expense: 62€ yearly, Feb-Aug
    let regular_expenses = db
        .get_all_regular()
        .expect("Failed to get regular entries");
    let recurring_expense = regular_expenses
        .iter()
        .find(|e| !e.is_income)
        .expect("No recurring expense found");

    assert_eq!(
        recurring_expense.amount, 62.0,
        "Expected recurring expense amount 62, got {}",
        recurring_expense.amount
    );
    assert_eq!(
        recurring_expense.periodicity,
        Periodicity::Yearly,
        "Expected yearly periodicity"
    );
    assert_eq!(
        recurring_expense.description, "Test Claude 1",
        "Expected description 'Test Claude 1', got '{}'",
        recurring_expense.description
    );
    assert_eq!(
        recurring_expense.start_date,
        NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
        "Expected start date February 1, 2025"
    );
    assert_eq!(
        recurring_expense.end_date,
        NaiveDate::from_ymd_opt(2025, 8, 31).unwrap(),
        "Expected end date August 31, 2025"
    );

    // 2. One-time expense: 50€ on 1.2.2025, T2/N1
    let singular_expenses = db
        .get_all_singular()
        .expect("Failed to get singular entries");
    let onetime_expense = singular_expenses
        .iter()
        .find(|e| !e.is_income)
        .expect("No one-time expense found");

    assert_eq!(
        onetime_expense.amount, 50.0,
        "Expected one-time expense amount 50, got {}",
        onetime_expense.amount
    );
    assert_eq!(
        onetime_expense.date,
        NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
        "Expected date February 1, 2025"
    );
    assert_eq!(
        onetime_expense.description, "Test Claude 2",
        "Expected description 'Test Claude 2', got '{}'",
        onetime_expense.description
    );
    assert_eq!(
        onetime_expense.type_category,
        TypeCategory::T2,
        "Expected type category T2 (Freizeit)"
    );
    assert_eq!(
        onetime_expense.necessity_category,
        NecessityCategory::N1,
        "Expected necessity category N1 (Notwendig)"
    );

    // 3. Recurring income: 1000€ monthly, Mar-Nov
    let recurring_income = regular_expenses
        .iter()
        .find(|e| e.is_income)
        .expect("No recurring income found");

    assert_eq!(
        recurring_income.amount, 1000.0,
        "Expected recurring income amount 1000, got {}",
        recurring_income.amount
    );
    assert_eq!(
        recurring_income.periodicity,
        Periodicity::Monthly,
        "Expected monthly periodicity"
    );
    assert_eq!(
        recurring_income.description, "Test Claude 3",
        "Expected description 'Test Claude 3', got '{}'",
        recurring_income.description
    );
    assert_eq!(
        recurring_income.start_date,
        NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
        "Expected start date March 1, 2025"
    );
    assert_eq!(
        recurring_income.end_date,
        NaiveDate::from_ymd_opt(2025, 11, 30).unwrap(),
        "Expected end date November 30, 2025"
    );

    // 4. One-time income: 800€ on 1.10.2025
    let onetime_income = singular_expenses
        .iter()
        .find(|e| e.is_income)
        .expect("No one-time income found");

    assert_eq!(
        onetime_income.amount, 800.0,
        "Expected one-time income amount 800, got {}",
        onetime_income.amount
    );
    assert_eq!(
        onetime_income.date,
        NaiveDate::from_ymd_opt(2025, 10, 1).unwrap(),
        "Expected date October 1, 2025"
    );
    assert_eq!(
        onetime_income.description, "Test Claude 4",
        "Expected description 'Test Claude 4', got '{}'",
        onetime_income.description
    );

    // Clean up - drop database connection before removing file
    drop(db);
    std::fs::remove_file(db_path).expect("Failed to remove test database");

    println!("✓ All import tests passed!");
}
