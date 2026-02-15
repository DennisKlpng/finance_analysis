use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TypeCategory {
    A,
    B,
    C,
}

impl TypeCategory {
    pub fn as_str(&self) -> &str {
        match self {
            TypeCategory::A => "A",
            TypeCategory::B => "B",
            TypeCategory::C => "C",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "A" => Some(TypeCategory::A),
            "B" => Some(TypeCategory::B),
            "C" => Some(TypeCategory::C),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NecessityCategory {
    D,
    E,
    F,
}

impl NecessityCategory {
    pub fn as_str(&self) -> &str {
        match self {
            NecessityCategory::D => "D",
            NecessityCategory::E => "E",
            NecessityCategory::F => "F",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "D" => Some(NecessityCategory::D),
            "E" => Some(NecessityCategory::E),
            "F" => Some(NecessityCategory::F),
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
