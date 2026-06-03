mod state;
mod download;
mod tui;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use clap::Parser;
use reqwest::Proxy;
use tokio::sync::{Mutex, Notify};

use download::{download_chunk, filter_alive_proxies, proxy_health_checker};
use state::{DynState, DynamicSemaphore, ProxyStats};
use tui::{PartStatus, TabState, TuiState, UiEvent};

#[derive(Parser)]
#[command(about = "Multi-Proxy Chunk Downloader (Rust port)")]
struct Args {
    #[arg(long)]
    url: String,

    #[arg(long, default_value_t = 5)]
    size: u32,

    #[arg(long)]
    size_kb: Option<u32>,

    #[arg(long, default_value_t = 5)]
    connections: u32,

    #[arg(long)]
    proxies: String,

    #[arg(long)]
    output: Option<String>,

    #[arg(long, default_value_t = String::from("."))]
    output_dir: String,

    #[arg(long)]
    timeout: Option<u64>,

    #[arg(long, default_value_t = 20)]
    connect_timeout: u32,

    #[arg(long, default_value_t = 35)]
    read_timeout: u32,

    #[arg(long, default_value_t = String::from("temp"))]
    temp_dir: String,

}

impl Args {
    fn chunk_size_bytes(&self) -> u64 {
        if let Some(kb) = self.size_kb {
            (kb as u64) * 1024
        } else {
            (self.size as u64) * 1024 * 1024
        }
    }

    fn chunk_size_mb_f64(&self) -> f64 {
        if let Some(kb) = self.size_kb {
            kb as f64 / 1024.0
        } else {
            self.size as f64
        }
    }
}

fn parse_proxies(raw: &str) -> Vec<String> {
    let path = Path::new(raw);
    if path.exists() {
        std::fs::read_to_string(raw)
            .unwrap_or_default()
            .lines()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("socks") || t.starts_with("http")
            })
            .map(|l| l.trim().to_string())
            .collect()
    } else {
        raw.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| s.starts_with("socks") || s.starts_with("http"))
            .collect()
    }
}

async fn get_file_size(url: &str) -> Result<u64> {
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

fn get_hex_name(part_num: u32) -> String {
    format!("{:x}", part_num)
}

fn assemble_file(
    total_parts: u32,
    dl_dir: &Path,
    output_path: &Path,
    log: &Arc<dyn Fn(&str) + Send + Sync>,
) {
    log("[*] All parts validated perfectly! Merging files into final output...");
    use std::io::Write;
    let mut final_file = std::fs::File::create(output_path).unwrap();
    for i in 0..total_parts {
        let part_path = dl_dir.join(format!("part_{}.tmp", get_hex_name(i)));
        let bytes = std::fs::read(&part_path).unwrap();
        final_file.write_all(&bytes).unwrap();
        let _ = std::fs::remove_file(&part_path);
    }
    let _ = std::fs::remove_dir(dl_dir);
    log(&format!(
        "[✓] Complete! Final file saved as: {}",
        output_path.display()
    ));
}

async fn process_url(
    url: &str,
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
    let file_name = url
        .split('/')
        .last()
        .unwrap_or("downloaded_file")
        .to_string();
    let dl_dir = temp_dir.join(&file_name);
    std::fs::create_dir_all(&dl_dir).unwrap();
    log(&format!("[*] Target download directory set to: {}", dl_dir.display()));

    log("[*] Connecting directly to VPS to fetch file size...");
    let file_size = match get_file_size(url).await {
        Ok(s) => s,
        Err(e) => {
            log(&format!("[-] Failed to get file size: {}", e));
            return;
        }
    };
    log(&format!(
        "[*] Total File Size: {:.2} GB ({} bytes)",
        file_size as f64 / 1_073_741_824.0,
        file_size
    ));

    let alive_proxies = filter_alive_proxies(proxy_urls, url, &log).await;
    if alive_proxies.is_empty() {
        log("[-] Error: No active proxies available at startup!");
        return;
    }

    let mut chunks: Vec<(u64, u64, u32)> = Vec::new();
    let mut start = 0u64;
    let mut part_num = 0u32;
    while start < file_size {
        let end = (start + chunk_size - 1).min(file_size - 1);
        chunks.push((start, end, part_num));
        start += chunk_size;
        part_num += 1;
    }
    let total_parts_needed = chunks.len() as u32;
    log(&format!("[*] Total planned parts: {}", total_parts_needed));

    // Scan for already-downloaded parts
    let initial_done: Vec<u32> = chunks
        .iter()
        .filter(|(start, end, p_num)| {
            let part_path = dl_dir.join(format!("part_{}.tmp", get_hex_name(*p_num)));
            if part_path.exists() {
                let expected = end - start + 1;
                std::fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0) == expected
            } else {
                false
            }
        })
        .map(|(_, _, p)| *p)
        .collect();
    let initial_count = initial_done.len() as u32;
    if initial_count > 0 {
        log(&format!(
            "[*] Found {} already-downloaded parts, skipping.",
            initial_count
        ));
        for pn in &initial_done {
            on_part_update(*pn, "finished", "(cached)", 0.0);
        }
        on_progress(initial_count, total_parts_needed);
    }

    let dyn_mc = dyn_state.max_connections.clone();
    let sem = Arc::new(DynamicSemaphore::with_getter_and_extra(
        max_connections,
        dyn_state.config_notify.clone(),
        move || dyn_mc.load(Ordering::Acquire),
    ));

    // Create one reqwest::Client per proxy (proxy baked into client)
    let proxy_clients: Vec<(String, reqwest::Client)> = alive_proxies
        .iter()
        .map(|u| {
            let ct = dyn_state.connect_timeout.load(Ordering::Acquire);
            let p = Proxy::all(u).unwrap();
            let c = reqwest::Client::builder()
                .proxy(p)
                .connect_timeout(std::time::Duration::from_secs(ct as u64))
                .build()
                .unwrap();
            (u.clone(), c)
        })
        .collect();

    let proxy_stats: Arc<Mutex<HashMap<String, ProxyStats>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let active_proxies: Arc<Mutex<Vec<String>>> =
        Arc::new(Mutex::new(
            proxy_clients.iter().map(|(u, _)| u.clone()).collect()
        ));
    let dead_proxies: Arc<Mutex<Vec<String>>> =
        Arc::new(Mutex::new(Vec::new()));
    let fast_tokens: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    let fast_idx: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    let explorer_idx: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    let attempt_counter = Arc::new(AtomicU64::new(0));
    let download_done = Arc::new(Notify::new());

    // Start health checker (re-tests dead proxies every 20 min, 1 at a time)
    let hc_dead = dead_proxies.clone();
    let hc_active = active_proxies.clone();
    let hc_url = url.to_string();
    let hc_done = download_done.clone();
    let hc_log = log.clone();
    tokio::spawn(async move {
        proxy_health_checker(hc_dead, hc_active, hc_url, hc_done, hc_log).await;
    });

    // Worker tasks — one per chunk
    let mut handles = Vec::new();
    let completed_parts = Arc::new(AtomicU64::new(initial_count as u64));
    let total_parts = chunks.len() as u32;
    for chunk in chunks {
        let dl_dir_c = dl_dir.clone();
        let url_c = url.to_string();
        let sem_c = sem.clone();
        let proxy_clients_c = proxy_clients.clone();
        let active_proxies_c = active_proxies.clone();
        let dead_proxies_c = dead_proxies.clone();
        let fast_tokens_c = fast_tokens.clone();
        let fast_idx_c = fast_idx.clone();
        let explorer_idx_c = explorer_idx.clone();
        let proxy_stats_c = proxy_stats.clone();
        let attempt_counter_c = attempt_counter.clone();
        let dyn_state_c = dyn_state.clone();
        let log_c = log.clone();
        let part_cb = on_part_update.clone();
        let proxy_cb = on_proxy_update.clone();
        let completed_c = completed_parts.clone();
        let prog_cb = on_progress.clone();
        let total = total_parts;

        handles.push(tokio::spawn(async move {
            let (start, end, p_num) = chunk;
            let mut success = false;

            while !success {
                dyn_state_c.wait_while_paused().await;

                let _guard = sem_c.acquire().await;
                // Re-check pause after acquiring semaphore (might have queued before pause)
                dyn_state_c.wait_while_paused().await;
                attempt_counter_c.fetch_add(1, Ordering::Release);

                // Sort proxies by avg speed and split into fast/explorer tiers
                let sorted_proxies: Vec<String> = {
                    let ap = active_proxies_c.lock().await;
                    let ps = proxy_stats_c.lock().await;
                    let mut pairs: Vec<(&String, f64)> = ap
                        .iter()
                        .map(|u| {
                            let avg = ps.get(u).map(|s| s.avg_time()).unwrap_or(f64::MAX);
                            (u, avg)
                        })
                        .collect();
                    pairs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                    pairs.into_iter().map(|(u, _)| u.clone()).collect()
                };

                let proxy_count = sorted_proxies.len();
                let current_max = dyn_state_c.max_connections.load(Ordering::Acquire);
                let fast_count = if proxy_count <= 1 {
                    0
                } else {
                    ((current_max as f64 * 0.9).floor() as usize).min(proxy_count - 1)
                };

                // Claim a fast token (atomic CAS) or fall back to explorer tier
                let is_fast = if fast_count > 0 {
                    loop {
                        let used = fast_tokens_c.load(Ordering::Acquire);
                        if used as usize >= fast_count {
                            break false;
                        }
                        if fast_tokens_c
                            .compare_exchange_weak(used, used + 1, Ordering::AcqRel, Ordering::Acquire)
                            .is_ok()
                        {
                            break true;
                        }
                    }
                } else {
                    false
                };

                let (proxy_url, client) = if is_fast {
                    let idx = fast_idx_c.fetch_add(1, Ordering::AcqRel) as usize % fast_count;
                    let u = sorted_proxies[idx].clone();
                    let c = proxy_clients_c
                        .iter()
                        .find(|(pu, _)| pu == &u)
                        .map(|(_, c)| c.clone())
                        .unwrap_or_else(|| {
                            let p = Proxy::all(&u).unwrap();
                            reqwest::Client::builder().proxy(p).build().unwrap()
                        });
                    (u, c)
                } else if proxy_count > fast_count {
                    let explorer_count = proxy_count - fast_count;
                    let idx = explorer_idx_c.fetch_add(1, Ordering::AcqRel) as usize % explorer_count;
                    let u = sorted_proxies[fast_count + idx].clone();
                    let c = proxy_clients_c
                        .iter()
                        .find(|(pu, _)| pu == &u)
                        .map(|(_, c)| c.clone())
                        .unwrap_or_else(|| {
                            let p = Proxy::all(&u).unwrap();
                            reqwest::Client::builder().proxy(p).build().unwrap()
                        });
                    (u, c)
                } else {
                    let idx = fast_idx_c.fetch_add(1, Ordering::AcqRel) as usize % proxy_count;
                    let u = sorted_proxies[idx].clone();
                    let c = proxy_clients_c
                        .iter()
                        .find(|(pu, _)| pu == &u)
                        .map(|(_, c)| c.clone())
                        .unwrap_or_else(|| {
                            let p = Proxy::all(&u).unwrap();
                            reqwest::Client::builder().proxy(p).build().unwrap()
                        });
                    (u, c)
                };

                let part_name = get_hex_name(p_num);
                log_c(&format!("[ ] Part {} → {}", part_name, proxy_url));
                part_cb(p_num, "downloading", "", 0.0);

                match download_chunk(
                    &client,
                    &url_c,
                    start,
                    end,
                    p_num,
                    &dl_dir_c,
                    &dyn_state_c,
                    &log_c,
                )
                .await
                {
                    Ok((true, elapsed)) => {
                        if elapsed > 0.0 {
                            let mut ps = proxy_stats_c.lock().await;
                            ps.entry(proxy_url.clone())
                                .or_insert_with(ProxyStats::new)
                                .record(elapsed);
                            let mut items: Vec<(String, f64, u64)> = ps
                                .iter()
                                .map(|(u, s)| (u.clone(), s.avg_time(), s.successes.load(Ordering::Acquire)))
                                .collect();
                            items.sort_by(|a, b| {
                                let sa = a.2 as f64 / a.1.max(0.001);
                                let sb = b.2 as f64 / b.1.max(0.001);
                                sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                            });
                            proxy_cb(items);
                            let done = completed_c.fetch_add(1, Ordering::Release) + 1;
                            prog_cb(done as u32, total);
                        }
                        success = true;
                        part_cb(p_num, "finished", &proxy_url, elapsed);
                    }
                    Ok((false, _)) => {
                        log_c(&format!(
                            "[X] Part {} failed, retrying...",
                            get_hex_name(p_num)
                        ));
                        part_cb(p_num, "error", "", 0.0);
                    }
                    Err(e) => {
                        log_c(&format!(
                            "[X] Part {} error: {}, retrying...",
                            get_hex_name(p_num),
                            e
                        ));
                        part_cb(p_num, "error", "", 0.0);
                        // Mark proxy as dead on connection-level failure
                        {
                            let mut ap = active_proxies_c.lock().await;
                            if let Some(pos) = ap.iter().position(|p| *p == proxy_url) {
                                ap.remove(pos);
                            }
                        }
                        {
                            let mut dp = dead_proxies_c.lock().await;
                            if !dp.contains(&proxy_url) {
                                dp.push(proxy_url.clone());
                            }
                        }
                    }
                }

                if is_fast {
                    fast_tokens_c.fetch_sub(1, Ordering::Release);
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }
    download_done.notify_waiters();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Validate
    log("\n[*] Validating downloaded parts before assembly...");
    let mut healthy = 0u32;
    for i in 0..total_parts_needed {
        let part_path = dl_dir.join(format!("part_{}.tmp", get_hex_name(i)));
        let expected_size = if (i as u64) < (total_parts_needed as u64 - 1) {
            chunk_size
        } else {
            file_size - (i as u64) * chunk_size
        };
        if part_path.exists()
            && std::fs::metadata(&part_path).map(|m| m.len()).unwrap_or(0) == expected_size
        {
            healthy += 1;
        }
    }
    on_progress(healthy, total_parts_needed);
    log(&format!(
        "[*] Integrity Status: {} / {} parts are complete.",
        healthy, total_parts_needed
    ));

    if healthy == total_parts_needed {
        assemble_file(total_parts_needed, &dl_dir, output_path, &log);
    } else {
        log(&format!(
            "[-] Error: Integrity check failed! Missing {} parts.",
            total_parts_needed - healthy
        ));
    }

    let _ = std::fs::remove_dir(&dl_dir);
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(kb) = args.size_kb {
        if kb == 0 {
            eprintln!("[-] Error: --size-kb must be at least 1");
            std::process::exit(1);
        }
    }

    let proxy_urls = parse_proxies(&args.proxies);
    if proxy_urls.is_empty() {
        eprintln!("[-] Error: No valid proxies found!");
        std::process::exit(1);
    }

    std::fs::create_dir_all(&args.output_dir)?;
    std::fs::create_dir_all(&args.temp_dir)?;

    let urls: Vec<String> = args.url.split(',').map(|s| s.trim().to_string()).collect();
    let url_count = urls.len();
    let timeout = args
        .timeout
        .unwrap_or_else(|| ((args.chunk_size_mb_f64() * 1024.0) as u64 / 56).max(30));

    // Pre-gather file info (blocking call inside a quick async block)
    let file_infos: Vec<Result<u32>> = {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            let mut infos = Vec::new();
            for url in &urls {
                println!("[*] Checking: {}", url);
                match get_file_size(url).await {
                    Ok(size) => {
                        let chunk_size = args.chunk_size_bytes();
                        let total_parts = ((size + chunk_size - 1) / chunk_size) as u32;
                        infos.push(Ok(total_parts));
                    }
                    Err(e) => {
                        infos.push(Err(anyhow!("{}: {}", url, e)));
                    }
                }
            }
            infos
        })
    };

    // Filter to valid downloads
    let mut tab_states = Vec::new();
    let mut download_urls = Vec::new();
    let mut output_paths = Vec::new();

    for (i, url) in urls.iter().enumerate() {
        let file_name = url.split('/').last().unwrap_or("downloaded_file");
        let output_path = if args.output.is_some() && url_count == 1 {
            PathBuf::from(args.output.as_ref().unwrap())
        } else {
            Path::new(&args.output_dir).join(file_name)
        };

        match &file_infos[i] {
            Ok(_total_parts) => {
                tab_states.push(TabState::new(url.clone()));
                download_urls.push(url.clone());
                output_paths.push(output_path);
            }
            Err(e) => {
                eprintln!("[-] Skipping {}: {}", url, e);
            }
        }
    }

    if download_urls.is_empty() {
        eprintln!("[-] No valid URLs to download!");
        std::process::exit(1);
    }

    let dyn_state = DynState::new(
        args.connections,
        timeout as u32,
        args.connect_timeout,
        args.read_timeout,
    );

    let (event_tx, event_rx) = crossbeam_channel::unbounded::<UiEvent>();

    let tui_state = TuiState {
        tabs: tab_states,
        current_tab: 0,
        total_files: download_urls.len(),
        current_file: 0,
        dyn_state: dyn_state.clone(),
        total_proxies: proxy_urls.len(),
        paused: false,
        all_done: false,
    };

    // Spawn download thread
    let d_tx = event_tx.clone();
    let d_dyn = dyn_state.clone();
    let d_urls = download_urls.clone();
    let d_paths = output_paths.clone();
    let d_proxies = proxy_urls.clone();
    let d_temp = args.temp_dir.clone();
    let d_chunk_size = args.chunk_size_bytes();
    let d_conn = args.connections;

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
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
            }

            let _ = d_tx.send(UiEvent::AllDone);
        });
    });

    // Run TUI on main thread
    tui::run_tui(tui_state, event_rx)?;

    Ok(())
}
