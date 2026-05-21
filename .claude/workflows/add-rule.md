# Workflow: Adding a New WCAG Rule

Step-by-step guide for implementing a new WCAG accessibility rule.

---

## Prerequisites

- [ ] Read WCAG documentation for the rule
- [ ] Understand the success criterion
- [ ] Identify AXTree properties needed
- [ ] Create test case HTML

---

## Step 1: Research the Rule

1. **Read WCAG spec:**  
   https://www.w3.org/WAI/WCAG21/Understanding/{rule-name}

2. **Identify key criteria:**
   - What makes content pass?
   - What makes content fail?
   - Edge cases to consider

3. **Determine AXTree properties:**
   - Which `role` values are affected?
   - Which properties (`name`, `value`, `properties`) to check?
   - Are parent/child relationships relevant?

---

## Step 2: Create Rule File from Template

```bash
# Copy template
cp .claude/templates/wcag-rule.rs.template \
   src/wcag/rules/{rule_name}.rs

# Example: For WCAG 2.1.1 (Keyboard)
cp .claude/templates/wcag-rule.rs.template \
   src/wcag/rules/keyboard.rs
```

**Fill in placeholders:**
- `{RULE_CODE}` → e.g., `2.1.1`
- `{RULE_NAME}` → e.g., `Keyboard`
- `{LEVEL}` → `A`, `AA`, or `AAA`
- `{STRUCT_NAME}` → e.g., `Keyboard`
- `{DESCRIPTION}` → Brief description
- `{SUCCESS_CRITERION}` → From WCAG spec
- `{VIOLATION_MESSAGE}` → Error message for users
- `{RULE_CODE_UNDERSCORE}` → e.g., `2_1_1` (for test names)

---

## Step 3: Implement Logic

### Example: Image Alt Text (1.1.1)

```rust
fn is_violation(&self, node: &AXNode) -> bool {
    // Images must have accessible name
    (node.role == "image" || node.role == "img") && 
    node.name.is_none()
}
```

### Example: Form Labels (4.1.2)

```rust
fn is_violation(&self, node: &AXNode) -> bool {
    // Interactive elements need names
    let requires_name = matches!(node.role.as_str(), 
        "button" | "textbox" | "checkbox" | "radio" | "combobox"
    );
    
    requires_name && (node.name.is_none() || node.name.as_ref().unwrap().is_empty())
}
```

### Example: Heading Hierarchy (2.4.6)

```rust
fn check(&self, tree: &AXTree) -> Vec<Violation> {
    let mut violations = Vec::new();
    let headings: Vec<&AXNode> = tree.nodes.iter()
        .filter(|n| n.role == "heading")
        .collect();
    
    let mut prev_level = 0;
    for heading in headings {
        let level = self.get_heading_level(heading);
        
        if level > prev_level + 1 {
            violations.push(/* ... */);
        }
        
        prev_level = level;
    }
    
    violations
}
```

---

## Step 4: Register Rule

**Edit:** `src/wcag/rules/mod.rs`

```rust
// Add module declaration
pub mod {rule_name};

// Re-export
pub use {rule_name}::{STRUCT_NAME}Rule;
```

**Edit:** `src/wcag/engine.rs`

```rust
impl WCAGEngine {
    pub fn new(level: WCAGLevel) -> Self {
        let mut engine = Self { /* ... */ };
        
        // Add rule registration
        engine.register_rule(Box::new({STRUCT_NAME}Rule));
        
        engine
    }
}
```

---

## Step 5: Create Test Fixture

**File:** `tests/fixtures/{rule_name}_violation.html`

### Example: Missing Alt

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Test: Missing Alt Text</title>
</head>
<body>
    <h1>Image Alt Text Test</h1>
    
    <!-- VIOLATION: No alt text -->
    <img src="logo.png">
    
    <!-- OK: Has alt text -->
    <img src="photo.jpg" alt="A beautiful sunset">
    
    <!-- OK: Decorative image (empty alt) -->
    <img src="decoration.png" alt="">
</body>
</html>
```

### Example: Unlabeled Form

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Test: Form Labels</title>
</head>
<body>
    <h1>Form Label Test</h1>
    
    <form>
        <!-- VIOLATION: No label -->
        <input type="text" name="email">
        
        <!-- OK: Has label -->
        <label for="name">Name:</label>
        <input type="text" id="name" name="name">
        
        <!-- OK: aria-label -->
        <input type="text" name="search" aria-label="Search">
    </form>
</body>
</html>
```

---

## Step 6: Write Integration Test

**Edit:** `tests/integration_test.rs`

```rust
#[tokio::test]
async fn test_{rule_name}_detection() {
    let html = include_str!("fixtures/{rule_name}_violation.html");
    
    // Serve HTML (use local HTTP server or file://)
    let report = audit_html(html).await.unwrap();
    
    // Assert violation detected
    assert!(
        report.violations.iter().any(|v| v.rule == "{RULE_CODE}"),
        "Expected to find {RULE_CODE} violation"
    );
    
    // Assert message contains expected text
    let violation = report.violations.iter()
        .find(|v| v.rule == "{RULE_CODE}")
        .unwrap();
    
    assert!(violation.message.contains("expected text"));
}
```

---

## Step 7: Run Tests

```bash
# Run unit tests for the rule
cargo test {rule_name}

# Run all WCAG tests
.claude/skills/test-wcag/test.sh

# Run full audit
.claude/skills/audit/run.sh
```

---

## Step 8: Update Documentation

1. **Add to `.claude/wcag-rules.md`:**
   ```markdown
   ### {RULE_CODE} {RULE_NAME}
   
   **File:** `src/wcag/rules/{rule_name}.rs`
   **Priority:** P0/P1/P2
   **WCAG Link:** https://...
   
   **Success Criterion:**
   ...
   
   **Implementation:**
   ```rust
   ...
   ```
   ```

2. **Update README.md:**
   - Increment rule count
   - Add to supported rules list

3. **Update CHANGELOG.md:**
   ```markdown
   ## [Unreleased]
   ### Added
   - WCAG {RULE_CODE} ({RULE_NAME}) detection
   ```

---

## Step 9: Verify End-to-End

```bash
# Build project
cargo build --release

# Test against fixture
./target/release/auditmysit file://$(pwd)/tests/fixtures/{rule_name}_violation.html -f table

# Expected output:
# ┌──────────┬──────────┬────────────────────┬─────────┐
# │ Rule     │ Severity │ Message            │ Element │
# ├──────────┼──────────┼────────────────────┼─────────┤
# │ {RULE_CODE} │ Error    │ {VIOLATION_MESSAGE}│ node-42 │
# └──────────┴──────────┴────────────────────┴─────────┘

# Test against real website
./target/release/auditmysit https://example.com
```

---

## Checklist

Before marking rule as complete:

- [ ] Rule code implemented in `src/wcag/rules/{rule_name}.rs`
- [ ] Unit tests pass with 100% coverage
- [ ] Fixture HTML created in `tests/fixtures/`
- [ ] Integration test added to `tests/integration_test.rs`
- [ ] Rule registered in `src/wcag/engine.rs`
- [ ] Module exported in `src/wcag/rules/mod.rs`
- [ ] Documentation updated in `.claude/wcag-rules.md`
- [ ] README.md updated
- [ ] End-to-end test passes
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt`

---

## Tips & Best Practices

### AXTree Navigation

```rust
// Find all nodes with specific role
let buttons: Vec<&AXNode> = tree.nodes.iter()
    .filter(|n| n.role == "button")
    .collect();

// Find nodes with property
let focusable: Vec<&AXNode> = tree.nodes.iter()
    .filter(|n| {
        n.properties.iter().any(|p| 
            p.name == "focusable" && p.value == serde_json::Value::Bool(true)
        )
    })
    .collect();

// Check parent-child relationships
fn find_parent<'a>(tree: &'a AXTree, child_id: &str) -> Option<&'a AXNode> {
    tree.nodes.iter().find(|n| n.children.contains(&child_id.to_string()))
}
```

### Custom Violation Messages

```rust
fn get_violation_message(&self, node: &AXNode) -> String {
    match node.role.as_str() {
        "button" => format!("Button missing accessible name"),
        "link" => format!("Link missing accessible name"),
        _ => format!("{} element requires accessible name", node.role),
    }
}
```

### Handling Edge Cases

```rust
// Ignore hidden elements
if node.ignored || node.properties.iter().any(|p| p.name == "hidden") {
    continue;
}

// Skip decorative images (empty alt is valid)
if node.role == "image" {
    match &node.name {
        None => { /* violation */ }
        Some(name) if name.is_empty() => { /* decorative, OK */ }
        Some(_) => { /* has alt, OK */ }
    }
}
```

---

## Example: Complete Implementation

See existing rules for reference:
- `src/wcag/rules/text_alternatives.rs` - Simple check
- `src/wcag/rules/headings.rs` - Hierarchical check
- `src/wcag/rules/contrast.rs` - Async CDP calls

---

**Questions?** Check `.claude/wcag-rules.md` or existing rule implementations.
