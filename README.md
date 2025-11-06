# recursive_copy

A minimalist, dependency-free library for performing 
**secure recursive file and directory copies** on **Unix-like systems** — 
including Linux, BSDs, and Solaris. Designed for simplicity, safety, and stability.

## ✨ Features

* Fully recursive copy of directories and files.
* Basic protection against symlink loops.
* Configurable recursion depth limit (`max_depth`).
* Optional overwrite, symlink following, and permission preservation.
* Efficient I/O with adjustable buffer size.
* Safe defaults for minimal risk operations.

## 🧱 Compatibility

* **Supported platforms:** Linux, *BSD systems, Solaris.
* **Requirements:** A POSIX-compliant filesystem interface.
* **Not supported:** Windows (by design).

This crate uses only the Rust standard library and POSIX APIs (via `std::os::unix`),
ensuring consistent behavior across Unix environments.

## ⚙️ Example

```rust
use recursive_copy::{copy_recursive, CopyOptions};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = Path::new("/home/user/docs");
    let dst = Path::new("/backup/docs_copy");

    let opts = CopyOptions::default();
    let summary = copy_recursive(src, dst, &opts)?;

    println!("Copied {} bytes with {} errors", summary.bytes_copied, summary.errors);
    Ok(())
}
```

## 🔐 Security & Safety

* Prevents infinite recursion via a **symlink loop detector**.
* Enforces a **maximum directory depth**.
* Avoids accidental overwrites unless explicitly allowed.
* Supports **permission preservation** using POSIX `mode` bits.

## 🧩 Philosophy

This project follows a **minimalist design philosophy**:

* **No dependencies** — relies solely on the Rust standard library.
* **Stable behavior** — safe for system utilities, backups, and embedded tools.
* **Secure by default** — cautious handling of links, files, and permissions.

## 🤝 Contributing

Contributions are welcome! However, the project’s core principles
are **minimalism, security, and stability**.

Please note:

* New functionality should only be added behind **optional feature flags**.
* The **default build** must remain dependency-free and minimal.
* Focus on code clarity and POSIX compliance.

## 📄 License

Licensed under the MIT License.
