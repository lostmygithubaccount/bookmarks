# bookmarks

Bookmarks in your filesystem.

## Commands

```bash
bin/build          # Build all (Rust + Python)
bin/build-rs       # Build Rust crate
bin/build-py       # Build Python bindings (maturin develop)
bin/check          # Run all checks (format, lint, test)
bin/check-rs       # Rust checks (fmt, clippy, test)
bin/check-py       # Python checks (ruff, ty)
bin/test           # Run all tests
bin/test-rs        # Rust tests
bin/format         # Format all code
bin/install        # Install CLI (Rust + Python)
bin/bump-version   # Bump version (--patch, --minor (default), --major)
```

## Architecture

```
crates/bookmarks-cli/     # Core Rust crate (dkdc-bookmarks on crates.io, bookmarks binary)
  src/lib.rs              # Library root
  src/main.rs             # Binary entry point
  src/cli.rs              # CLI (clap) with -f, --app, --webapp flags
  src/config.rs           # Config loading/saving
  src/open.rs             # Link resolution (alias → link → URI)
  src/app.rs              # iced desktop app (behind `app` feature flag)
  src/webapp.rs           # Axum HTMX webapp on port 1414 (behind `webapp` feature flag)
  assets/icon.png         # App window icon
crates/bookmarks-py/      # PyO3 bindings (cdylib)
py/bookmarks/             # Python wrapper + type stubs (core.pyi, py.typed)
```

Config resolution: `-f` flag > `./bookmarks.toml` (cwd) > `~/.config/bookmarks/bookmarks.toml` (global).

Config structure: aliases map to links, links map to URIs, groups expand to multiple aliases/links.
