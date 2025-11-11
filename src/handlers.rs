use crate::auth::get_session_store;
use crate::models::{Condition, Field, Operand, Operator, Rule, RuleStore};
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
}

#[derive(Template)]
#[template(path = "rule_view.html")]
struct RuleViewTemplate {
    rule: Rule,
    rule_id: Uuid,
    rule_json: String,
}

#[derive(Template)]
#[template(path = "condition_form.html")]
struct ConditionFormTemplate {
    fields: Vec<Field>,
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
        let rule_json = serde_json::to_string_pretty(&rule).unwrap_or_else(|_| "{}".to_string());
        let template = IndexTemplate {
            rule,
            rule_id,
            rule_json,
        };
        HtmlTemplate(template).into_response()
    } else {
        Html("<div>Rule not found</div>".to_string()).into_response()
    }
}

pub async fn new_condition_form() -> impl IntoResponse {
    let template = ConditionFormTemplate {
        fields: Field::all(),
    };
    HtmlTemplate(template).into_response()
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

pub async fn add_condition(Form(form): Form<AddConditionForm>) -> Response {
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

        let condition = Condition {
            id: Uuid::new_v4(),
            left,
            operator,
            right,
        };

        let rule_id = rule.id;
        rule.conditions.push(condition.clone());
        store.update_rule(rule.clone());

        // Simple approach: Re-render the entire rule view
        // This is the HTMX best practice for most cases
        let rule_json = serde_json::to_string_pretty(&rule).unwrap_or_else(|_| "{}".to_string());
        let template = RuleViewTemplate {
            rule,
            rule_id,
            rule_json,
        };
        HtmlTemplate(template).into_response()
    } else {
        Html("<div>Rule not found</div>".to_string()).into_response()
    }
}

pub async fn delete_condition(Path(condition_id): Path<Uuid>) -> Response {
    let store = get_store();

    if let Some(mut rule) = store.get_rule() {
        let rule_id = rule.id;
        rule.conditions.retain(|c| c.id != condition_id);
        store.update_rule(rule.clone());

        // Simple approach: Re-render the entire rule view
        let rule_json = serde_json::to_string_pretty(&rule).unwrap_or_else(|_| "{}".to_string());
        let template = RuleViewTemplate {
            rule,
            rule_id,
            rule_json,
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
