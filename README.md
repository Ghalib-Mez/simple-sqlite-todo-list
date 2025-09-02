# 📝 Rust TODO CLI

A simple command-line TODO manager written in Rust, backed by **Google Tasks** (with an optional in-memory implementation).
Supports adding, listing, completing, and removing tasks directly in your Google account.

---

## 🚀 Features
- ✅ Add TODO items with title + content
- ✅ List all items in a nice formatted view
- ✅ Mark tasks as complete
- ✅ Remove tasks
- ✅ Sync with Google Tasks via OAuth2
- ✅ Trait-based design (swap out Google Tasks for in-memory store)

---

## ⚙️ Requirements
- [Rust](https://www.rust-lang.org/) (latest stable recommended)
- A Google Cloud project with the **Google Tasks API** enabled
- OAuth2 credentials (`rust_oauth.json`)

---

## 🔧 Build & Run

### 1. Clone the repo
```bash
git clone https://github.com/yourname/todo-cli-rust.git
cd todo-cli-rust
```

### 2. Build
```bash
cargo build --release
```

### 3. Authenticate & Run
```bash
cargo run
```

On the first run, the app will ask you to authenticate with Google and store tokens in `tokencache.json`.

---

## 💻 Usage

Inside the CLI, you can type commands:

### Add an item
```bash
add "Title" "This is the content"
```

### List all items
```bash
list
```

Example output:
```
--- TODO List ---
[ ]: BuyMilk: Remember to get 2%
[X]: FinishRust: Implement complete_item()
-----------------
```

### Mark an item complete
```bash
complete "Title"
```

---

## 📌 Notes
- Tokens are cached locally in `tokencache.json` (ignored by git).
- Requires valid OAuth2 credentials (`rust_oauth.json`).

---

MIT – use however you want!
