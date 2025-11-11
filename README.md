# HTMX Rule Builder

A fraud detection rule builder built with Rust and HTMX. This project demonstrates building an AST-based rule engine with a dynamic, server-rendered UI.

## Features

- 🦀 **Rust Backend**: Built with Axum, a modern async web framework
- 🔄 **HTMX Frontend**: Dynamic interactions without JavaScript frameworks
- 🌳 **AST-Based Rules**: Structured rule representation with validation
- 🎨 **Modern UI**: Clean, gradient-based design
- ⚡ **Real-time Updates**: Add/remove conditions dynamically
- ✅ **Validation**: Built-in rule validation with error reporting

## Architecture

### Data Model

The rule builder uses an Abstract Syntax Tree (AST) approach:

- **Rule**: Top-level container with name, description, and conditions
- **Condition**: A single comparison (field, operator, value)
- **Field**: Available fields from the fraud detection system (transaction amount, user country, etc.)
- **Operator**: Comparison operators (equals, greater than, contains, etc.)
- **Logical Operator**: How conditions combine (AND/OR)

### Tech Stack

- **Web Framework**: [Axum](https://github.com/tokio-rs/axum) - Ergonomic and modular web framework
- **Templating**: [Askama](https://github.com/djc/askama) - Type-safe Jinja2-like templates
- **HTMX**: For dynamic interactions
- **Serialization**: Serde for JSON handling
- **Async Runtime**: Tokio

## Getting Started

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))

### Installation

1. Clone the repository (or you're already here!)

2. Build and run:

```bash
cargo run
```

3. Open your browser to [http://localhost:3000](http://localhost:3000)

### Development

For hot-reloading during development, you can use `cargo-watch`:

```bash
cargo install cargo-watch
cargo watch -x run
```

## Usage

### Creating a Rule

1. Click "New Rule" button
2. Enter a name and description
3. Click "Create Rule"

### Adding Conditions

1. Click "View Details" on a rule
2. Click "+ Add Condition"
3. Select a field, operator, and enter a value
4. Click "Add Condition"

### Validating Rules

1. Open a rule's details
2. Click "Validate Rule"
3. See validation results

## Project Structure

```
htmx-builder/
├── src/
│   ├── main.rs           # Application entry point
│   ├── handlers.rs       # HTTP request handlers
│   └── models.rs         # Data structures and business logic
├── templates/            # Askama HTML templates
│   ├── index.html        # Main page
│   ├── rule_view.html    # Rule details
│   ├── condition_form.html
│   └── ...
├── static/
│   └── style.css         # Styling
└── Cargo.toml            # Dependencies
```

## API Endpoints

- `GET /` - Main page
- `GET /rules` - List all rules (HTMX partial)
- `POST /rules` - Create new rule
- `GET /rules/:id` - View rule details
- `POST /rules/:id/validate` - Validate rule
- `GET /rules/:id/conditions/new` - Condition form
- `POST /rules/:id/conditions` - Add condition
- `DELETE /rules/:id/conditions/:condition_id` - Remove condition

## Extending the Project

### Adding New Fields

In `src/models.rs`, add to the `Field` enum:

```rust
pub enum Field {
    // Existing fields...
    YourNewField,
}
```

Update the `as_str()` and `display_name()` methods accordingly.

### Adding Custom Validation

In `src/models.rs`, extend the `Rule::validate()` method:

```rust
impl Rule {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        // Add your validation logic
    }
}
```

### Storage

Currently uses in-memory storage. To add persistence:

1. Replace `RuleStore` with a database client (e.g., SQLx, Diesel)
2. Update handlers to use async database queries
3. Add database connection pool to application state

## Learning Resources

### Rust Web Development

- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

### HTMX

- [HTMX Documentation](https://htmx.org/docs/)
- [HTMX Examples](https://htmx.org/examples/)

### Fraud Detection

- Rule engines and pattern matching
- AST-based expression evaluation

## Future Enhancements

- [ ] Add rule execution/evaluation
- [ ] Support nested conditions (groups)
- [ ] Add more operators (regex, between, etc.)
- [ ] Implement rule versioning
- [ ] Add test data simulation
- [ ] Export rules as JSON
- [ ] Import rules from JSON
- [ ] Add authentication
- [ ] Persist to database
- [ ] Add rule analytics

## License

MIT License - feel free to use this for learning and experimentation!

## Contributing

This is a learning project, but feel free to fork and experiment!
