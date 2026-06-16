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
| rust     | Scaffolds a new project with `cargo new` |
| vue      | Scaffolds a new Vue project via the Vue CLI |

**Examples:**

```
total create rust my-app
total create vue my-app
```

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
total run <language>
```

**Supported languages:**

| Language | Behavior |
|----------|----------|
| rust     | Runs `cargo run` in the current directory |
| vue      | Runs `npm run serve` in the current directory |
| php      | Detects plain PHP or Laravel. If Laravel with a Vue frontend is detected, starts both `php artisan serve` and `npm run dev` concurrently |
| python   | Runs `main.py` or `app.py` if present, otherwise accepts `--path <file.py>` or `-p <file.py>` |

**Examples:**

```
total run rust
total run vue
total run php
total run python
total run python --path src/server.py
```

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
