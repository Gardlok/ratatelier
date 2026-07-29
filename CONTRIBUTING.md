# Contributing

Ratatelier is terminal-first and model-driven. Changes should preserve these rules:

- editable state belongs in the serializable model;
- transient interaction state belongs in `App`;
- rendering must not become the canonical source of project data;
- exporters are deterministic readers of the model;
- mouse hit testing must use the same geometry as rendering;
- project format changes increment or migrate `PROJECT_VERSION`;
- avoid raw terminal escape sequences when Ratatui or Crossterm provides the operation.

Run before submitting changes:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
cargo test --all-targets --all-features
```
