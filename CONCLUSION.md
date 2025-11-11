# HTMX Fraud Rule Builder - Conclusion

## Project Overview

Built a fraud detection rule builder with nested conditions using HTMX, Rust/Axum, and Alpine.js. Started simple, progressively added complexity to test HTMX's limits.

**Final Feature Set:**

- ✅ Tree-based nested rule groups (AND/OR)
- ✅ Dynamic dependent dropdowns
- ✅ Field/value operand selection
- ✅ Type-aware inputs (number vs string)
- ✅ Session-based authentication
- ✅ Path-based tree navigation
- ✅ Full CRUD operations on tree nodes

---

## What HTMX Did Well

### 1. **Simple CRUD Operations**

**Excellent for:**

- List/view/delete operations
- Form submissions
- Basic page updates

**Example:**

```html
<button
  hx-delete="/rule/node/0-1"
  hx-target="#rule-container"
  hx-confirm="Delete?"
>
  Delete
</button>
```

**Verdict:** ⭐⭐⭐⭐⭐ Perfect. Clean, declarative, minimal code.

---

### 2. **Server-Side Rendering Benefits**

**Advantages:**

- No JavaScript bundle to download
- SEO-friendly (full HTML from server)
- Simple mental model for basic interactions
- Works without JavaScript (progressive enhancement)

**Verdict:** ⭐⭐⭐⭐⭐ For content-heavy apps, this is ideal.

---

### 3. **Rapid Prototyping (Initially)**

**First 2 hours:**

- Basic rule list: 30 minutes
- Add/delete conditions: 45 minutes
- Validation: 15 minutes
- Auth system: 30 minutes

**Verdict:** ⭐⭐⭐⭐⭐ Extremely fast for simple features.

---

## What HTMX Struggled With

### 1. **Dependent Dropdowns / Complex Forms**

**The Problem:**
Needed left field selection to affect:

- Available operators
- Right field options
- Input type (number vs text)

**HTMX Solution:**

```html
<select
  hx-get="/operators-for-field"
  hx-target="#operator-group"
  hx-include="[name='left_field']"
></select>
```

**Issues:**

- ❌ Multiple server round-trips
- ❌ String-based targeting (`#operator-group`)
- ❌ No type safety
- ❌ Hard to debug (which request updated what?)
- ⚠️ Had to add Alpine.js for client-side state

**Verdict:** ⭐⭐ Works, but awkward. Needed Alpine.js as a crutch.

---

### 2. **Shared State Across Components**

**The Problem:**
`leftFieldType` needed by multiple parts of the form:

- Operator dropdown (filter options)
- Right field selector (show compatible fields)
- Right value input (number vs text)

**HTMX Solution:**

```html
<div x-data="{ leftFieldType: null }">  <!-- Alpine.js! -->
    <select @change="leftFieldType = ...">
    <input x-show="leftFieldType === 'number'">
</div>
```

**Issues:**

- ❌ Had to use Alpine.js (not pure HTMX)
- ❌ Implicit scope (hard to track dependencies)
- ❌ No props/explicit data flow
- ❌ Easy to break by moving HTML

**React Equivalent:**

```tsx
const [leftFieldType, setLeftFieldType] = useState(null);
// Explicit, type-safe, clear data flow
```

**Verdict:** ⭐⭐ HTMX alone couldn't handle it. Alpine.js saved us.

---

### 3. **Tree Structure / Nested Data**

**The Problem:**
Needed to represent:

```
Root (AND)
├─ Condition: amount > 1000
├─ Group (OR)
│  ├─ Condition: country = "NG"
│  └─ Condition: country = "RU"
└─ Condition: age < 30
```

**HTMX Solution:**

- Path-based IDs: `"0"`, `"0-1"`, `"0-1-0"`
- Full tree re-render on every change
- 200+ lines of `format!()` strings in Rust

**Issues:**

- ❌ **Manual HTML generation** (lost template benefits)
- ❌ **Fragile path strings** (no type safety)
- ❌ **Full re-render** (can't do surgical updates)
- ❌ **No drag-and-drop** (would be extremely complex)
- ❌ **No undo/redo** (no state history)

**Code Smell:**

```rust
// 100+ lines of this:
format!(r##"<div id="node-{path}" class="condition-group">
    <select hx-post="/rule/node/{path}/operator">
    ...
</div>"##, path = path)
```

**React Equivalent:**

```tsx
function TreeNode({ node, path }: Props) {
  return node.type === "group" ? (
    <Group {...node}>
      {node.children.map((child, i) => (
        <TreeNode node={child} path={[...path, i]} />
      ))}
    </Group>
  ) : (
    <Condition {...node} />
  );
}
```

**Verdict:** ⭐ Technically works, but painful. This is where React shines.

---

### 4. **Context-Aware Forms**

**The Problem:**
Each group needs a form that posts to its specific path:

- Root group: `POST /rule/node/0/add-condition`
- Nested group: `POST /rule/node/0-1/add-condition`

**Initial Attempt (Failed):**

```html
<!-- Static template - can't customize per group -->
<form hx-post="/rule/node/0/add-condition"></form>
```

**Final Solution:**

```rust
// Generate entire form in Rust with path baked in
pub async fn new_condition_form(Path(path): Path<String>) -> Response {
    let form_html = format!(
        r##"<form hx-post="/rule/node/{}/add-condition">
            <!-- 100+ lines of HTML -->
        </form>"##,
        path
    );
    Html(form_html).into_response()
}
```

**Issues:**

- ❌ **Lost template separation** (HTML in Rust code)
- ❌ **No syntax highlighting** for HTML
- ❌ **Hard to maintain** (string concatenation)
- ❌ **Deleted the template file** (became unused)

**Verdict:** ⭐ Works, but feels wrong. Lost all benefits of templating.

---

### 5. **Localization (i18n)**

**The Problem:**
Server-side i18n is less mature than frontend.

**HTMX Approach:**

```rust
t!("rules.title", locale = locale)  // No autocomplete, no type safety
```

**Issues:**

- ❌ Worse tooling than frontend (no i18n-ally, etc.)
- ❌ No compile-time checking of translation keys
- ❌ Locale switch requires full page reload
- ❌ Manual date/number formatting

**React Approach:**

```tsx
const { t } = useTranslation();
t("rules.title"); // Autocomplete, type-safe, instant switch
```

**Verdict:** ⭐⭐ Functional but inferior to frontend solutions.

---

### 6. **Debugging & Developer Experience**

**HTMX Debugging:**

- Network tab: "Which request updated this?"
- HTML inspector: "What triggered this swap?"
- No stack traces
- No component tree
- No state inspector
- String-based targeting (typos = silent failures)

**React DevTools:**

- Component tree with props/state
- Time-travel debugging
- Re-render tracking
- Type errors at compile time

**Verdict:** ⭐⭐ Debugging HTMX apps is harder.

---

## The Complexity Curve

```
Simple ←────────────────────────────────→ Complex
        │                    │                    │
     HTMX                  POC                 React
   Sweet Spot          (We Are Here)        Sweet Spot

   ⭐⭐⭐⭐⭐              ⭐⭐                  ⭐⭐⭐⭐⭐
```

### **HTMX Sweet Spot:**

- CRUD operations
- Content-heavy pages
- Simple forms (1-3 fields)
- Server-driven workflows
- Progressive enhancement

### **Our POC:**

- ✅ Started in sweet spot
- ⚠️ Dependent dropdowns → needed Alpine.js
- ❌ Tree structure → painful
- ❌ Context-aware forms → lost templating
- ❌ Would need expressions → very complex

### **React Sweet Spot:**

- Complex forms with dependencies
- Tree/nested data structures
- Rich interactions
- Client-side state
- Real-time updates

---

## Key Learnings

### 1. **HTMX is NOT "Just HTML"**

**Marketing:** "HTMX lets you build SPAs with HTML attributes"

**Reality:**

- Simple cases: Yes, mostly HTML
- Complex cases: Lots of server-side HTML generation in Rust
- Lost template benefits
- Ended up with 200+ lines of `format!()` strings

**Lesson:** HTMX pushes complexity to the server, but it's still complexity.

---

### 2. **The "Re-render Everything" Pattern**

**HTMX Best Practice:** When in doubt, re-render the whole section.

**Our Implementation:**

```rust
// Every change re-renders entire tree
pub async fn add_condition(...) -> Response {
    // ... modify tree ...
    render_tree_node(&rule.root, "0", 0)  // Full tree
}
```

**Pros:**

- ✅ Simple (no partial update logic)
- ✅ Always consistent (no stale state)

**Cons:**

- ❌ Inefficient (re-render unchanged nodes)
- ❌ Loses scroll position
- ❌ Loses focus state
- ❌ No animations/transitions

**React:** Virtual DOM diffs and updates only what changed.

**Verdict:** HTMX's simplicity is also its limitation.

---

### 3. **String-Based Everything**

**HTMX:**

```html
hx-target="#operator-group"
<!-- String, no type safety -->
hx-include="[name='left_field']"
<!-- CSS selector magic -->
```

**Issues:**

- Typo in ID? Silent failure
- Rename an element? Find all string references
- No IDE support (go to definition, refactoring)

**React:**

```tsx
const operatorRef = useRef();
<div ref={operatorRef}>  // Type-safe reference
```

**Verdict:** String-based targeting is fragile at scale.

---

### 4. **Alpine.js as a Crutch**

**Started:** Pure HTMX  
**Ended:** HTMX + Alpine.js

**Why?**

- Client-side state (leftFieldType)
- Toggle UI (field vs value)
- Conditional rendering (x-show)

**Lesson:** Pure HTMX is rarely enough for complex UIs. You end up adding a JavaScript framework anyway.

---

### 5. **The Auth System Was Easy**

**Implemented in 30 minutes:**

- ✅ Session management
- ✅ Login/logout
- ✅ Protected routes
- ✅ Middleware

**Why It Worked:**

- Server-side only (HTMX strength)
- No complex client state
- Standard HTTP patterns

**Verdict:** ⭐⭐⭐⭐⭐ HTMX is great for auth!

---

## Performance Considerations

### **Bundle Size:**

- **HTMX:** ~14KB (gzipped)
- **Alpine.js:** ~15KB (gzipped)
- **React + ReactDOM:** ~130KB (gzipped)

**Winner:** HTMX (9x smaller)

### **Initial Load:**

- **HTMX:** Fast (server-rendered HTML)
- **React:** Slower (JS bundle + render)

**Winner:** HTMX

### **Interactions:**

- **HTMX:** Server round-trip for each action
- **React:** Instant (client-side)

**Winner:** React

### **Complex Updates:**

- **HTMX:** Re-render entire section
- **React:** Surgical updates (Virtual DOM)

**Winner:** React

**Verdict:** HTMX wins on initial load, React wins on interactions.

---

## Would I Use HTMX for This Project in Production?

### **No. Here's why:**

1. **Tree structure is painful**

   - Manual path management
   - Full re-renders
   - Fragile string-based IDs

2. **Lost templating benefits**

   - HTML generation in Rust
   - No syntax highlighting
   - Hard to maintain

3. **Needed Alpine.js anyway**

   - Not pure HTMX
   - Two mental models

4. **No room to grow**

   - Adding expressions? Very complex
   - Drag-and-drop? Nearly impossible
   - Undo/redo? No state history

5. **Debugging is harder**
   - No component tree
   - String-based targeting
   - Silent failures

### **What I'd Use Instead:**

**For this specific app:**

- **React** for the rule builder (complex form)
- **HTMX** for the shell (navigation, auth)
- **Hybrid approach** (best of both worlds)

**Architecture:**

```
/                    → HTMX (shell)
/login               → HTMX (simple form)
/rules               → HTMX (list)
/rules/:id/builder   → React (complex tree UI)
```

---

## When WOULD I Use HTMX?

### **Perfect Use Cases:**

1. **Content Management Systems**

   - Lots of CRUD
   - Simple forms
   - Server-driven

2. **Admin Dashboards**

   - Tables with filters
   - Simple charts
   - Form submissions

3. **E-commerce Product Pages**

   - Add to cart
   - Reviews
   - Image gallery

4. **Blog/Documentation Sites**

   - Comments
   - Search
   - Pagination

5. **Traditional Web Apps**
   - Where you'd use jQuery
   - Progressive enhancement
   - Server-side rendering

### **When to Avoid HTMX:**

1. **Complex Forms**

   - Multi-step wizards
   - Heavy dependencies between fields
   - Real-time validation

2. **Tree/Nested Structures**

   - File explorers
   - Org charts
   - Rule builders (like ours!)

3. **Rich Interactions**

   - Drag-and-drop
   - Canvas/drawing
   - Real-time collaboration

4. **Heavy Client-Side Logic**
   - Calculations
   - Data transformations
   - Complex state machines

---

## The Verdict

### **HTMX Rating by Feature:**

| Feature             | Rating     | Notes               |
| ------------------- | ---------- | ------------------- |
| Simple CRUD         | ⭐⭐⭐⭐⭐ | Perfect             |
| Basic Forms         | ⭐⭐⭐⭐⭐ | Excellent           |
| Auth                | ⭐⭐⭐⭐⭐ | Great               |
| Dependent Dropdowns | ⭐⭐⭐     | Needed Alpine.js    |
| Shared State        | ⭐⭐       | Needed Alpine.js    |
| Tree Structures     | ⭐         | Painful             |
| Context-Aware Forms | ⭐         | Lost templating     |
| i18n                | ⭐⭐       | Worse than frontend |
| Debugging           | ⭐⭐       | Limited tools       |

### **Overall:**

**HTMX is excellent for 70% of web apps.**

For the other 30% (complex UIs, heavy client state, rich interactions), React/Vue/Svelte are better choices.

**Our fraud rule builder fell into that 30%.**

---

## Final Thoughts

### **What I Learned:**

1. **HTMX's simplicity is real** - for simple cases
2. **Complexity doesn't disappear** - it moves to the server
3. **String-based targeting is fragile** at scale
4. **You often need Alpine.js** for client state
5. **Template generation in code** defeats the purpose
6. **React's complexity is justified** for complex UIs

### **HTMX's Best Contribution:**

Not as a React replacement, but as a **reminder that we over-engineer**.

Many apps that use React could be simpler with HTMX. But not all.

### **The Right Tool for the Job:**

```
Simple app → HTMX (avoid React overhead)
Complex app → React (avoid HTMX pain)
Hybrid app → Both (use each where it shines)
```

---

## Code Statistics

**Final Codebase:**

- **Rust:** ~850 lines (handlers.rs)
- **Models:** ~366 lines
- **Templates:** 6 files
- **CSS:** ~478 lines

**Key Metrics:**

- **HTML in Rust:** 100+ lines (format! strings)
- **Deleted templates:** 1 (became unused)
- **JavaScript frameworks:** 2 (HTMX + Alpine.js)
- **Server round-trips per action:** 1-3

**Complexity Indicators:**

- Path parsing: Custom implementation
- Tree navigation: Manual recursion
- Form generation: Dynamic in Rust
- State management: Split between server and Alpine.js

---

## Conclusion

**HTMX is a valuable tool**, but it's not a silver bullet.

For our fraud rule builder:

- ✅ Started well (simple CRUD)
- ⚠️ Got complex (dependent forms)
- ❌ Became painful (tree structure)

**The lesson:** Know your app's complexity before choosing HTMX.

If you're building:

- A blog? Use HTMX.
- An admin panel? Use HTMX.
- A rule builder with nested groups? Use React.

**HTMX didn't fail us. We pushed it beyond its sweet spot.**

And that's valuable knowledge! 🎯

---

## Resources

- **HTMX Docs:** https://htmx.org/
- **Alpine.js:** https://alpinejs.dev/
- **This POC:** A case study in HTMX's limits

**Thank you for this exploration!** It was a perfect test of HTMX's capabilities and boundaries.
