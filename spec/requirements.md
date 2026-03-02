# Finance Analysis Tool - Requirements

## Overview
A local finance tracking application built in Rust with a web interface for managing personal income and expenses.

## Core Requirements

### 1. Application Architecture
- **Language**: Rust
- **Database**: SQLite for persistent storage
- **Interface**: Local web server (127.0.0.1:3000)
- **Deployment**: Single binary with embedded frontend assets
- **Operating Mode**: Local-only, no remote access

### 2. Data Model

#### 2.1 Entry Types
- **Singular Entries**: One-time income or expenses with a specific date
- **Regular Entries**: Recurring income or expenses with:
  - Periodicity (Monthly or Yearly)
  - Start and end date range

#### 2.2 Categories
All entries must be categorized by two dimensions:

**Type Categories** (T1-TN):
- see models.rs

**Necessity Categories** (N1-N3):
- N1: Notwendig (Necessary)
- N2: Nützlich (Useful)
- N3: Luxus (Luxury)

**Note**: Incomes are exempt from categorization and use default values (T10/N1).

#### 2.3 Entry Fields

**Singular Entry**:
- ID (optional, auto-generated)
- Amount (f64)
- Description (String)
- Date (NaiveDate)
- Type Category
- Necessity Category
- Is Income (bool)

**Regular Entry**:
- ID (optional, auto-generated)
- Amount (f64, per period)
- Description (String)
- Periodicity (Monthly/Yearly)
- Start Date (NaiveDate)
- End Date (NaiveDate)
- Type Category
- Necessity Category
- Is Income (bool)

### 3. Database Requirements

#### 3.1 Initialization
- Database file path: Configurable (default: `Z:/Finanzen/finance.db`)
- Initialization policy: **Only create tables if the database file doesn't exist or is empty**
- Never overwrite existing non-empty database

#### 3.2 Operations
- CRUD operations for both singular and regular entries
- Monthly summary calculations
- Yearly summary calculations
- Expense distribution analytics by category

### 4. User Interface Requirements

#### 4.1 Year Overview (Front Page)
- Display current year by default
- Year navigation controls (◀ Previous | Next ▶)
- Monthly breakdown table showing:
  - Income per month
  - Expenses per month
  - Balance per month
  - Totals row
- Two pie charts:
  - Expense distribution by Type Category
  - Expense distribution by Necessity Category

#### 4.2 Entry Management
- Forms for creating/editing singular and regular entries
- Separate views for expenses and incomes
- Category selection dropdowns with full German names
- Date pickers for singular entries
- Month range selectors for regular entries

#### 4.3 Month Navigation
- List of available months based on existing entries
- Always include current month even if no entries exist

### 5. Excel/ODS Import

#### 5.1 File Format Support
- Microsoft Excel (.xlsx)
- OpenDocument Spreadsheet (.ods)

#### 5.2 Sheet Structure
**Sheet 2: Expenses**
- Regular expenses section (top)
- Marker row: Contains "Variable Kosten" (in any column)
- Singular expenses section (below marker)

**Sheet 3: Incomes**
- Regular incomes section (top)
- Marker row: Contains "Variable Einnahmen" (in any column)
- Singular incomes section (below marker)

#### 5.3 Column Layout

**Regular Expenses**:
- Column C: Type Category (mapped via JSON)
- Column D: Necessity Category (mapped via JSON)
- Column E: Periodicity (monatlich/jährlich or monthly/yearly)
- Column G: Amount
- Column H: Description
- Column I: Start Month (1-12)
- Column J: End Month (1-12)

**Singular Expenses**:
- Column B: Date
- Column C: Type Category (mapped via JSON)
- Column D: Necessity Category (mapped via JSON)
- Column G: Amount
- Column H: Description

**Regular Incomes**:
- Column C: Periodicity
- Column D: Amount
- Column F: Description
- Column G: Start Month
- Column H: End Month

**Singular Incomes**:
- Column B: Date
- Column D: Amount
- Column F: Description

#### 5.4 Category Mapping
- External JSON configuration file (`excel_mapping.json`)
- Maps Excel cell values to system category codes
- Template provided (`excel_mapping_template.json`)
- Excluded from version control for privacy

#### 5.5 Import Features
- Year parameter required for import
- Skip header rows automatically
- Report import statistics (counts per type)
- Collect and report errors/warnings
- Support German number format (comma as decimal separator)
- Support multiple date formats:
  - ISO: 2025-02-01
  - German long: 01.02.2025
  - German short: 01.02.25
  - Alternative: 01/02/2025

### 6. Localization

#### 6.1 Number Format
- Accept German format: `1.234,56` (period as thousands separator, comma as decimal)
- Store internally as standard float

#### 6.2 Date Format
- Display: German format (DD.MM.YYYY)
- Accept multiple input formats (see 5.5)
- Store: ISO format internally

### 7. Non-Functional Requirements

#### 7.1 Performance
- Fast local execution
- Minimal startup time
- Responsive UI

#### 7.2 Security
- Local-only access (127.0.0.1)
- No authentication required (single-user system)
- Sensitive mapping data excluded from source control

#### 7.3 Maintainability
- Clean code structure
- Comprehensive error handling
- Integration tests for critical paths
- Documentation for setup and usage

#### 7.4 Deployment
- Single binary executable
- No build step required for frontend
- All assets embedded in binary
- Minimal dependencies

### 8. Out of Scope
- Multi-user support
- Remote access / cloud sync
- Mobile applications
- Budget planning features
- Automated bank integration
- Currency conversion
- Report generation (PDF/Excel exports)