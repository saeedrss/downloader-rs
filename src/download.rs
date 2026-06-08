use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{anyhow, Result};
use reqwest::Proxy;
use tokio::sync::Mutex;

use crate::state::DynState;

pub async fn test_single_proxy(proxy_url: &str, test_url: &str, timeout_secs: u64) -> bool {
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
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .send()
        .await
    {
        Ok(resp) => resp.status().is_success() || resp.status().as_u16() == 206,
        Err(_) => false,
    }
}

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

    let mut last_err = anyhow!("All proxies exhausted");
    for proxy_url in proxies {
        log(&format!("[*] Trying proxy {} to fetch file info...", proxy_url));
        match get_file_size_via_proxy(url, proxy_url).await {
            Ok(size) => return Ok(size),
            Err(e) => {
                log(&format!("[-] Proxy {} failed: {}", proxy_url, e));
                last_err = e;
            }
        }
    }

    Err(last_err)
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
            test_single_proxy(&u, &t, 8).await
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
    dead_proxies: Arc<Mutex<Vec<String>>>,
    active_proxies: Arc<Mutex<Vec<String>>>,
    test_url: String,
    download_done: Arc<tokio::sync::Notify>,
    log: Arc<dyn Fn(&str) + Send + Sync + 'static>,
) {
    loop {
        tokio::select! {
            _ = download_done.notified() => break,
            _ = tokio::time::sleep(std::time::Duration::from_secs(1200)) => {}
        }

        let proxy = {
            let mut dead = dead_proxies.lock().await;
            if dead.is_empty() {
                continue;
            }
            dead.remove(0)
        };

        log(&format!("[*] Re-testing dead proxy: {}", proxy));
        if test_single_proxy(&proxy, &test_url, 20).await {
            log(&format!("[+] Dead proxy revived: {}", proxy));
            active_proxies.lock().await.push(proxy);
        } else {
            log(&format!("[-] Proxy still dead: {}", proxy));
            dead_proxies.lock().await.push(proxy);
        }
    }
}
