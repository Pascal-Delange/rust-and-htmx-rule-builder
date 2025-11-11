use crate::models::{Condition, Field, Operator, Rule, RuleStore};
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
    rule_id: Uuid,
    fields: Vec<Field>,
    operators: Vec<Operator>,
}

#[derive(Template)]
#[template(path = "condition_row.html")]
struct ConditionRowTemplate {
    rule_id: Uuid,
    condition: Condition,
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
    let store = get_store();
    if let Some(rule) = store.get_rule() {
        let template = ConditionFormTemplate {
            rule_id: rule.id,
            fields: Field::all(),
            operators: Operator::all(),
        };
        HtmlTemplate(template).into_response()
    } else {
        Html("<div>Rule not found</div>".to_string()).into_response()
    }
}

#[derive(Deserialize)]
pub struct AddConditionForm {
    field: String,
    operator: String,
    value: String,
}

pub async fn add_condition(Form(form): Form<AddConditionForm>) -> Response {
    let store = get_store();

    if let Some(mut rule) = store.get_rule() {
        let field: Field = serde_json::from_str(&format!("\"{}\"", form.field)).unwrap();
        let operator: Operator = serde_json::from_str(&format!("\"{}\"", form.operator)).unwrap();

        let condition = Condition {
            id: Uuid::new_v4(),
            field,
            operator,
            value: form.value,
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

pub async fn view_rule() -> Response {
    let store = get_store();
    if let Some(rule) = store.get_rule() {
        let rule_id = rule.id;
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
