use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use reqwest::Proxy;
use tokio::sync::Mutex;

use crate::state::DynState;

pub async fn test_single_proxy(proxy_url: &str, test_url: &str) -> bool {
    let proxy = match Proxy::all(proxy_url) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let client = match reqwest::Client::builder().proxy(proxy).build() {
        Ok(c) => c,
        Err(_) => return false,
    };
    match client
        .get(test_url)
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
    {
        Ok(resp) => resp.status().is_success() || resp.status().as_u16() == 206,
        Err(_) => false,
    }
}

pub async fn filter_alive_proxies(
    proxy_urls: &[String],
    test_url: &str,
    log: &Arc<dyn Fn(&str) + Send + Sync>,
) -> Vec<String> {
    log("[*] Checking proxies availability, please wait...");
    let mut tasks = Vec::new();
    for url in proxy_urls {
        let u = url.clone();
        let t = test_url.to_string();
        tasks.push(tokio::spawn(async move {
            test_single_proxy(&u, &t).await
        }));
    }
    let results = futures::future::join_all(tasks).await;
    let alive: Vec<String> = proxy_urls
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            results
                .get(*i)
                .and_then(|r| r.as_ref().ok())
                .copied()
                .unwrap_or(false)
        })
        .map(|(_, u)| u.clone())
        .collect();
    log(&format!(
        "[+] Found {} active proxies out of {}.",
        alive.len(),
        proxy_urls.len()
    ));
    alive
}

pub async fn download_chunk(
    client: &reqwest::Client,
    url: &str,
    start: u64,
    end: u64,
    part_num: u32,
    dl_dir: &Path,
    dyn_state: &DynState,
    log: &Arc<dyn Fn(&str) + Send + Sync>,
) -> Result<(bool, f64)> {
    let part_name = format!("{:x}", part_num);
    let filename = dl_dir.join(format!("part_{}.tmp", part_name));
    let expected_size = (end - start + 1) as u64;

    if filename.exists() {
        if filename.metadata().map(|m| m.len()).unwrap_or(0) == expected_size {
            return Ok((true, 0.0));
        }
        let _ = std::fs::remove_file(&filename);
    }

    let t = dyn_state.timeout.load(Ordering::Acquire) as u64;
    let t0 = Instant::now();

    let result: Result<reqwest::Response, reqwest::Error> = client
        .get(url)
        .header("Range", format!("bytes={}-{}", start, end))
        .header("User-Agent", "curl/8.14.1")
        .header("Accept", "*/*")
        .header("Connection", "keep-alive")
        .timeout(std::time::Duration::from_secs(t))
        .send()
        .await;

    match result {
        Ok(response) => {
            let status = response.status();
            if status.is_success() || status.as_u16() == 206 {
                let bytes = response.bytes().await?;
                tokio::fs::write(&filename, &bytes).await?;
                let actual_size = std::fs::metadata(&filename).map(|m| m.len()).unwrap_or(0);
                if actual_size == expected_size {
                    let elapsed = t0.elapsed().as_secs_f64();
                    log(&format!(
                        "[✓] Part {} finished ({:.1}s)",
                        part_name, elapsed
                    ));
                    Ok((true, elapsed))
                } else {
                    log(&format!(
                        "[X] Part {} download corrupted ({} bytes written).",
                        part_name, actual_size
                    ));
                    Ok((false, 0.0))
                }
            } else {
                log(&format!(
                    "[X] Part {} failed with status {}",
                    part_name,
                    status.as_u16()
                ));
                Ok((false, 0.0))
            }
        }
        Err(e) => {
            log(&format!("[X] Part {} error: {}", part_name, e));
            Ok((false, 0.0))
        }
    }
}

pub async fn proxy_health_checker(
    raw_proxies: Vec<String>,
    url: String,
    max_connections: u32,
    active_proxies: Arc<Mutex<Vec<String>>>,
    attempt_counter: Arc<AtomicU64>,
    download_done: Arc<tokio::sync::Notify>,
    log: Arc<dyn Fn(&str) + Send + Sync + 'static>,
) {
    let mut last_check: u64 = 0;
    let check_interval: u64 = if (raw_proxies.len() as f64 * 1.5) > max_connections as f64 {
        1000
    } else {
        200
    };

    loop {
        tokio::select! {
            _ = download_done.notified() => break,
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
        }

        let current = attempt_counter.load(Ordering::Acquire);
        if current - last_check >= check_interval {
            last_check = current;
            log(&format!("\n[*] Proxy health check at attempt {}...", current));
            let refreshed = filter_alive_proxies(&raw_proxies, &url, &log).await;
            if !refreshed.is_empty() {
                let mut ap = active_proxies.lock().await;
                *ap = refreshed;
            }
        }
    }
}
