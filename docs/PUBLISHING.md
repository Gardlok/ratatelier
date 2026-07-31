# Publishing Ratatelier

This checklist is intentionally conservative. Publication is a repository-owner action; CI validates the package but never publishes it.

## 1. Confirm the release baseline

```bash
cargo --version
rustc --version
git status --short
git log -1 --oneline
```

The worktree should be clean and the release commit should already be merged into `main`.

Ratatelier requires Rust 1.88 or newer. CI checks and tests the package on exact Rust 1.88.0 as well as current stable Rust.

## 2. Run the complete validation set

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features
cargo test --all-targets --all-features
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features
cargo package
```

Inspect the package contents:

```bash
cargo package --list
```

The package should contain source, license, README, changelog, viewing/conversion documentation, publishing documentation, and the demo guide. It must not contain build output, editor state, generated exports, or personal project files.

## 3. Test the installable CLI

```bash
install_root="$(mktemp -d)"
cargo install --path . --root "$install_root"
"$install_root/bin/ratatelier" --version
"$install_root/bin/ratatelier" --help
"$install_root/bin/ratatelier" view --help
"$install_root/bin/ratatelier" export --help
rm -rf "$install_root"
```

For final acceptance, launch the installed binary in a terminal and verify:

- editor mouse input, colors, clipboard behavior, and the unsaved-exit dialog;
- one-pass artwork and component viewing;
- pause, resume, replay, close, and optional looping;
- plain and ANSI output to standard output;
- direct output-file conversion;
- Rust artwork, animation, and component generation.

## 4. Review public metadata

Confirm `Cargo.toml` contains the intended:

- package name and version;
- Rust edition and MSRV;
- description;
- MIT license;
- repository, homepage, and documentation URLs;
- keywords and categories;
- README path.

Check whether the crate name is available and whether the authenticated crates.io account is authorized to publish it.

## 5. Dry-run against crates.io

```bash
cargo publish --dry-run
```

Do not proceed if this differs materially from `cargo package` or reports metadata, ownership, or verification errors.

## 6. Publish deliberately

```bash
cargo publish
```

After crates.io accepts the release:

```bash
git tag -s v0.3.0 -m 'Ratatelier 0.3.0'
git push origin v0.3.0
```

Create a GitHub release from the matching changelog section and attach screenshots or recordings captured with the demo guide.

## 7. Verify the published artifact

After the index updates:

```bash
cargo install ratatelier --version 0.3.0 --locked
ratatelier --version
ratatelier view --help
ratatelier export --help
```

Open a known project in both editor and viewer modes. Run at least one plain, ANSI, and generated-Rust conversion from the installed crates.io binary.

Also verify that docs.rs builds the release and that the repository links rendered by crates.io are correct.
