# Quick Start Guide

## 1. Install Rust (if you haven't already)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the prompts, then reload your shell or run:

```bash
source $HOME/.cargo/env
```

## 2. Run the Application

```bash
cargo run
```

The first run will take a few minutes to download and compile dependencies.

## 3. Open Your Browser

Navigate to: **http://localhost:3000**

## 4. Try It Out!

### Create Your First Rule

1. Click the **"+ New Rule"** button
2. Enter:
   - Name: "High Value Transaction"
   - Description: "Flag transactions over $10,000"
3. Click **"Create Rule"**

### Add Conditions

1. Click **"View Details"** on your rule
2. Click **"+ Add Condition"**
3. Select:
   - Field: **Transaction Amount**
   - Operator: **Greater Than**
   - Value: **10000**
4. Click **"Add Condition"**

### Add More Conditions

Try adding another condition:

- Field: **User Country**
- Operator: **In**
- Value: **US,CA,UK**

### Validate Your Rule

1. Scroll down in the rule details
2. Click **"Validate Rule"**
3. See the validation result

### View the AST

Scroll down to see the JSON representation of your rule's Abstract Syntax Tree!

## What to Explore Next

- Try different field types (amounts, countries, IPs, etc.)
- Create multiple rules
- Test validation by leaving fields empty
- Look at the AST JSON to understand the data structure
- Modify the code to add new fields or operators

## Understanding the HTMX Magic

Notice how the page updates without full page reloads? That's HTMX!

- Open your browser's **Network tab**
- Add a condition and watch the requests
- You'll see HTML fragments being swapped in dynamically

## Development Tips

### Auto-reload on Changes

```bash
cargo install cargo-watch
cargo watch -x run
```

Now changes to your Rust code will automatically rebuild and restart the server.

### Check for Compilation Errors

```bash
cargo check
```

### Run Tests (when you add them)

```bash
cargo test
```

## Next Steps for Learning

1. **Add a new field**: Edit `src/models.rs` and add a new field type
2. **Add a new operator**: Extend the `Operator` enum
3. **Improve validation**: Add more validation rules in `Rule::validate()`
4. **Add nested conditions**: Support grouping conditions with parentheses
5. **Add persistence**: Replace in-memory storage with SQLite

## Troubleshooting

### Port 3000 already in use?

Edit `src/main.rs` line 31 and change:

```rust
let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
```

to use a different port like 3001.

### Compilation errors?

Make sure you have the latest Rust:

```bash
rustup update
```

### Templates not found?

Make sure you're running `cargo run` from the project root directory.

---

**Happy coding! 🦀✨**
