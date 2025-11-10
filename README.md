<p align="center">
  <img src="logo.png" width="256">
</p>

<h1 align="center">Recursive Copy for POSIX Systems</h1> 

A lightweight, dependency-minimal, and secure implementation of recursive
directory copying for POSIX systems. Designed around the principles of
**minimalism**, **robustness**, and **predictability**, this module provides a
low-level yet safe interface for copying files and directories recursively,
while avoiding unnecessary abstractions or dependencies.

This implementation builds upon the custom crate
[`walkdir_minimal`](https://crates.io/crates/walkdir_minimal), a POSIX-only
reimplementation of directory walking that avoids the overhead of the
standard `walkdir` crate and provides fine-grained control over traversal
depth, symbolic link handling, and filesystem limits.

## ✨ Features

* Fully recursive copy of directories and files.
* Basic protection against symlink loops.
* Configurable recursion depth limit (`depth`).
* Efficient I/O with adjustable buffer size.
* Safe defaults for minimal risk operations.
* Automatically creates parent directories.
* Respects the `overwrite` flag.
* Uses an efficient buffered copy (`io::copy`).
* Preserves permissions by copying mode bits from source to destination (`& 0o777`).

## 🪶 Philosophy

The core philosophy of this implementation is to provide a **deterministic, auditable**,
and **secure** file copy mechanism that works strictly within POSIX constraints.
The design emphasizes:

* **Minimal dependencies** – only the Rust standard library and
* `walkdir_minimal` are used.
* **Security** – prevents following dangerous symlinks, ignores special
* files (devices, FIFOs, sockets), and sanitizes permissions.
* **Simplicity** – clear and direct code flow with minimal abstraction.
* **Predictability** – deterministic behavior with explicit user-controlled options.

This makes it suitable for use in low-level environments, system utilities,
AppImage or container tooling, and situations where a predictable and portable
copy operation is required.

## 📦 CopyOptions Structure

```rust
#[derive(Clone, Debug)]
pub struct CopyOptions {
    pub overwrite: bool,
    pub restrict_symlinks: bool,
    pub follow_symlinks: bool,
    pub content_only: bool,
    pub buffer_size: usize,
    pub depth: usize,
}
```

Each field provides precise control over copy behavior:

* **overwrite** – if `true`, existing destination files are replaced.
* **restrict_symlinks** – block traversal of symlinks pointing outside the source
directory (protects against path traversal).
* **follow_symlinks** – if `true`, copies the target of symlinks; otherwise,
recreates them as symlinks.
* **content_only** – copies only the contents of the source directory into the
destination (without creating a subdirectory).
* **buffer_size** – controls the buffer used by the internal `io::copy`
operation (default: 64 KiB).
* **depth** – limits directory traversal depth (default: 512 levels).

All fields have safe defaults via `CopyOptions::default()`.

## 🦉 Error Handling

All errors are represented by the following enum:

```rust
#[derive(Debug)]
pub enum CopyError {
    Io(io::Error),
    Walk(WalkError),
    DepthExceeded(PathBuf),
    SymlinkLoop(PathBuf),
    SrcNotFound(PathBuf),
    DestNotDir(PathBuf),
    NotSupported(PathBuf),
}
```

* **Io**: Any I/O failure during file operations.
* **Walk**: Errors from `walkdir_minimal`, e.g. permission denied or traversal issues.
* **DepthExceeded**: Triggered when the maximum depth limit is reached.
* **SymlinkLoop**: Prevents infinite recursion by tracking visited paths.
* **SrcNotFound**: Indicates that the source path does not exist.
* **DestNotDir**: Raised when destination is not a directory but should be.
* **NotSupported**: Returned for unsupported file types (devices, FIFOs, sockets, etc.).

Errors are propagated using idiomatic Rust `Result` types, allowing simple and
predictable handling.


## 🧩 Core Public Function: `copy_recursive`

```rust
pub fn copy_recursive(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<(), CopyError>
```

This is the main entry point for recursive copying. It supports copying files,
directories, and symbolic links according to the configured options.

### High-level algorithm

1. **Validate source** – checks whether `src` exists and determines its type.
2. **Prepare destination** – ensures directories exist or are created as needed.
3. **Handle file directly** – if `src` is a single file, copy it immediately.
4. **Traverse recursively** – for directories, it uses `walkdir_minimal` to iterate
over entries.
5. **Filter unsupported types** – devices, FIFOs, and sockets are silently ignored.
6. **Handle symlinks safely** – depending on `restrict_symlinks` and `follow_symlinks`
flags, either replicate or skip them.
7. **Preserve permissions** – sanitizes inherited permissions by masking to 0o777.

This function serves as a high-level controller that delegates actual operations to
helpers like `walk_and_copy`, `copy_one`, and `recreate_symlink`.

### File Copying: `copy_one`

Handles copying of individual files using standard I/O.

```rust
fn copy_one(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<(), CopyError>
```

To prevent unsafe permission inheritance, only the lower 9 permission bits are
copied (user/group/other), discarding sticky/SUID/SGID bits. This avoids
privilege escalation risks.

## 🧩 Directory Traversal: `walk_and_copy`

This is the core recursion engine, built on top of `walkdir_minimal`.

* Tracks visited directories using a `HashSet<PathBuf>` to avoid infinite loops.
* Enforces a maximum traversal depth.
* Supports optional symlink following and restriction logic.
* Ignores unsupported file types.
* Creates destination directories lazily when needed.

### Symlink Security

When both `follow_symlinks` and `restrict_symlinks` are enabled, the function
checks whether the resolved target of a symlink remains within the base source
directory. If not, the link is ignored and a warning is printed:

```text
Skipping symlink outside source: /path/a -> /etc/passwd
```

This prevents unintentional or malicious path traversal while still allowing
benign internal symlinks.

## ⚖️ Comparison with fs_extra

| Feature         | recursive_copy                                       | fs_extra                              |
| --------------- | ---------------------------------------------------- | ------------------------------------- |
| Dependencies    | Only std + walkdir_minimal                           | Multiple (serde, walkdir, etc.)       |
| Platform        | POSIX-only                                           | Cross-platform                        |
| Safety          | Enforced symlink restrictions, ignores special files | No strict restriction on symlinks     |
| Permissions     | Sanitized (0o777 mask)                               | Copies raw mode bits                  |
| Configurability | Core essentials only                                 | Many user-level options               |
| Purpose         | Minimal, auditable system utility                    | High-level library for app developers |

This module prioritizes **safety, predictability, and maintainability** over feature
breadth. It is ideal for system-level tooling, embedded environments, or container
systems where dependency minimization is essential.

This crate uses only the Rust standard library and POSIX APIs (via `std::os::unix`),
ensuring consistent behavior across Unix environments.

## 🦉 Examples

```rust
use std::path::Path;
use walkdir_minimal_copy::{copy_recursive, CopyOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = CopyOptions {
        overwrite: true,
        follow_symlinks: false,
        restrict_symlinks: true,
        content_only: false,
        ..Default::default()
    };

    copy_recursive(Path::new("/source"), Path::new("/backup"), &opts)?;
    Ok(())
}
```

The example above copies `/source` to `/backup`, preserving file modes, skipping
sockets and devices, and re-creating symlinks without following them.

Another example following symlinks safely:

```rust
let mut opts = CopyOptions::default();
opts.follow_symlinks = true;
opts.restrict_symlinks = true; // Prevents copying out of source directory
copy_recursive(Path::new("/src"), Path::new("/mirror"), &opts)?;
```

## 🤝 Contributing

Contributions are welcome! However, the project’s core principles
are **minimalism, security, and stability**.

Please note:

* New functionality should only be added behind **optional feature flags**.
* The **default build** must remain dependency-free and minimal.
* Focus on code clarity and POSIX compliance.


## 🔐 Testing and Reliability

`copy_recursive` was designed to be predictable, reproducible, and deterministic.
The included test suite focuses on edge cases rather than trivial success cases.
Tests ensure that recursion depth limits, symlink handling, and overwrite policies
behave as intended. Because the crate targets POSIX systems, all tests are written
assuming Unix semantics (e.g., permissions, symlinks, devices).

When testing, symbolic link loops are detected through an in-memory `HashSet<PathBuf>`
of visited canonical paths. This simple yet robust loop prevention method ensures that
no infinite recursion occurs, even when circular links are encountered. Similarly,
unusual file types such as block devices, character devices, sockets, and FIFOs are
explicitly skipped to prevent accidental interference with system resources.

### Expected Behavior Summary

* **Regular files:** Copied using `std::io::copy` with preserved permissions.
* **Directories:** Recursively traversed and created as needed.
* **Symlinks:** Either followed or re-created based on user options.
* **Unsupported special files:** Ignored by design to ensure safety.
* **Depth limit:** Ensures that extremely deep or cyclic structures do not cause
stack overflows.

These guarantees make the crate safe for system utilities, backup tools, and
packaging systems that require fine control and predictable outcomes.

## 📄 License

This project is licensed under the **MIT License**.
See the [`LICENSE`](./LICENSE) file for full details.

By contributing to this repository, you agree that your contributions
will be licensed under the same MIT terms.

## 📄 Changelog

All notable changes to this project will be documented
in the [`changelog`](./changelog) file.

## 🧑‍💻 Author

Created and maintained by **LinuxDicasPro**.

This crate is part of a minimalist POSIX-oriented Rust toolchain aimed at
creating small, safe, and efficient filesystem utilities suitable for
environments where reliability, clarity, and deterministic behavior are
prioritized over feature bloat.

If you find this project useful, consider starring the repository or
contributing feedback to improve it further.