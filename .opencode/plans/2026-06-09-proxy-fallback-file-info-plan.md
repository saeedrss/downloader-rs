# Proxy Fallback for File Info Fetching — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make file info fetching resilient by falling back through available proxies when direct connection fails.

**Architecture:** Extract direct-file-info logic into its own function, create a proxy-variant, and an orchestrator that tries direct → each proxy in sequence. Pass `file_size` to `process_url` as a parameter to eliminate a redundant fetch.

**Tech Stack:** Rust, reqwest, anyhow

---

### Task 1: Add helper functions to `download.rs`

**Files:**
- Modify: `src/download.rs` — add 3 new public functions after the existing `test_single_proxy` function

- [ ] **Step 1: Add `get_file_size_direct` to `download.rs`**

Extract the current `get_file_size` logic from `main.rs` into a standalone function in `download.rs`. It does a GET via a direct (no-proxy) `reqwest::Client` and parses `Content-Length`.

Insert after line 30 (after `test_single_proxy`):

```rust
pub async fn get_file_size_direct(url: &str) -> Result<u64> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "curl/8.14.1")
        .header("Accept", "*/*")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;
    if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(anyhow!("Server returned status {}", resp.status()));
    }
    let size: u64 = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow!("Content-Length not found"))?;
    if size == 0 {
        return Err(anyhow!("File size is 0"));
    }
    Ok(size)
}
```

- [ ] **Step 2: Add `get_file_size_via_proxy` to `download.rs`**

Same logic as `get_file_size_direct` but builds client with `Proxy::all(proxy_url)`. Insert after `get_file_size_direct`:

```rust
pub async fn get_file_size_via_proxy(url: &str, proxy_url: &str) -> Result<u64> {
    let proxy = Proxy::all(proxy_url)
        .map_err(|e| anyhow!("Invalid proxy {}: {}", proxy_url, e))?;
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| anyhow!("Failed to build proxy client: {}", e))?;
    let resp = client
        .get(url)
        .header("User-Agent", "curl/8.14.1")
        .header("Accept", "*/*")
        .send()
        .await?;
    if !resp.status().is_success() && resp.status().as_u16() != 206 {
        return Err(anyhow!("Server returned status {}", resp.status()));
    }
    let size: u64 = resp
        .headers()
        .get(reqwest::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow!("Content-Length not found"))?;
    if size == 0 {
        return Err(anyhow!("File size is 0"));
    }
    Ok(size)
}
```

- [ ] **Step 3: Add `get_file_size_with_fallback` to `download.rs`**

Orchestrator that tries direct first, then iterates proxies. Insert after `get_file_size_via_proxy`:

```rust
pub async fn get_file_size_with_fallback(
    url: &str,
    proxies: &[String],
    log: &Arc<dyn Fn(&str) + Send + Sync>,
) -> Result<u64> {
    log("[*] Trying direct connection to fetch file info...");
    match get_file_size_direct(url).await {
        Ok(size) => return Ok(size),
        Err(e) => log(&format!("[-] Direct connection failed: {}", e)),
    }

    for proxy_url in proxies {
        log(&format!("[*] Trying proxy {} to fetch file info...", proxy_url));
        match get_file_size_via_proxy(url, proxy_url).await {
            Ok(size) => return Ok(size),
            Err(e) => log(&format!("[-] Proxy {} failed: {}", proxy_url, e)),
        }
    }

    Err(anyhow!("[-] All connection methods failed to fetch file size"))
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build`
Expected: compiles (may have warnings about unused imports in `main.rs` — that's fine, Task 2 fixes it)

- [ ] **Step 5: Commit**

```bash
git add src/download.rs
git commit -m "feat: add proxy fallback helpers for file info fetching"
```

---

### Task 2: Update `process_url` to accept `file_size` parameter

**Files:**
- Modify: `src/main.rs` — change `process_url` signature and remove its `get_file_size` call

- [ ] **Step 1: Add `file_size` parameter to `process_url`**

Change the `process_url` function signature (line 145-157) to add `file_size: u64` between `url` and `chunk_size`:

```rust
async fn process_url(
    url: &str,
    file_size: u64,
    chunk_size: u64,
    max_connections: u32,
    proxy_urls: &[String],
    output_path: &Path,
    temp_dir: &Path,
    dyn_state: DynState,
    log: Arc<dyn Fn(&str) + Send + Sync + 'static>,
    on_part_update: Arc<dyn Fn(u32, &str, &str, f64) + Send + Sync + 'static>,
    on_progress: Arc<dyn Fn(u32, u32) + Send + Sync + 'static>,
    on_proxy_update: Arc<dyn Fn(Vec<(String, f64, u64)>) + Send + Sync + 'static>,
) {
```

- [ ] **Step 2: Remove the `get_file_size` call and use the parameter**

Replace lines 167-179 (the `get_file_size` call and its match) with:

```rust
    log(&format!(
        "[*] File size: {:.2} GB ({} bytes)",
        file_size as f64 / 1_073_741_824.0,
        file_size
    ));
```

- [ ] **Step 3: Remove unused `anyhow` import if it becomes unused**

Check if `anyhow` and `get_file_size` references remain in `main.rs`. The `get_file_size` function itself (lines 95-117) will still be used by the pre-gathering phase — keep it or remove it depending on whether pre-gathering still calls it.

Actually, since pre-gathering (line 549) will now call `get_file_size_with_fallback` from `download.rs`, and the only other caller was `process_url` which we just changed, the old `get_file_size` function in `main.rs` is now dead code. **Remove it** (lines 95-117).

- [ ] **Step 4: Verify compilation**

Run: `cargo build`
Expected: compiles cleanly

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "refactor: pass file_size to process_url instead of fetching internally"
```

---

### Task 3: Fix pre-gathering phase to use proxy fallback

**Files:**
- Modify: `src/main.rs` — replace `get_file_size` call in pre-gathering loop

- [ ] **Step 1: Update `main.rs` imports**

Add `get_file_size_with_fallback` to the import from `download` module (line 15). Change:

```rust
use download::{download_chunk, filter_alive_proxies, proxy_health_checker};
```

to:

```rust
use download::{
    download_chunk, filter_alive_proxies, get_file_size_with_fallback, proxy_health_checker,
};
```

- [ ] **Step 2: Replace `get_file_size` in pre-gathering**

In the pre-gathering async block (line 543-562), replace line 549:

```rust
match get_file_size(url).await {
```

with:

```rust
let plog = |msg: &str| println!("{}", msg);
let plog_arc = Arc::new(plog);
match get_file_size_with_fallback(url, &proxy_urls, &plog_arc).await {
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: compiles cleanly

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: use proxy fallback in pre-gathering phase"
```

---

### Task 4: Fix download thread to use proxy fallback

**Files:**
- Modify: `src/main.rs` — restructure download thread to define log closure early and use fallback

- [ ] **Step 1: Move `log` closure before the file size fetch in download thread**

In the download thread (starting line 624), the `log` closure is currently defined at line 650 inside the `if let Some(file_size) = file_size_opt` block. We need to define it earlier so it can be used for the fallback fetch.

Restructure lines 627-692 from:

```rust
for (tab_idx, url) in d_urls.iter().enumerate() {
    let file_size_opt: Option<u64> = {
        match get_file_size(url).await {
            Ok(s) => Some(s),
            Err(_) => {
                let _ = d_tx.send(UiEvent::Log(
                    tab_idx,
                    format!("[-] Failed to get file size: {}", url),
                ));
                None
            }
        }
    };

    if let Some(file_size) = file_size_opt {
        let chunk_size = d_chunk_size;
        let total_parts = ((file_size + chunk_size - 1) / chunk_size) as u32;
        let _ = d_tx.send(UiEvent::FileStart(
            tab_idx,
            url.clone(),
            total_parts,
        ));

        let log_tx = d_tx.clone();
        let log = Arc::new(move |msg: &str| {
            let _ = log_tx.send(UiEvent::Log(tab_idx, msg.to_string()));
        });
        // ... rest of the block
    }
}
```

to:

```rust
for (tab_idx, url) in d_urls.iter().enumerate() {
    let log_tx = d_tx.clone();
    let log = Arc::new(move |msg: &str| {
        let _ = log_tx.send(UiEvent::Log(tab_idx, msg.to_string()));
    });

    let file_size = match get_file_size_with_fallback(url, &d_proxies, &log).await {
        Ok(s) => s,
        Err(e) => {
            log(&format!("[-] Failed to get file size: {}", e));
            continue;
        }
    };

    let chunk_size = d_chunk_size;
    let total_parts = ((file_size + chunk_size - 1) / chunk_size) as u32;
    let _ = d_tx.send(UiEvent::FileStart(
        tab_idx,
        url.clone(),
        total_parts,
    ));

    // log closure, part_cb, prog_cb, proxy_cb remain as before
    let part_tx = d_tx.clone();
    let part_cb = Arc::new(move |pn: u32, status: &str, _proxy: &str, elapsed: f64| {
        let s = match status {
            "finished" => PartStatus::Finished,
            "downloading" => PartStatus::Downloading,
            "error" => PartStatus::Error,
            _ => PartStatus::Idle,
        };
        let _ = part_tx.send(UiEvent::PartUpdate(tab_idx, pn, s, elapsed));
    });

    let prog_tx = d_tx.clone();
    let prog_cb = Arc::new(move |done: u32, total: u32| {
        let _ = prog_tx.send(UiEvent::Progress(tab_idx, done, total));
    });

    let proxy_tx = d_tx.clone();
    let proxy_cb = Arc::new(move |items: Vec<(String, f64, u64)>| {
        let _ = proxy_tx.send(UiEvent::ProxyUpdate(tab_idx, items));
    });

    process_url(
        url,
        file_size,  // NEW: pass file_size directly
        d_chunk_size,
        d_conn,
        &d_proxies,
        &d_paths[tab_idx],
        Path::new(&d_temp),
        d_dyn.clone(),
        log,
        part_cb,
        prog_cb,
        proxy_cb,
    )
    .await;

    let _ = d_tx.send(UiEvent::FileComplete(tab_idx));
}
```

- [ ] **Step 2: Remove unused imports if any**

The `get_file_size` import/usage in `main.rs` should now be gone (replaced by `get_file_size_with_fallback`). The `anyhow` import may still be used elsewhere. Verify no dead imports remain.

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: compiles cleanly

- [ ] **Step 4: Commit**

```bash
git add src/main.rs src/download.rs
git commit -m "feat: use proxy fallback in download thread for file info"
```

---

### Task 5: Final cleanup and verification

- [ ] **Step 1: Full build check**

Run: `cargo build`
Expected: clean compile with no warnings

- [ ] **Step 2: Run the binary (smoke test)**

```bash
cargo run -- --url "http://example.com/file" --proxies "socks5://127.0.0.1:9050" --size 5
```

Expected: binary starts, TUI loads, file info fetch appears in logs

- [ ] **Step 3: Commit if not already committed**

```bash
git add -A
git commit -m "feat: proxy fallback for file info fetching"
```
