# Proxy Fallback for File Info Fetching

## Problem

`get_file_size()` uses a direct connection only. If the direct connection fails (VPS behind proxy, network issues, etc.), the entire download is dropped. Called in 3 places (pre-gathering, download thread, `process_url`), a single failure at any point aborts the URL.

Chunk downloads already have infinite retry with proxy rotation — that works fine.

## Solution

Add proxy fallback for file info fetching: try direct first, then iterate through all proxies sequentially until one succeeds.

## Changes

### 1. New functions in `download.rs`

- **`get_file_size_direct(url)`** — extracted from current `get_file_size()`. Direct `reqwest::Client`, GET with User-Agent, parse Content-Length. Returns `Result<u64>`.
- **`get_file_size_via_proxy(url, proxy_url)`** — same but via `Proxy::all(proxy_url)`. Returns `Result<u64>`.
- **`get_file_size_with_fallback(url, proxies, log)`** — orchestrator: try direct → iterate proxies → return first success or last error.

### 2. Modify `process_url` in `main.rs`

- Add `file_size: u64` parameter.
- Remove `get_file_size` call at line 168.
- Update log message.

### 3. Fix callers in `main.rs`

- **Pre-gathering** (line 549): Use `get_file_size_with_fallback` with `proxy_urls`.
- **Download thread** (line 629): Use `get_file_size_with_fallback` with `d_proxies`. Move `log` closure definition before this call. Pass `file_size` to `process_url`.

## Files affected

| File | Change |
|---|---|
| `src/download.rs` | +~40 lines (3 new functions) |
| `src/main.rs` | ~25 lines across 3 locations |

## Error handling

If direct + all proxies fail, the URL is still skipped — correct when every path is dead.

## Implementation order

1. Add 3 functions to `download.rs`
2. Update `process_url` signature in `main.rs`
3. Fix pre-gathering caller
4. Fix download thread caller (restructure log closure + pass file_size)
5. `cargo build` to verify
