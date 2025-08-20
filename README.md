# 📝 Rust TODO CLI


A simple command-line TODO manager written in Rust, backed by **SQLite** (with an optional in-memory implementation).
Supports adding, listing, completing, and removing tasks.


---


## 🚀 Features
- ✅ Add TODO items with title + content
- ✅ List all items in a nice formatted view
- ✅ Mark tasks as complete
- ✅ Remove tasks
- ✅ SQLite persistent storage
- ✅ Trait-based design (swap out DB for in-memory store)


---


## ⚙️ Requirements
- [Rust](https://www.rust-lang.org/) (latest stable recommended)
- [SQLite](https://sqlite.org/) (installed on your system)


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


### 3. Run
```bash
cargo run
```


This will create (or reuse) a `todos.db` SQLite file in your project directory.


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


MIT – use however you want!
