use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a field in the fraud detection system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    TransactionAmount,
    TransactionCurrency,
    UserCountry,
    UserAge,
    IpAddress,
    DeviceFingerprint,
    #[serde(rename = "transaction_count_24h")]
    TransactionCount24h,
    AccountAge,
}

impl Field {
    pub fn all() -> Vec<Field> {
        vec![
            Field::TransactionAmount,
            Field::TransactionCurrency,
            Field::UserCountry,
            Field::UserAge,
            Field::IpAddress,
            Field::DeviceFingerprint,
            Field::TransactionCount24h,
            Field::AccountAge,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            Field::TransactionAmount => "transaction_amount",
            Field::TransactionCurrency => "transaction_currency",
            Field::UserCountry => "user_country",
            Field::UserAge => "user_age",
            Field::IpAddress => "ip_address",
            Field::DeviceFingerprint => "device_fingerprint",
            Field::TransactionCount24h => "transaction_count_24h",
            Field::AccountAge => "account_age",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Field::TransactionAmount => "Transaction Amount",
            Field::TransactionCurrency => "Transaction Currency",
            Field::UserCountry => "User Country",
            Field::UserAge => "User Age",
            Field::IpAddress => "IP Address",
            Field::DeviceFingerprint => "Device Fingerprint",
            Field::TransactionCount24h => "Transaction Count (24h)",
            Field::AccountAge => "Account Age",
        }
    }
}

/// Operators for comparisons
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Contains,
    In,
}

impl Operator {
    pub fn all() -> Vec<Operator> {
        vec![
            Operator::Equals,
            Operator::NotEquals,
            Operator::GreaterThan,
            Operator::LessThan,
            Operator::GreaterThanOrEqual,
            Operator::LessThanOrEqual,
            Operator::Contains,
            Operator::In,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            Operator::Equals => "equals",
            Operator::NotEquals => "not_equals",
            Operator::GreaterThan => "greater_than",
            Operator::LessThan => "less_than",
            Operator::GreaterThanOrEqual => "greater_than_or_equal",
            Operator::LessThanOrEqual => "less_than_or_equal",
            Operator::Contains => "contains",
            Operator::In => "in",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Operator::Equals => "Equals",
            Operator::NotEquals => "Not Equals",
            Operator::GreaterThan => "Greater Than",
            Operator::LessThan => "Less Than",
            Operator::GreaterThanOrEqual => "Greater Than or Equal",
            Operator::LessThanOrEqual => "Less Than or Equal",
            Operator::Contains => "Contains",
            Operator::In => "In",
        }
    }
}

/// A single condition in the rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub id: Uuid,
    pub field: Field,
    pub operator: Operator,
    pub value: String,
}

/// Logical operator for combining conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogicalOperator {
    And,
    Or,
}

impl std::fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogicalOperator::And => write!(f, "AND"),
            LogicalOperator::Or => write!(f, "OR"),
        }
    }
}

/// The main rule structure - represents an AST
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub conditions: Vec<Condition>,
    pub logical_operator: LogicalOperator,
    pub action: String,
}

impl Rule {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            conditions: Vec::new(),
            logical_operator: LogicalOperator::And,
            action: String::from("flag_for_review"),
        }
    }

    /// Validate the rule
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("Rule name cannot be empty".to_string());
        }

        if self.conditions.is_empty() {
            errors.push("Rule must have at least one condition".to_string());
        }

        // Validate each condition
        for condition in &self.conditions {
            if condition.value.is_empty() {
                errors.push(format!(
                    "Condition value for {} cannot be empty",
                    condition.field.display_name()
                ));
            }

            // Type-specific validation
            match condition.field {
                Field::TransactionAmount
                | Field::UserAge
                | Field::TransactionCount24h
                | Field::AccountAge => {
                    if condition.value.parse::<f64>().is_err() {
                        errors.push(format!(
                            "{} must be a number",
                            condition.field.display_name()
                        ));
                    }
                }
                _ => {}
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// In-memory storage (in a real app, this would be a database)
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct RuleStore {
    rule: Arc<Mutex<Option<Rule>>>,
}

impl RuleStore {
    pub fn new() -> Self {
        // Initialize with a default rule
        let default_rule = Rule::new(
            "Fraud Detection Rule".to_string(),
            "Main fraud detection rule for transactions".to_string(),
        );
        Self {
            rule: Arc::new(Mutex::new(Some(default_rule))),
        }
    }

    pub fn get_rule(&self) -> Option<Rule> {
        self.rule.lock().unwrap().clone()
    }

    pub fn update_rule(&self, rule: Rule) {
        let mut r = self.rule.lock().unwrap();
        *r = Some(rule);
    }
}
