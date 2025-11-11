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

/// Represents either a field reference or a literal value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Operand {
    Field { field: Field },
    Value { value: String },
}

impl Operand {
    pub fn display(&self) -> String {
        match self {
            Operand::Field { field } => field.display_name().to_string(),
            Operand::Value { value } => format!("\"{}\"", value),
        }
    }
}

/// A node in the condition tree - either a leaf (condition) or a group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConditionNode {
    Leaf {
        id: Uuid,
        left: Operand,
        operator: Operator,
        right: Operand,
    },
    Group {
        id: Uuid,
        operator: LogicalOperator,
        children: Vec<ConditionNode>,
    },
}

impl ConditionNode {
    pub fn id(&self) -> Uuid {
        match self {
            ConditionNode::Leaf { id, .. } => *id,
            ConditionNode::Group { id, .. } => *id,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, ConditionNode::Leaf { .. })
    }

    pub fn is_group(&self) -> bool {
        matches!(self, ConditionNode::Group { .. })
    }

    /// Navigate to a node at the given path
    pub fn get_at_path(&self, path: &[usize]) -> Option<&ConditionNode> {
        if path.is_empty() {
            return Some(self);
        }

        match self {
            ConditionNode::Group { children, .. } => children
                .get(path[0])
                .and_then(|child| child.get_at_path(&path[1..])),
            ConditionNode::Leaf { .. } => None,
        }
    }

    /// Navigate to a mutable node at the given path
    pub fn get_at_path_mut(&mut self, path: &[usize]) -> Option<&mut ConditionNode> {
        if path.is_empty() {
            return Some(self);
        }

        match self {
            ConditionNode::Group { children, .. } => children
                .get_mut(path[0])
                .and_then(|child| child.get_at_path_mut(&path[1..])),
            ConditionNode::Leaf { .. } => None,
        }
    }

    /// Add a child to a group at the given path
    pub fn add_child_at_path(&mut self, path: &[usize], child: ConditionNode) -> bool {
        if let Some(ConditionNode::Group { children, .. }) = self.get_at_path_mut(path) {
            children.push(child);
            true
        } else {
            false
        }
    }

    /// Delete a node at the given path
    pub fn delete_at_path(&mut self, path: &[usize]) -> bool {
        if path.is_empty() {
            return false; // Can't delete root
        }

        let parent_path = &path[..path.len() - 1];
        let child_index = path[path.len() - 1];

        if let Some(ConditionNode::Group { children, .. }) = self.get_at_path_mut(parent_path) {
            if child_index < children.len() {
                children.remove(child_index);
                return true;
            }
        }
        false
    }
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

/// The main rule structure - represents an AST with tree-based conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub root: ConditionNode, // Tree structure
    pub action: String,
}

impl Rule {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            root: ConditionNode::Group {
                id: Uuid::new_v4(),
                operator: LogicalOperator::And,
                children: Vec::new(),
            },
            action: String::from("flag_for_review"),
        }
    }

    /// Validate the rule
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("Rule name cannot be empty".to_string());
        }

        // Validate the tree
        self.validate_node(&self.root, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_node(&self, node: &ConditionNode, errors: &mut Vec<String>) {
        match node {
            ConditionNode::Leaf { left, right, .. } => {
                // Validate that value operands are not empty
                if let Operand::Value { value } = left {
                    if value.is_empty() {
                        errors.push("Condition value cannot be empty".to_string());
                    }
                }
                if let Operand::Value { value } = right {
                    if value.is_empty() {
                        errors.push("Condition value cannot be empty".to_string());
                    }
                }
            }
            ConditionNode::Group { children, .. } => {
                if children.is_empty() {
                    errors.push("Group must have at least one condition".to_string());
                }
                // Recursively validate children
                for child in children {
                    self.validate_node(child, errors);
                }
            }
        }
    }
}

/// Parse a path string like "0-1-2" into indices [1, 2]
/// The first "0" is always the root, so we skip it
pub fn parse_path(path: &str) -> Vec<usize> {
    if path == "0" {
        return vec![];
    }

    path.split('-')
        .skip(1) // Skip the root "0"
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// Convert indices back to path string
pub fn path_to_string(indices: &[usize]) -> String {
    if indices.is_empty() {
        return "0".to_string();
    }

    let mut result = String::from("0");
    for idx in indices {
        result.push('-');
        result.push_str(&idx.to_string());
    }
    result
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
