# Total CLI

A command-line tool for scaffolding, running, and managing development projects. Built with Rust.

---

## Installation

Clone the repository and build with Cargo:

```
git clone https://github.com/jdn-the-dev/total-cli
cd total-cli
cargo build --release
```

Add the resulting binary to your PATH to use `total` from anywhere.

---

## Commands

### create

Scaffold a new project in a supported language.

```
total create <language> <title>
```

**Supported languages:**

| Language | Description                          |
|----------|--------------------------------------|
| python   | Creates a Python project with an entrypoint, dependency file, tests directory, and `.gitignore` |
| rust     | Scaffolds a new project with `cargo new` |
| vue      | Scaffolds a new Vue project via the Vue CLI |

**Examples:**

```
total create rust my-app
total create vue my-app
total create python my-app
```

Every successful scaffold includes `.total/app.toml`, the project manifest used by
Total CLI. It records project metadata plus the framework-specific run, build,
development, test, log, cleanup, AI, deployment, health, and environment settings.

---

### delete

Delete a file or folder at the given path. Prompts for confirmation before removing anything.

```
total delete <path>
total d <path>
total --delete <path>
total --d <path>
```

**Examples:**

```
total delete .\my-folder
total --d .\my-folder
total delete my-file.txt
```

**Notes:**

- Accepts both files and directories. Directories are removed recursively.
- Trailing backslashes in the path are handled automatically, including the Windows argument-parsing edge case where a quoted path ending in `\` injects a stray `"` character.
- On Windows, paths are canonicalized before deletion to correctly resolve relative components (`.`, `..`) that the `\\?\` UNC prefix would otherwise fail to handle.

---

### run

Run an existing project in the current directory.

```
total run <language> [extra args...]
```

Any arguments after the language are forwarded directly to the underlying tool. For example, `total run rust --release` passes `--release` to `cargo run`.

**Supported languages:**

| Language | Behavior |
|----------|----------|
| rust     | Runs `cargo run` in the current directory |
| vue      | Runs `npm run dev` in the current directory |
| next     | Runs `npm run dev` in the current directory (also accepts `nextjs` or `next.js`) |
| php      | Detects plain PHP or Laravel. Plain PHP runs `php -S localhost:8000`. Laravel alone runs `php artisan serve`. Laravel with a Vue frontend starts both `php artisan serve` and `npm run dev` concurrently |
| python   | Runs `main.py` or `app.py` if present, otherwise accepts `--path <file.py>` or `-p <file.py>` |

**Examples:**

```
total run rust
total run rust --release
total run vue
total run next
total run php
total run python
total run python --path src/server.py
```

**Notes:**

- All run commands stream output in real time (stdin, stdout, and stderr are inherited from the terminal).
- When running a Laravel + Vue project, both the backend and frontend are waited on concurrently so neither blocks the other's output.

---

## Project Structure

```
src/
├── args.rs           # CLI argument definitions (clap)
├── main.rs           # Entry point and command dispatch
├── delete/
│   └── mod.rs        # Delete command logic
├── scaffolding/
│   ├── mod.rs        # Create command logic (Rust, Vue scaffolding)
│   └── package_manger.rs
└── installer/
    └── mod.rs
```

Each command is isolated in its own module so new commands can be added without touching unrelated code.

---

## Requirements

- Rust / Cargo
- For `create vue` and `run vue`: Node.js, npm, and the Vue CLI (`npm install -g @vue/cli`)
- For `run php`: PHP installed and available in PATH
- For `run python`: Python installed and available in PATH
