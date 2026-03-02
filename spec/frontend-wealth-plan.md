# Frontend Implementation Plan: Wealth & Salary Tracking

## Overview
Add comprehensive wealth and salary tracking UI to the existing Finance Analysis application.

## Backend Status
✅ **COMPLETE** - All database tables, models, and API endpoints are implemented and ready.

## Frontend Changes Required

### 1. Navigation Updates
**File**: `templates/index.html`

**Changes**:
- Add new "Wealth Overview" button as FIRST tab (before "Year Overview")
- Update existing "Year Overview" button text to "Finance Overview" for clarity
- Navigation order: Wealth Overview → Finance Overview → Monthly → One-time → Recurring

**Code Location**: Lines 155-160

```html
<nav>
  <button class="active" onclick="showPage('wealth')">Wealth Overview</button>
  <button onclick="showPage('year')">Finance Overview</button>
  <button onclick="showPage('dashboard')">Monthly</button>
  <button onclick="showPage('singular')">One-time</button>
  <button onclick="showPage('regular')">Recurring</button>
</nav>
```

### 2. New Page Section: Wealth Overview
**Location**: After line 164 (before Year Overview section)

**Components to add**:

#### 2.1 Wealth Development Chart
- **Type**: Line chart (using Chart.js)
- **X-axis**: Dates (from wealth snapshots)
- **Y-axis**: Total wealth in €
- **Data source**: GET `/api/wealth` (all snapshots)
- **Canvas ID**: `wealthChart`

#### 2.2 Yearly Balance Table
- **Columns**: Year | Starting Wealth | Ending Wealth | Change | % Change
- **Calculation**:
  - Starting Wealth = First snapshot of year
  - Ending Wealth = First snapshot of next year (or last snapshot if no next year)
  - Change = Ending - Starting
  - % Change = (Change / Starting) * 100
- **Data source**: Calculated from wealth snapshots

#### 2.3 Wealth Snapshot Management
**Sub-section: Current Snapshots**
- Table showing all wealth snapshots
- Columns: Date | Components (expandable) | Total | Actions (Edit/Delete)
- Click on row to expand and show all components

**Sub-section: Add New Snapshot**
- Date picker
- Dynamic component fields (Add/Remove buttons)
- Each component: Name input + Amount input
- Submit button to POST `/api/wealth`
- Shows calculated total as components are added
- Error handling when a snapshot is added with the same date as an already existing one

#### 2.4 Salary Overview Section

**Sub-section A: Fixed Salary Management**
- Table of all fixed salary entries
- Columns: Effective Date | Monthly Salary | Payments/Year | Annual Total | Actions
- Form to add new entry: Date + Monthly Amount + Payments/Year
- API: POST `/api/salary/fixed`

**Sub-section B: Variable Salary Management**
- Table of all variable salary entries
- Columns: Date | Amount | Description | Actions
- Form to add new entry: Date + Amount + Description
- API: POST `/api/salary/variable`

**Sub-section C: Salary Development Table**
- **Table 1: Fixed Salary History**
  - Columns: Date from which the salary was effective | Monthly Salary | Payments/Year | Annual Salary
  - Shows each step in salary change
  - Annual salary = Monthy salary * Payments/Year

- **Table 2: January 1st Snapshot & Changes**
  - Columns: Year | Effective Salary (Jan 1) | Change vs Prev Year | % Change vs Prev | Cumulative % Change vs First
  - Effective Salary = The monthly salary * payments_per_year that was in effect on Jan 1
  - Change vs Prev = Difference from previous year's Jan 1 salary
  - % Change vs Prev = (Change / Prev Year Salary) * 100
  - Cumulative % = (Current - First) / First * 100

### 3. JavaScript Functions to Add

#### Data Fetching
```javascript
async function loadWealthSnapshots()
async function loadFixedSalaries()
async function loadVariableSalaries()
```

#### Chart Rendering
```javascript
function renderWealthChart(snapshots) {
  // Create line chart with Chart.js
  // X-axis: dates, Y-axis: totals
}
```

#### Table Calculations
```javascript
function calculateYearlyBalances(snapshots) {
  // Group by year, find first of each year
  // Calculate differences
}

function calculateSalaryByYear(fixedSalaries, variableSalaries) {
  // Determine which fixed salary applies to each year
  // Sum variable salaries per year
  // Return combined data
}

function calculateJan1Snapshots(fixedSalaries) {
  // For each year, find salary effective on Jan 1
  // Calculate changes and percentages
}
```

#### Form Handling
```javascript
async function submitWealthSnapshot(formData)
async function addWealthComponent() // Add new component field
function removeWealthComponent(index)
async function submitFixedSalary(formData)
async function submitVariableSalary(formData)
async function deleteWealthSnapshot(date)
async function deleteFixedSalary(id)
async function deleteVariableSalary(id)
```

#### Page Management
```javascript
function showWealthOverview() {
  // Load all data
  // Render chart
  // Render tables
}
```

### 4. CSS Additions

```css
/* Wealth component row */
.component-row {
  display: flex;
  gap: 10px;
  margin-bottom: 8px;
  align-items: center;
}

.component-row input[type="text"] {
  flex: 1;
}

.component-row input[type="number"] {
  width: 150px;
}

.component-row button {
  width: 30px;
  height: 30px;
  padding: 0;
}

/* Expandable row */
.expandable-row {
  cursor: pointer;
}

.expandable-row:hover {
  background: var(--surface2);
}

.component-details {
  display: none;
  padding: 10px;
  background: var(--surface2);
}

.component-details.show {
  display: block;
}

/* Salary tables */
.salary-section {
  margin-top: 40px;
}

.salary-section h3 {
  margin-bottom: 16px;
}

/* Positive/negative indicators */
.positive {
  color: #4ade80;
}

.negative {
  color: #f87171;
}
```

### 5. Implementation Order

**Phase 1: Basic Structure** (~100 lines)
1. Add navigation button
2. Add empty page section with header
3. Wire up showPage('wealth')

**Phase 2: Wealth Snapshots** (~200 lines)
1. Add wealth snapshot form with dynamic components
2. Add snapshot list table
3. Implement add/edit/delete functions
4. Add wealth development chart

**Phase 3: Salary Forms** (~150 lines)
1. Add fixed salary form and table
2. Add variable salary form and table
3. Implement add/edit/delete functions

**Phase 4: Calculations & Tables** (~200 lines)
1. Implement yearly balance calculations
2. Implement salary development calculations
3. Render all three salary tables
4. Add percentage change formatting

**Total Estimated Addition**: ~650 lines of HTML/CSS/JavaScript

### 6. API Integration Points

All endpoints are ready:
- `GET /api/wealth` - List all snapshots
- `POST /api/wealth` - Create snapshot
- `GET /api/wealth/:date` - Get specific snapshot
- `PUT /api/wealth/:date` - Update snapshot
- `DELETE /api/wealth/:date` - Delete snapshot
- `GET /api/salary/fixed` - List fixed salaries
- `POST /api/salary/fixed` - Create fixed salary
- `PUT /api/salary/fixed/:id` - Update fixed salary
- `DELETE /api/salary/fixed/:id` - Delete fixed salary
- `GET /api/salary/variable` - List variable salaries
- `POST /api/salary/variable` - Create variable salary
- `PUT /api/salary/variable/:id` - Update variable salary
- `DELETE /api/salary/variable/:id` - Delete variable salary

### 7. Testing Checklist

After implementation:
- [ ] Wealth Overview tab appears first
- [ ] Can add wealth snapshot with multiple components
- [ ] Error handling when snapshots are added with the same date
- [ ] Components calculate correct total
- [ ] Multiple snapshots with different components handled correctly
- [ ] Wealth chart displays correctly
- [ ] Yearly balance table shows correct calculations
- [ ] Positive and negative balances handled
- [ ] Can add/edit/delete fixed salary entries
- [ ] Can add/edit/delete variable salary entries
- [ ] Salary development tables show correct data
- [ ] Percentage changes calculate correctly
- [ ] All tables format numbers with German locale (€ symbol, decimal comma)
- [ ] Responsive layout works on different screen sizes

### 8. Future Enhancements (Not in Initial Scope)

- Export wealth/salary data to Excel
- Import wealth snapshots from CSV
- Interactive chart tooltips showing component breakdown
- Date range filter for wealth chart
- Salary projection calculator
- Wealth goal setting and tracking

## Ready to Implement?

Once you approve this plan, I'll implement it in 4-5 focused edits to the `templates/index.html` file, adding approximately 650 lines of code while maintaining the existing dark theme and responsive design.
