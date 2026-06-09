#[derive(Clone, Debug, PartialEq)]
pub enum DownloadStatus {
    Downloading,
    Completed,
    Paused,
    Error,
    Queued,
}

impl DownloadStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Downloading => "Downloading",
            Self::Completed => "Completed",
            Self::Paused => "Paused",
            Self::Error => "Error",
            Self::Queued => "Queued",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DownloadItem {
    pub id: u64,
    pub file_name: String,
    pub status: DownloadStatus,
    pub size: u64,
    pub downloaded: u64,
    pub speed: u64,
    pub progress: f32,
    pub remaining: String,
    pub added_date: String,
}

pub trait DownloadProvider {
    fn downloads(&self) -> Vec<DownloadItem>;
    fn categories(&self) -> Vec<(String, usize)>;
}

use rand::Rng;

pub struct MockDownloadProvider;

impl MockDownloadProvider {
    pub fn new() -> Self {
        Self
    }
}

impl DownloadProvider for MockDownloadProvider {
    fn downloads(&self) -> Vec<DownloadItem> {
        let names = [
            "ubuntu-24.04-desktop-amd64.iso",
            "python-3.12.0-amd64.exe",
            "vscode_1.95.3_amd64.deb",
            "node-v22.0.0-x64.msi",
            "docker-desktop-4.34.0-amd64.exe",
            "ffmpeg-7.1-full_build.zip",
            "libreoffice-24.8.3_Win_x86-64.msi",
            "goland-2024.3.1.exe",
            "postgresql-17.2-1-windows-x64.exe",
            "rust-1.83.0-x86_64-pc-windows-msvc.msi",
            "nginx-1.26.2.zip",
            "mysql-9.1.0-winx64.msi",
            "git-2.47.1-64-bit.exe",
            "teamviewer_15.58.4.exe",
            "steam_latest.exe",
            "discord-0.0.616.exe",
            "obs-studio-31.0.1-full.zip",
            "blender-4.3.2-windows-x64.msi",
            "gimp-2.10.38-setup.exe",
            "inkscape-1.4.1-x64.exe",
            "virtualbox-7.1.4-165100-Win.exe",
            "vagrant_2.4.3_windows_amd64.msi",
            "terraform_1.10.3_windows_amd64.zip",
            "kubernetes-node-v1.32.0.msi",
            "dotnet-sdk-9.0.102-win-x64.exe",
            "jdk-21_windows-x64_bin.exe",
            "android-studio-2024.3.1.12-windows.exe",
            "unityhub-3.10.0.exe",
            "matlab_R2024b_Win64.iso",
            "autocad-2025-win64.exe",
            "photoshop-2025-win64.exe",
            "office-2024-pro-win64.iso",
            "visual-studio-2022-community.exe",
            "sql-server-2022-developer.exe",
            "mongodb-8.0.4-windows-x64.msi",
            "redis-7.4.1-Windows-x64.msi",
            "elasticsearch-8.17.0-windows-x86_64.zip",
            "kibana-8.17.0-windows-x86_64.zip",
            "grafana-11.4.0.windows-amd64.msi",
            "prometheus-3.1.0.windows-amd64.zip",
            "wolfram-mathematica-14.1-win64.iso",
            "solidworks-2025-sp0-win64.iso",
            "catia-v6-2025-win64.iso",
            "ansys-2025-r1-win64.iso",
            "comsol-6.3-win64.iso",
            "stata-19-win64.exe",
            "spss-29-win64.exe",
            "sas-9.4-win64.iso",
            "eviews-14-win64.exe",
            "minitab-22-win64.exe",
        ];
        let statuses = [
            DownloadStatus::Downloading,
            DownloadStatus::Completed,
            DownloadStatus::Paused,
            DownloadStatus::Error,
            DownloadStatus::Queued,
        ];
        let mut rng = rand::thread_rng();
        names
            .into_iter()
            .enumerate()
            .map(|(i, name)| {
                let status = statuses[rng.gen_range(0..statuses.len())].clone();
                let size = rng.gen_range(50_000_000u64..8_000_000_000u64);
                let downloaded = match &status {
                    DownloadStatus::Completed => size,
                    DownloadStatus::Downloading => rng.gen_range(0..size),
                    _ => 0,
                };
                let progress = if size > 0 {
                    downloaded as f32 / size as f32
                } else {
                    0.0
                };
                let speed = if status == DownloadStatus::Downloading {
                    rng.gen_range(500_000..50_000_000)
                } else {
                    0
                };
                let remaining = if speed > 0 && progress < 1.0 {
                    let secs = (size - downloaded) / speed;
                    format!("{}m {}s", secs / 60, secs % 60)
                } else {
                    "-".to_string()
                };
                DownloadItem {
                    id: i as u64,
                    file_name: name.to_string(),
                    status,
                    size,
                    downloaded,
                    speed,
                    progress,
                    remaining,
                    added_date: format!(
                        "2025-{:02}-{:02}",
                        rng.gen_range(1..12),
                        rng.gen_range(1..28)
                    ),
                }
            })
            .collect()
    }

    fn categories(&self) -> Vec<(String, usize)> {
        let downloads = self.downloads();
        let total = downloads.len();
        let downloading = downloads.iter().filter(|d| d.status == DownloadStatus::Downloading).count();
        let completed = downloads.iter().filter(|d| d.status == DownloadStatus::Completed).count();
        let paused = downloads.iter().filter(|d| d.status == DownloadStatus::Paused).count();
        let error = downloads.iter().filter(|d| d.status == DownloadStatus::Error).count();
        let inactive = downloads.iter().filter(|d| d.status == DownloadStatus::Queued).count();
        vec![
            ("All".into(), total),
            ("Downloading".into(), downloading),
            ("Completed".into(), completed),
            ("Inactive".into(), inactive),
            ("Software Updates".into(), 0),
            ("Uncategorized".into(), error + paused),
        ]
    }
}
