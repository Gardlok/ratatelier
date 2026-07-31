# Publishing Ratatelier

This checklist is intentionally conservative. Publication is a repository-owner action; CI validates the package but never publishes it.

Ratatelier `0.2.2` declares and verifies **Rust 1.88** as its minimum supported Rust version. Ratatui `0.30.2` requires Rust 1.88, so older toolchains are not a supported release configuration.

## 1. Confirm the release baseline

```bash
cargo --version
rustc --version
git status --short
git log -1 --oneline
```

The worktree should be clean and the release commit should already be merged into `main`. The normal release validation may use current stable Rust, but the exact MSRV checks below must also pass on Rust `1.88.0`.

## 2. Run the complete validation set

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features
cargo test --all-targets --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features
cargo package --list
cargo package
```

The package should contain source, license, README, changelog, and publishing documentation. It must not contain build output, editor state, generated exports, or personal project files.

## 3. Verify the exact MSRV

Install Rust 1.88 through rustup when it is not already available:

```bash
rustup toolchain install 1.88.0
cargo +1.88.0 check --all-targets --all-features
cargo +1.88.0 test --all-targets --all-features
```

Do not lower `rust-version` without also selecting dependencies that genuinely compile and test on the lower toolchain.

## 4. Test the installable CLI

```bash
install_root="$(mktemp -d)"
cargo install --path . --root "$install_root"
"$install_root/bin/ratatelier" --version
"$install_root/bin/ratatelier" --help
rm -rf "$install_root"
```

For final acceptance, launch the installed binary in a terminal and verify mouse input, colors, clipboard behavior, command history, named undo feedback, and the unsaved-exit dialog.

## 5. Review public metadata

Confirm `Cargo.toml` contains the intended:

- package name and version;
- Rust 2024 edition and Rust 1.88 MSRV;
- description;
- MIT license;
- repository, homepage, and documentation URLs;
- keywords and categories;
- README path.

Check whether the crate name is available and whether the authenticated crates.io account is authorized to publish it.

## 6. Dry-run against crates.io

```bash
cargo publish --dry-run
```

Do not proceed if this differs materially from `cargo package` or reports metadata, ownership, or verification errors.

## 7. Publish deliberately

```bash
cargo publish
```

After crates.io accepts the release:

```bash
git tag -s v0.2.2 -m 'Ratatelier 0.2.2'
git push origin v0.2.2
```

Create a GitHub release from the matching changelog section and attach screenshots or recordings captured with the demo guide.

## 8. Verify the published artifact

After the index updates:

```bash
cargo install ratatelier --version 0.2.2 --locked
ratatelier --version
```

Also verify that docs.rs builds the release and that the repository links rendered by crates.io are correct.
