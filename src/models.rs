use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TypeCategory {
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    T8,
    T9,
    T10
}

impl TypeCategory {
    pub fn as_str(&self) -> &str {
        match self {
            TypeCategory::T1 => "Lebensmittel&Haushaltsbedarf",
            TypeCategory::T2 => "Freizeit",
            TypeCategory::T3 => "Mobilität",
            TypeCategory::T4 => "Kleidung",
            TypeCategory::T5 => "Elektronik",
            TypeCategory::T6 => "Wohnungseinrichtung",
            TypeCategory::T7 => "Urlaub",
            TypeCategory::T8 => "Miete",
            TypeCategory::T9 => "Versicherungen",
            TypeCategory::T10 => "Sonstiges",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Lebensmittel&Haushaltsbedarf" => Some(TypeCategory::T1),
            "Freizeit" => Some(TypeCategory::T2),
            "Mobilität" => Some(TypeCategory::T3),
            "Kleidung" => Some(TypeCategory::T4),
            "Elektronik" => Some(TypeCategory::T5),
            "Wohnungseinrichtung" => Some(TypeCategory::T6),
            "Urlaub" => Some(TypeCategory::T7),
            "MieteC" => Some(TypeCategory::T8),
            "Versicherungen" => Some(TypeCategory::T9),
            "Sonstiges" => Some(TypeCategory::T10),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NecessityCategory {
    N1,
    N2,
    N3
}

impl NecessityCategory {
    pub fn as_str(&self) -> &str {
        match self {
            NecessityCategory::N1 => "Notwendig",
            NecessityCategory::N2 => "Nützlich",
            NecessityCategory::N3 => "Luxus",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Notwendig" => Some(NecessityCategory::N1),
            "Nützlich" => Some(NecessityCategory::N2),
            "Luxus" => Some(NecessityCategory::N3),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Periodicity {
    Monthly,
    Yearly,
}

impl Periodicity {
    pub fn as_str(&self) -> &str {
        match self {
            Periodicity::Monthly => "monthly",
            Periodicity::Yearly => "yearly",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "monthly" => Some(Periodicity::Monthly),
            "yearly" => Some(Periodicity::Yearly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingularEntry {
    pub id: Option<i64>,
    pub amount: f64,
    pub description: String,
    pub date: NaiveDate,
    pub type_category: TypeCategory,
    pub necessity_category: NecessityCategory,
    pub is_income: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegularEntry {
    pub id: Option<i64>,
    pub amount: f64,
    pub description: String,
    pub periodicity: Periodicity,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub type_category: TypeCategory,
    pub necessity_category: NecessityCategory,
    pub is_income: bool,
}

#[derive(Debug, Serialize)]
pub struct MonthSummary {
    pub year: i32,
    pub month: u32,
    pub singular_income: f64,
    pub singular_expenses: f64,
    pub regular_income: f64,
    pub regular_expenses: f64,
    pub total_income: f64,
    pub total_expenses: f64,
    pub net: f64,
}

// ─── Wealth Tracking ──────────────────────────────────────────────────────────

/// Standardized wealth component categories. Change names here to rename across the app.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WealthComponentCategory {
    W1,
    W2,
    W3,
    W4,
    W5,
    W6,
    W7,
}

impl WealthComponentCategory {
    pub fn as_str(&self) -> &str {
        match self {
            WealthComponentCategory::W1 => "Girokonto",
            WealthComponentCategory::W2 => "Sparkonto",
            WealthComponentCategory::W3 => "Depot",
            WealthComponentCategory::W4 => "Bargeld",
            WealthComponentCategory::W5 => "Immobilien",
            WealthComponentCategory::W6 => "Schulden",
            WealthComponentCategory::W7 => "Sonstiges",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Girokonto" => Some(WealthComponentCategory::W1),
            "Sparkonto" => Some(WealthComponentCategory::W2),
            "Depot" => Some(WealthComponentCategory::W3),
            "Bargeld" => Some(WealthComponentCategory::W4),
            "Immobilien" => Some(WealthComponentCategory::W5),
            "Schulden" => Some(WealthComponentCategory::W6),
            "Sonstiges" => Some(WealthComponentCategory::W7),
            _ => None,
        }
    }

    pub fn code(&self) -> &str {
        match self {
            WealthComponentCategory::W1 => "w1",
            WealthComponentCategory::W2 => "w2",
            WealthComponentCategory::W3 => "w3",
            WealthComponentCategory::W4 => "w4",
            WealthComponentCategory::W5 => "w5",
            WealthComponentCategory::W6 => "w6",
            WealthComponentCategory::W7 => "w7",
        }
    }

    pub fn all() -> &'static [WealthComponentCategory] {
        &[
            WealthComponentCategory::W1,
            WealthComponentCategory::W2,
            WealthComponentCategory::W3,
            WealthComponentCategory::W4,
            WealthComponentCategory::W5,
            WealthComponentCategory::W6,
            WealthComponentCategory::W7,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthSnapshot {
    pub id: Option<i64>,
    pub date: NaiveDate,
    pub components: Vec<WealthComponent>,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WealthComponent {
    pub id: Option<i64>,
    pub snapshot_id: Option<i64>,
    pub category: WealthComponentCategory,
    pub amount: f64,
}

// ─── Salary Tracking ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixedSalary {
    pub id: Option<i64>,
    pub effective_date: NaiveDate,
    pub monthly_amount: f64,
    pub payments_per_year: u32, // 12, 13, 14, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableSalary {
    pub id: Option<i64>,
    pub date: NaiveDate,
    pub amount: f64,
    pub description: String,
}
