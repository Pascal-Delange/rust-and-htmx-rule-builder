use crate::auth::get_session_store;
use crate::models::{
    parse_path, ConditionNode, Field, LogicalOperator, Operand, Operator, Rule, RuleStore,
};
use askama::Template;
use axum::{
    extract::Path,
    response::{Html, IntoResponse, Response},
    Form,
};
use serde::Deserialize;
use std::sync::OnceLock;
use uuid::Uuid;

// Global rule store (in a real app, you'd use proper state management)
static RULE_STORE: OnceLock<RuleStore> = OnceLock::new();

fn get_store() -> &'static RuleStore {
    RULE_STORE.get_or_init(|| RuleStore::new())
}

// Templates
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    rule: Rule,
    rule_id: Uuid,
    rule_json: String,
    tree_html: String,
}

#[derive(Template)]
#[template(path = "rule_view.html")]
struct RuleViewTemplate {
    rule: Rule,
    rule_id: Uuid,
    rule_json: String,
    tree_html: String, // Pre-rendered tree HTML
}

#[derive(Template)]
#[template(path = "validation_result.html")]
struct ValidationResultTemplate {
    success: bool,
    errors: Vec<String>,
}

// Handlers
pub async fn index() -> impl IntoResponse {
    let store = get_store();
    if let Some(rule) = store.get_rule() {
        let rule_id = rule.id;
        let tree_html = render_tree_node(&rule.root, "0".to_string(), 0);
        let rule_json = serde_json::to_string_pretty(&rule).unwrap_or_else(|_| "{}".to_string());
        let template = IndexTemplate {
            rule,
            rule_id,
            rule_json,
            tree_html,
        };
        HtmlTemplate(template).into_response()
    } else {
        Html("<div>Rule not found</div>".to_string()).into_response()
    }
}

pub async fn new_condition_form(Path(path): Path<String>) -> impl IntoResponse {
    // Return the form with the path baked into the action
    let fields = Field::all();

    // Build the form HTML with the correct path
    let form_html = format!(
        r##"<div class="card condition-form">
        <h4>Add Condition to Group</h4>
        <form hx-post="/rule/node/{}/add-condition"
              hx-target="#rule-container"
              hx-swap="innerHTML">
            <div class="form-row" x-data="{{ leftFieldType: null }}">
                <div class="form-group">
                    <label>Left Side</label>
                    <div class="operand-selector" x-data="{{ type: 'field' }}">
                        <div class="operand-input-group">
                            <button type="button" 
                                    @click="type = (type === 'field' ? 'value' : 'field')"
                                    class="operand-toggle"
                                    :title="type === 'field' ? 'Switch to value' : 'Switch to field'">
                                <span x-show="type === 'field'">📊</span>
                                <span x-show="type === 'value'">✏️</span>
                            </button>
                            <div class="operand-input">
                                <select x-show="type === 'field'" name="left_field" :required="type === 'field'" :disabled="type !== 'field'" x-cloak
                                        hx-get="/rule/conditions/operators-and-right" hx-target="#operator-group" hx-swap="innerHTML"
                                        hx-include="[name='left_type'], [name='left_field']"
                                        @change="const numericFields = ['transaction_amount', 'user_age', 'transaction_count24h', 'account_age'];
                                                 leftFieldType = numericFields.includes($event.target.value) ? 'number' : 'string';">
                                    <option value="">Select a field...</option>
                                    {}
                                </select>
                                <input x-show="type === 'value'" type="text" name="left_value" placeholder="Enter value..."
                                       :required="type === 'value'" :disabled="type !== 'value'" x-cloak
                                       hx-get="/rule/conditions/operators-for-value" hx-target="#operator-group" hx-swap="innerHTML"
                                       hx-trigger="input changed delay:200ms" hx-include="[name='left_type'], [name='left_value']"
                                       @input="const val = $event.target.value; if (val === '') {{ leftFieldType = null; }} 
                                               else if (!isNaN(val) && val.trim() !== '') {{ leftFieldType = 'number'; }} 
                                               else {{ leftFieldType = 'string'; }}">
                            </div>
                        </div>
                        <input type="hidden" name="left_type" :value="type">
                    </div>
                </div>
                
                <div class="form-group" id="operator-group">
                    <label for="operator">Operator</label>
                    <select id="operator" name="operator" required>
                        <option value="">Select left side first...</option>
                    </select>
                </div>
                
                <div class="form-group" id="right-side-group">
                    <label>Right Side</label>
                    <p class="hint" x-show="leftFieldType === 'number'" x-cloak style="font-size: 0.85em; color: #666; margin-bottom: 0.5rem;">
                        💡 Tip: Use a number value or another numeric field
                    </p>
                    <p class="hint" x-show="leftFieldType === 'string'" x-cloak style="font-size: 0.85em; color: #666; margin-bottom: 0.5rem;">
                        💡 Tip: Use a text value or another text field
                    </p>
                    <div class="operand-selector" x-data="{{ type: 'field' }}">
                        <div class="operand-input-group">
                            <button type="button" 
                                    @click="type = (type === 'field' ? 'value' : 'field')"
                                    class="operand-toggle"
                                    :title="type === 'field' ? 'Switch to value' : 'Switch to field'">
                                <span x-show="type === 'field'">📊</span>
                                <span x-show="type === 'value'">✏️</span>
                            </button>
                            <div class="operand-input">
                                <select x-show="type === 'field'" name="right_field" :required="type === 'field'" :disabled="type !== 'field'" x-cloak
                                        x-show="!leftFieldType || (leftFieldType === 'number' && ['transaction_amount', 'user_age', 'transaction_count24h', 'account_age'].includes('{{{{ field.as_str() }}}}')) || (leftFieldType === 'string' && !['transaction_amount', 'user_age', 'transaction_count24h', 'account_age'].includes('{{{{ field.as_str() }}}}'))">
                                    <option value="">Select a field...</option>
                                    {}
                                </select>
                                <input x-show="type === 'value' && leftFieldType === 'number'" type="number" name="right_value" placeholder="Enter a number..."
                                       step="any" :required="type === 'value' && leftFieldType === 'number'" :disabled="type !== 'value' || leftFieldType !== 'number'" x-cloak>
                                <input x-show="type === 'value' && leftFieldType !== 'number'" type="text" name="right_value" placeholder="Enter value..."
                                       :required="type === 'value' && leftFieldType !== 'number'" :disabled="type !== 'value' || leftFieldType === 'number'" x-cloak>
                            </div>
                        </div>
                        <input type="hidden" name="right_type" :value="type">
                    </div>
                </div>
            </div>
            
            <div class="form-actions">
                <button type="submit" class="btn btn-primary">Add Condition</button>
                <button type="button" class="btn btn-secondary" onclick="this.closest('.card').innerHTML = ''">Cancel</button>
            </div>
        </form>
    </div>"##,
        path,
        fields
            .iter()
            .map(|f| format!(
                r#"<option value="{}">{}</option>"#,
                f.as_str(),
                f.display_name()
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        fields
            .iter()
            .map(|f| format!(
                r#"<option value="{}">{}</option>"#,
                f.as_str(),
                f.display_name()
            ))
            .collect::<Vec<_>>()
            .join("\n"),
    );

    Html(form_html).into_response()
}

/// Render a tree node recursively
fn render_tree_node(node: &ConditionNode, path: String, depth: usize) -> String {
    let indent = depth * 20;

    match node {
        ConditionNode::Leaf {
            left,
            operator,
            right,
            ..
        } => {
            let left_display = left.display();
            let operator_display = operator.display_name();
            let right_display = right.display();

            format!(
                r##"<div id="node-{path}" class="condition-leaf" style="margin-left: {indent}px">
                    <div class="condition-content">
                        <span class="condition-field">{left_display}</span>
                        <span class="condition-operator">{operator_display}</span>
                        <span class="condition-value">{right_display}</span>
                    </div>
                    <button class="btn-delete"
                            hx-delete="/rule/node/{path}"
                            hx-target="#rule-container"
                            hx-swap="innerHTML"
                            hx-confirm="Delete this condition?">✕</button>
                </div>"##,
                path = path,
                indent = indent,
                left_display = left_display,
                operator_display = operator_display,
                right_display = right_display,
            )
        }
        ConditionNode::Group {
            operator, children, ..
        } => {
            let children_html: String = children
                .iter()
                .enumerate()
                .map(|(i, child)| render_tree_node(child, format!("{}-{}", path, i), depth + 1))
                .collect::<Vec<_>>()
                .join("\n");

            let and_sel = if matches!(operator, LogicalOperator::And) {
                "selected"
            } else {
                ""
            };
            let or_sel = if matches!(operator, LogicalOperator::Or) {
                "selected"
            } else {
                ""
            };

            let delete_btn = if path == "0" {
                String::new() // Can't delete root
            } else {
                format!(
                    r##"<button class="btn-delete"
                        hx-delete="/rule/node/{}"
                        hx-target="#rule-container"
                        hx-swap="innerHTML"
                        hx-confirm="Delete this group?">✕</button>"##,
                    path
                )
            };

            format!(
                r##"<div id="node-{path}" class="condition-group" style="margin-left: {indent}px">
                    <div class="group-header">
                        <select class="group-operator"
                                hx-post="/rule/node/{path}/operator"
                                hx-target="#rule-container"
                                hx-swap="innerHTML"
                                name="operator">
                            <option value="and" {and_sel}>AND</option>
                            <option value="or" {or_sel}>OR</option>
                        </select>
                        {delete_btn}
                    </div>
                    <div class="group-children">
                        {children_html}
                    </div>
                    <div class="group-actions">
                        <button class="btn btn-small btn-primary"
                                hx-get="/rule/node/{path}/add-condition-form"
                                hx-target="#condition-form-container"
                                hx-swap="innerHTML">
                            + Add Condition
                        </button>
                        <button class="btn btn-small btn-secondary"
                                hx-post="/rule/node/{path}/add-group"
                                hx-target="#rule-container"
                                hx-swap="innerHTML">
                            + Add Group
                        </button>
                    </div>
                </div>"##,
                path = path,
                indent = indent,
                and_sel = and_sel,
                or_sel = or_sel,
                delete_btn = delete_btn,
                children_html = children_html,
            )
        }
    }
}

#[derive(Deserialize)]
pub struct AddConditionForm {
    left_type: String,
    left_field: Option<String>,
    left_value: Option<String>,
    operator: String,
    right_type: String,
    right_field: Option<String>,
    right_value: Option<String>,
}

pub async fn add_condition(
    Path(path): Path<String>,
    Form(form): Form<AddConditionForm>,
) -> Response {
    let store = get_store();

    if let Some(mut rule) = store.get_rule() {
        let operator: Operator = serde_json::from_str(&format!("\"{}\"", form.operator)).unwrap();

        // Parse left operand
        let left = if form.left_type == "field" {
            let field: Field =
                serde_json::from_str(&format!("\"{}\"", form.left_field.unwrap_or_default()))
                    .unwrap();
            Operand::Field { field }
        } else {
            Operand::Value {
                value: form.left_value.unwrap_or_default(),
            }
        };

        // Parse right operand
        let right = if form.right_type == "field" {
            let field: Field =
                serde_json::from_str(&format!("\"{}\"", form.right_field.unwrap_or_default()))
                    .unwrap();
            Operand::Field { field }
        } else {
            Operand::Value {
                value: form.right_value.unwrap_or_default(),
            }
        };

        let condition = ConditionNode::Leaf {
            id: Uuid::new_v4(),
            left,
            operator,
            right,
        };

        // Add to tree at path
        let indices = parse_path(&path);
        rule.root.add_child_at_path(&indices, condition);

        let rule_id = rule.id;
        store.update_rule(rule.clone());

        // Re-render the entire rule view
        let tree_html = render_tree_node(&rule.root, "0".to_string(), 0);
        let rule_json = serde_json::to_string_pretty(&rule).unwrap_or_else(|_| "{}".to_string());
        let template = RuleViewTemplate {
            rule,
            rule_id,
            rule_json,
            tree_html,
        };
        HtmlTemplate(template).into_response()
    } else {
        Html("<div>Rule not found</div>".to_string()).into_response()
    }
}

pub async fn delete_node(Path(path): Path<String>) -> Response {
    let store = get_store();

    if let Some(mut rule) = store.get_rule() {
        let rule_id = rule.id;

        // Delete node at path
        let indices = parse_path(&path);
        rule.root.delete_at_path(&indices);
        store.update_rule(rule.clone());

        // Re-render the entire rule view
        let tree_html = render_tree_node(&rule.root, "0".to_string(), 0);
        let rule_json = serde_json::to_string_pretty(&rule).unwrap_or_else(|_| "{}".to_string());
        let template = RuleViewTemplate {
            rule,
            rule_id,
            rule_json,
            tree_html,
        };
        HtmlTemplate(template).into_response()
    } else {
        Html("").into_response()
    }
}

pub async fn add_group(Path(path): Path<String>) -> Response {
    let store = get_store();

    if let Some(mut rule) = store.get_rule() {
        let rule_id = rule.id;

        // Create new group
        let new_group = ConditionNode::Group {
            id: Uuid::new_v4(),
            operator: LogicalOperator::And,
            children: vec![],
        };

        // Add to tree at path
        let indices = parse_path(&path);
        rule.root.add_child_at_path(&indices, new_group);
        store.update_rule(rule.clone());

        // Re-render the entire rule view
        let tree_html = render_tree_node(&rule.root, "0".to_string(), 0);
        let rule_json = serde_json::to_string_pretty(&rule).unwrap_or_else(|_| "{}".to_string());
        let template = RuleViewTemplate {
            rule,
            rule_id,
            rule_json,
            tree_html,
        };
        HtmlTemplate(template).into_response()
    } else {
        Html("").into_response()
    }
}

pub async fn update_operator(
    Path(path): Path<String>,
    Form(form): Form<std::collections::HashMap<String, String>>,
) -> Response {
    let store = get_store();

    if let Some(mut rule) = store.get_rule() {
        let rule_id = rule.id;

        // Get the operator
        let operator_str = form.get("operator").map(|s| s.as_str()).unwrap_or("and");
        let operator = if operator_str == "or" {
            LogicalOperator::Or
        } else {
            LogicalOperator::And
        };

        // Update operator at path
        let indices = parse_path(&path);
        if let Some(ConditionNode::Group {
            operator: ref mut op,
            ..
        }) = rule.root.get_at_path_mut(&indices)
        {
            *op = operator;
        }

        store.update_rule(rule.clone());

        // Re-render the entire rule view
        let tree_html = render_tree_node(&rule.root, "0".to_string(), 0);
        let rule_json = serde_json::to_string_pretty(&rule).unwrap_or_else(|_| "{}".to_string());
        let template = RuleViewTemplate {
            rule,
            rule_id,
            rule_json,
            tree_html,
        };
        HtmlTemplate(template).into_response()
    } else {
        Html("").into_response()
    }
}

#[derive(Deserialize)]
pub struct FieldQuery {
    field: String,
}

pub async fn get_operators_for_field(
    axum::extract::Query(query): axum::extract::Query<FieldQuery>,
) -> Response {
    let field_str = &query.field;

    // Parse the field to determine which operators are valid
    let operators = if let Ok(field) = serde_json::from_str::<Field>(&format!("\"{}\"", field_str))
    {
        match field {
            // Numeric fields: comparison operators
            Field::TransactionAmount
            | Field::UserAge
            | Field::TransactionCount24h
            | Field::AccountAge => {
                vec![
                    Operator::Equals,
                    Operator::NotEquals,
                    Operator::GreaterThan,
                    Operator::LessThan,
                    Operator::GreaterThanOrEqual,
                    Operator::LessThanOrEqual,
                ]
            }
            // String fields: equality and contains
            Field::TransactionCurrency
            | Field::UserCountry
            | Field::IpAddress
            | Field::DeviceFingerprint => {
                vec![
                    Operator::Equals,
                    Operator::NotEquals,
                    Operator::Contains,
                    Operator::In,
                ]
            }
        }
    } else {
        Operator::all()
    };

    let options_html = operators
        .iter()
        .map(|op| {
            format!(
                r#"<option value="{}">{}</option>"#,
                op.as_str(),
                op.display_name()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(
        r##"<label for="operator">Operator</label>
<select 
    id="operator" 
    name="operator" 
    required
    hx-get="/rule/conditions/value-input"
    hx-target="#value-group"
    hx-include="[name='field'], [name='operator']">
    <option value="">Select an operator...</option>
    {}
</select>"##,
        options_html
    );

    Html(html).into_response()
}

#[derive(Deserialize)]
pub struct ValueInputQuery {
    field: String,
    operator: String,
}

pub async fn get_value_input_for_field(
    axum::extract::Query(query): axum::extract::Query<ValueInputQuery>,
) -> Response {
    let field_str = &query.field;
    let _operator_str = &query.operator;

    // Determine the appropriate input type based on the field
    let html = if let Ok(field) = serde_json::from_str::<Field>(&format!("\"{}\"", field_str)) {
        match field {
            // Numeric fields: number input
            Field::TransactionAmount
            | Field::UserAge
            | Field::TransactionCount24h
            | Field::AccountAge => r#"<label for="value">Value</label>
<input 
    type="number" 
    id="value" 
    name="value" 
    placeholder="Enter a number..."
    step="any"
    required>"#
                .to_string(),
            // String fields: text input with suggestions
            Field::TransactionCurrency => r#"<label for="value">Value</label>
<input 
    type="text" 
    id="value" 
    name="value" 
    placeholder="e.g., USD, EUR, GBP..."
    list="currency-suggestions"
    required>
<datalist id="currency-suggestions">
    <option value="USD">
    <option value="EUR">
    <option value="GBP">
    <option value="JPY">
</datalist>"#
                .to_string(),
            Field::UserCountry => r#"<label for="value">Value</label>
<input 
    type="text" 
    id="value" 
    name="value" 
    placeholder="e.g., US, GB, FR..."
    list="country-suggestions"
    required>
<datalist id="country-suggestions">
    <option value="US">
    <option value="GB">
    <option value="FR">
    <option value="DE">
</datalist>"#
                .to_string(),
            // Default: text input
            _ => r#"<label for="value">Value</label>
<input 
    type="text" 
    id="value" 
    name="value" 
    placeholder="Enter value..."
    required>"#
                .to_string(),
        }
    } else {
        r#"<label for="value">Value</label>
<input 
    type="text" 
    id="value" 
    name="value" 
    placeholder="Select a field first..."
    required
    disabled>"#
            .to_string()
    };

    Html(html).into_response()
}

pub async fn get_operators_and_right_hint(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let left_type = params
        .get("left_type")
        .map(|s| s.as_str())
        .unwrap_or("field");
    let left_field_str = params.get("left_field").map(|s| s.as_str()).unwrap_or("");

    // Determine operators based on left side
    let operators = if left_type == "field" && !left_field_str.is_empty() {
        if let Ok(field) = serde_json::from_str::<Field>(&format!("\"{}\"", left_field_str)) {
            match field {
                // Numeric fields: comparison operators
                Field::TransactionAmount
                | Field::UserAge
                | Field::TransactionCount24h
                | Field::AccountAge => {
                    vec![
                        Operator::Equals,
                        Operator::NotEquals,
                        Operator::GreaterThan,
                        Operator::LessThan,
                        Operator::GreaterThanOrEqual,
                        Operator::LessThanOrEqual,
                    ]
                }
                // String fields: equality and contains
                Field::TransactionCurrency
                | Field::UserCountry
                | Field::IpAddress
                | Field::DeviceFingerprint => {
                    vec![
                        Operator::Equals,
                        Operator::NotEquals,
                        Operator::Contains,
                        Operator::In,
                    ]
                }
            }
        } else {
            Operator::all()
        }
    } else {
        Operator::all()
    };

    let options_html = operators
        .iter()
        .map(|op| {
            format!(
                r#"<option value="{}">{}</option>"#,
                op.as_str(),
                op.display_name()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(
        r##"<label for="operator">Operator</label>
<select id="operator" name="operator" required>
    <option value="">Select an operator...</option>
    {}
</select>"##,
        options_html
    );

    Html(html).into_response()
}

pub async fn get_operators_for_value(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    let left_value = params.get("left_value").map(|s| s.as_str()).unwrap_or("");

    // Determine if the value is numeric or string
    let is_numeric = !left_value.is_empty() && left_value.parse::<f64>().is_ok();

    let operators = if is_numeric {
        // Numeric value: comparison operators
        vec![
            Operator::Equals,
            Operator::NotEquals,
            Operator::GreaterThan,
            Operator::LessThan,
            Operator::GreaterThanOrEqual,
            Operator::LessThanOrEqual,
        ]
    } else {
        // String value: equality and contains
        vec![
            Operator::Equals,
            Operator::NotEquals,
            Operator::Contains,
            Operator::In,
        ]
    };

    let options_html = operators
        .iter()
        .map(|op| {
            format!(
                r#"<option value="{}">{}</option>"#,
                op.as_str(),
                op.display_name()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let html = format!(
        r##"<label for="operator">Operator</label>
<select id="operator" name="operator" required>
    <option value="">Select an operator...</option>
    {}
</select>"##,
        options_html
    );

    Html(html).into_response()
}

pub async fn validate_rule() -> Response {
    let store = get_store();

    if let Some(rule) = store.get_rule() {
        let result = rule.validate();
        let template = match result {
            Ok(_) => ValidationResultTemplate {
                success: true,
                errors: vec![],
            },
            Err(errors) => ValidationResultTemplate {
                success: false,
                errors,
            },
        };
        HtmlTemplate(template).into_response()
    } else {
        Html("<div>Rule not found</div>".to_string()).into_response()
    }
}

// ============================================================================
// Auth Handlers
// ============================================================================

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate;

pub async fn login_page() -> impl IntoResponse {
    let template = LoginTemplate;
    HtmlTemplate(template)
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

pub async fn do_login(Form(form): Form<LoginForm>) -> Response {
    // Fake auth - accept any non-empty username/password
    if !form.username.is_empty() && !form.password.is_empty() {
        // Create session
        let store = get_session_store();
        let session_id = store.create_session(form.username);

        // Set cookie and redirect using HX-Redirect for HTMX
        axum::response::Response::builder()
            .status(200)
            .header(
                "Set-Cookie",
                format!("session_id={}; Path=/; HttpOnly; SameSite=Lax", session_id),
            )
            .header("HX-Redirect", "/")
            .body(axum::body::Body::empty())
            .unwrap()
    } else {
        // Return error message
        Html(r#"<div class="error">Please enter both username and password</div>"#).into_response()
    }
}

pub async fn logout() -> Response {
    // Clear cookie and redirect using HX-Redirect for HTMX
    axum::response::Response::builder()
        .status(200)
        .header(
            "Set-Cookie",
            "session_id=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
        )
        .header("HX-Redirect", "/login")
        .body(axum::body::Body::empty())
        .unwrap()
}

// Helper for rendering Askama templates
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                tracing::error!("Template error: {}", err);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to render template. Error: {}", err),
                )
                    .into_response()
            }
        }
    }
}
