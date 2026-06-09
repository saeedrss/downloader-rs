# Fix Invalid Windows Filename Crash

**Problem:** `process_url` derives the temp directory name from the URL's last path segment without stripping query parameters. On Windows, `?` is a reserved character — `std::fs::create_dir_all()` returns `Err`, then `.unwrap()` panics.

**Trigger:** Only surfaces now because proxy fallback lets `get_file_size_with_fallback` succeed where direct connection failed.

**Fix scope:** 1 file, 2 changes.

---

### Task 1: Strip query params from filename and handle error gracefully

**File:** `src/main.rs`

**Step 1:** Replace the filename derivation (lines 137-142) to strip query strings.

Current:
```rust
    let file_name = url
        .split('/')
        .last()
        .unwrap_or("downloaded_file")
        .to_string();
```

Replace with:
```rust
    let file_name = url
        .split('/')
        .last()
        .unwrap_or("downloaded_file")
        .split('?')
        .next()
        .unwrap_or("downloaded_file")
        .to_string();
```

**Step 2:** Replace the `unwrap()` on `create_dir_all` (line 143) with proper error handling.

Current:
```rust
    std::fs::create_dir_all(&dl_dir).unwrap();
```

Replace with:
```rust
    if let Err(e) = std::fs::create_dir_all(&dl_dir) {
        log(&format!("[-] Failed to create download directory: {}", e));
        return;
    }
```

**Step 3:** Verify with `cargo build`.

**Step 4:** Commit:
```bash
git add src/main.rs
git commit -m "fix: strip query params from filename and handle dir creation error"
```
