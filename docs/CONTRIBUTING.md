# Contributing to auditmysite

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs))
- Chrome or Chromium (auto-downloaded if not found)

### Build

```bash
git clone https://github.com/casoon/auditmysite.git
cd auditmysite
cargo build
```

### Run Tests

```bash
cargo test
```

### Run Locally

```bash
cargo run -- https://example.com
```

## Code Style

- Format with `cargo fmt`
- Lint with `cargo clippy`
- No warnings allowed in CI

## Adding a New WCAG Rule

1. Create a new file in `src/wcag/rules/`:

```rust
// src/wcag/rules/my_new_rule.rs

use crate::accessibility::tree::AXTree;
use crate::accessibility::styles::NodeStyle;
use crate::cli::WcagLevel;
use crate::wcag::types::{Severity, Violation};

/// WCAG X.X.X - Rule Name
/// 
/// Level: A/AA/AAA
/// 
/// https://www.w3.org/WAI/WCAG21/Understanding/rule-name
pub fn check(tree: &AXTree, _styles: &[NodeStyle], level: WcagLevel) -> Vec<Violation> {
    let mut violations = Vec::new();
    
    for node in tree.iter() {
        // Your rule logic here
        if has_violation(node) {
            violations.push(
                Violation::new(
                    "X.X.X",
                    "Rule Name",
                    level,
                    Severity::Serious,
                    "Description of the violation",
                    &node.node_id,
                )
                .with_role(node.role.clone())
                .with_fix("How to fix this issue")
                .with_help_url("https://www.w3.org/WAI/WCAG21/Understanding/rule-name")
            );
        }
    }
    
    violations
}

fn has_violation(node: &crate::accessibility::tree::AXNode) -> bool {
    // Check logic
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rule_metadata() {
        // Test your rule
    }
}
```

2. Register in `src/wcag/rules/mod.rs`:

```rust
pub mod my_new_rule;
```

3. Add to engine in `src/wcag/engine.rs`:

```rust
use super::rules::my_new_rule;

// In check_all():
if level >= WcagLevel::AA {  // or A, AAA
    violations.extend(my_new_rule::check(tree, styles, level));
}
```

4. Add tests and run:

```bash
cargo test my_new_rule
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run lints (`cargo clippy`)
6. Format code (`cargo fmt`)
7. Commit with a descriptive message
8. Push and create a Pull Request

## Commit Message Format

```
type: short description

Longer description if needed.

- Bullet points for multiple changes
- Another change
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `test`: Tests
- `refactor`: Code refactoring
- `chore`: Maintenance

## Reporting Issues

Please include:
- auditmysite version (`auditmysite --version`)
- Operating system
- Chrome/Chromium version (if relevant)
- Steps to reproduce
- Expected vs actual behavior

## Code of Conduct

Be respectful and inclusive. We welcome contributions from everyone.
