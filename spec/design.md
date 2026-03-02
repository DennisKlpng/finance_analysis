# Finance Analysis Tool - Design Decisions

## Architecture Decisions

### 1. Technology Stack

#### 1.1 Backend Framework: Axum 0.7
**Decision**: Use Axum for the web server framework.

**Reasoning**:
- Modern, type-safe async web framework built on Tokio
- Excellent performance with minimal overhead
- Clean API design with extractors and middleware
- Strong ecosystem integration (Tower, Hyper)
- Compile-time guarantees reduce runtime errors
- Built-in support for multipart form data (needed for file uploads)

**Alternatives Considered**:
- Actix-web: More mature but more complex API
- Rocket: Simpler but less flexible, slower compilation
- Warp: Similar but less ergonomic API

#### 1.2 Database: SQLite with rusqlite
**Decision**: Use SQLite with the `rusqlite` crate (bundled feature enabled).

**Reasoning**:
- Zero-configuration embedded database
- Single-file storage perfect for local application
- ACID compliance ensures data integrity
- Excellent Rust support via rusqlite
- Bundled feature eliminates external dependencies
- Sufficient performance for personal finance tracking
- Easy backup (just copy the .db file)

**Alternatives Considered**:
- PostgreSQL/MySQL: Overkill for single-user local app
- JSON files: No querying capabilities, harder to maintain consistency
- CSV files: No relational structure, poor data integrity

#### 1.3 Date/Time Library: chrono
**Decision**: Use `chrono` for all date/time operations.

**Reasoning**:
- Industry standard for Rust date/time handling
- NaiveDate perfect for calendar dates without timezone complexity
- Excellent parsing and formatting support
- Serde integration for JSON serialization
- Arithmetic operations on dates (needed for monthly calculations)

**Alternatives Considered**:
- time crate: Less feature-complete for our use case
- Custom implementation: Reinventing the wheel

### 2. Frontend Architecture

#### 2.1 Single Page Application (SPA) with Embedded HTML
**Decision**: Embed entire frontend as a single HTML file in the binary using `include_str!`.

**Reasoning**:
- Zero build step deployment - just compile and run
- No asset serving complexity
- All resources embedded (HTML, CSS, JavaScript)
- Faster initial load (no external requests)
- Simpler deployment (single binary)
- Reduced attack surface (no file serving vulnerabilities)

**Trade-offs**:
- Larger binary size (acceptable for local tool)
- No hot reload during development (minor inconvenience)
- Code organization slightly less modular

#### 2.2 Visualization: Chart.js via CDN
**Decision**: Use Chart.js loaded from CDN for pie charts.

**Reasoning**:
- Popular, well-documented charting library
- Simple API for creating pie charts
- Responsive and interactive by default
- No build step required (can use CDN)
- Canvas-based rendering (good performance)

**Alternatives Considered**:
- D3.js: Too complex for simple pie charts
- Plotly: Heavier dependency
- SVG rendering: More implementation work

#### 2.3 UI Framework: Vanilla JavaScript
**Decision**: No JavaScript framework, use vanilla JS with DOM manipulation.

**Reasoning**:
- Simple enough application doesn't warrant framework overhead
- Faster initial load without framework bundle
- More control over behavior
- Easier to embed in single file
- No build tooling required

**Alternatives Considered**:
- React/Vue/Svelte: Overkill for this use case, requires build step

### 3. Data Model Design

#### 3.1 Separate Tables for Singular and Regular Entries
**Decision**: Maintain two separate tables: `singular_entries` and `regular_entries`.

**Reasoning**:
- Clear schema distinction (date vs date range)
- Simpler queries (no nullable date ranges)
- Better type safety in Rust models
- Easier to reason about data
- More efficient indexing

**Trade-offs**:
- Some code duplication between CRUD operations
- Queries need to join both tables for summaries

#### 3.2 Category Storage: String Enum Codes
**Decision**: Store categories as string codes (e.g., "T1", "N2") in database.

**Reasoning**:
- Compact storage (2 characters vs full names)
- Easy to change display names without migration
- Type-safe in Rust with enum mapping
- Simple to validate
- Human-readable in raw database

**Alternatives Considered**:
- Integer IDs: Less readable in database
- Full names: Wastes space, harder to maintain consistency

#### 3.3 Database Initialization Policy
**Decision**: Only initialize database if file doesn't exist or is empty.

**Reasoning**:
- Prevents accidental data loss
- Allows manual database inspection/modification
- Supports backup/restore workflows
- Clear intent: don't destructively recreate

**Implementation**:
```rust
let is_new = std::fs::metadata(path)
    .map(|m| m.len() == 0)
    .unwrap_or(true); // file doesn't exist → treat as new
if is_new {
    db.initialize()?;
}
```

### 4. Excel/ODS Import Design

#### 4.1 Marker-Based Section Detection
**Decision**: Use marker text rows ("Variable Kosten", "Variable Einnahmen") to separate regular from singular entries.

**Reasoning**:
- Robust against varying row counts in each section
- Clear visual separation in spreadsheet
- Allows blank rows within sections
- Handles legacy data with unexpected column values
- User-friendly (visible in Excel)

**Evolution**: Initially attempted column-based detection (date in column B), but this failed with legacy data. Marker-based approach is more reliable.

#### 4.2 Full Column Scan for Markers
**Decision**: Search all columns in a row for marker text, not just column A.

**Reasoning**:
- More flexible (marker can be in any column)
- Handles merged cells
- More forgiving of spreadsheet variations
- Minimal performance impact

**Implementation**:
```rust
fn row_contains_marker(sheet: &Range<Data>, row: usize, marker: &str) -> bool {
    for col in 0..width {
        if cell.to_string().contains(marker) {
            return true;
        }
    }
    false
}
```

#### 4.3 External Category Mapping File
**Decision**: Use external `excel_mapping.json` file (gitignored) for category mappings.

**Reasoning**:
- Privacy: Excel values may contain personal/sensitive information
- Flexibility: Users can customize Excel format without code changes
- Separation of concerns: Configuration vs code
- Template provided for easy setup

**Structure**:
```json
{
  "type_category_recurring": { "Excel Value": "T1" },
  "type_category_singular": { "Excel Value": "T2" },
  "necessity_category": { "Excel Value": "N1" }
}
```

#### 4.4 German Number Format Support
**Decision**: Automatically convert German number format (comma as decimal separator).

**Reasoning**:
- Users work with German Excel/ODS files
- Simple transformation: replace `,` with `.`
- Transparent to user
- No locale configuration required

**Implementation**:
```rust
Data::String(s) => {
    let normalized = s.replace(',', ".");
    normalized.parse::<f64>()
}
```

#### 4.5 Multiple Date Format Support
**Decision**: Support multiple date formats with fallback chain.

**Reasoning**:
- Different tools export different formats
- ODS uses ISO dates (DateTimeIso data type)
- Excel might use serial numbers or formatted strings
- German short dates common (01.02.25)

**Formats Supported**:
1. `Data::DateTimeIso` - ODS ISO format
2. `Data::DateTime` - Excel DateTime
3. `Data::Int` / `Data::Float` - Excel serial numbers
4. `Data::String` with patterns:
   - `%d.%m.%Y` (01.02.2025)
   - `%d.%m.%y` (01.02.25)
   - `%Y-%m-%d` (2025-02-01)
   - `%d/%m/%Y` (01/02/2025)

### 5. API Design

#### 5.1 RESTful Endpoints
**Decision**: Use RESTful conventions for API routes.

**Reasoning**:
- Industry standard, familiar to developers
- Clear resource-oriented structure
- HTTP methods map to operations (GET/POST/PUT/DELETE)
- Easy to test and document

**Routes**:
```
GET  /api/singular          - List all singular entries
POST /api/singular          - Create singular entry
GET  /api/singular/:id      - Get singular entry
PUT  /api/singular/:id      - Update singular entry
DELETE /api/singular/:id    - Delete singular entry

GET  /api/regular           - List all regular entries
POST /api/regular           - Create regular entry
GET  /api/regular/:id       - Get regular entry
PUT  /api/regular/:id       - Update regular entry
DELETE /api/regular/:id     - Delete regular entry

GET  /api/month/:year/:month          - Monthly summary
GET  /api/year/:year                  - Yearly summary
GET  /api/months                      - Available months
GET  /api/expenses/distribution/:year/:month - Expense distribution

POST /api/import/excel      - Import Excel/ODS file
```

#### 5.2 Shared Application State
**Decision**: Use `Arc<Mutex<Database>>` for shared database access.

**Reasoning**:
- Simple concurrency model
- Database connections aren't Send, so we share wrapped connection
- Arc provides shared ownership across async tasks
- Mutex ensures exclusive access during operations
- Sufficient for single-user local application

**Trade-offs**:
- Could use connection pool for better concurrency
- For this use case, single connection is adequate

### 6. Error Handling

#### 6.1 Error Response Strategy
**Decision**: Return JSON error responses with HTTP status codes.

**Reasoning**:
- Consistent error format for frontend
- HTTP status codes provide semantic meaning
- JSON structure allows detailed error messages
- Easy to handle in JavaScript

**Example**:
```rust
fn db_err(e: rusqlite::Error) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": e.to_string() })),
    ).into_response()
}
```

#### 6.2 Import Error Collection
**Decision**: Collect all import errors rather than failing on first error.

**Reasoning**:
- User can see all issues at once
- Partial imports still succeed
- Better user experience
- Errors are reported but don't stop processing

**Structure**:
```rust
pub struct ImportStats {
    pub regular_expenses: usize,
    pub singular_expenses: usize,
    pub regular_incomes: usize,
    pub singular_incomes: usize,
    pub errors: Vec<String>,
}
```

### 7. Testing Strategy

#### 7.1 Integration Tests for Import
**Decision**: Use integration tests in `tests/` directory for import functionality.

**Reasoning**:
- Import is critical and complex
- Integration test validates entire flow
- Real ODS file tests actual calamine behavior
- Separate from unit tests
- Uses library crate interface

**Structure**:
```
tests/
  import_test.rs    - Integration tests
test/               - Test data (gitignored)
  test.ods          - Sample ODS file
  excel_mapping_test.json - Test mappings
```

#### 7.2 Test Data Validation
**Decision**: Verify specific field values, not just counts.

**Reasoning**:
- Ensures correct parsing (amounts, dates, categories)
- Catches subtle bugs (wrong column mappings)
- Documents expected behavior
- More confidence in import correctness

### 8. Security and Privacy

#### 8.1 Local-Only CORS Policy
**Decision**: Allow all origins in CORS (development convenience).

**Reasoning**:
- Application runs on localhost only
- No sensitive data exposed to internet
- Simplifies development
- Single-user application

**Implementation**:
```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```

#### 8.2 Gitignore Strategy
**Decision**: Exclude database files, mapping files, and test data from git.

**Reasoning**:
- Privacy: Database contains personal financial data
- Privacy: Mapping file may contain personal category names
- Cleanliness: Test data shouldn't be in repo
- Template provided for mapping file

**Excluded**:
- `*.db`
- `excel_mapping.json`
- `test/`

### 9. Build and Deployment

#### 9.1 Library/Binary Split
**Decision**: Expose modules as library (`lib.rs`) while maintaining binary (`main.rs`).

**Reasoning**:
- Integration tests need access to db, models, import modules
- Cargo test can't access binary-only code
- Library still compiles to single binary
- Clean separation of concerns

**Structure**:
```toml
[lib]
name = "finance_analysis"
path = "src/lib.rs"

[[bin]]
name = "finance_analysis"
path = "src/main.rs"
```

#### 9.2 Dependency Selection
**Decision**: Use minimal, well-maintained dependencies.

**Dependencies**:
- `axum` 0.7 - Web framework
- `tokio` - Async runtime
- `rusqlite` (bundled) - Database
- `serde` + `serde_json` - Serialization
- `chrono` - Date/time handling
- `tower-http` (cors) - CORS middleware
- `anyhow` - Error handling
- `tracing` + `tracing-subscriber` - Logging
- `calamine` - Excel/ODS parsing

**Reasoning**: Each dependency solves a specific problem with minimal overlap.

## Future Considerations

### Potential Enhancements
1. **Export functionality**: Export data back to Excel/ODS
2. **Budgeting**: Set and track budget limits per category
3. **Trends**: Visualize spending trends over time
4. **Search**: Filter entries by date range, category, description
5. **Backup**: Automated database backup functionality
6. **Multi-year**: Compare year-over-year statistics

### Known Limitations
1. No undo functionality (rely on database backups)
2. No authentication (local-only, single-user assumption)
3. Limited visualization options (only pie charts currently)
4. No export to other formats (PDF, CSV)
5. Manual category mapping setup required for imports

### Technical Debt
1. Header row detection is naive (assumes Row 1)
2. Could add connection pooling for better concurrency
3. Frontend could be modularized for better organization
4. More comprehensive error messages for import failures
