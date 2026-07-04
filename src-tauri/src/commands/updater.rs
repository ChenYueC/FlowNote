use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Deserialize)]
struct UpdateManifest {
    version: String,
    #[serde(default)]
    download_proxy: Option<String>,
    url: String,
}

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub need_update: bool,
    pub version: String,
    pub download_url: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadProgress {
    pub percent: u8,
}

fn compare_versions(current: &str, remote: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|s| s.parse().ok()).collect()
    };
    let cur = parse(current);
    let rem = parse(remote);
    for i in 0..cur.len().max(rem.len()) {
        let c = cur.get(i).copied().unwrap_or(0);
        let r = rem.get(i).copied().unwrap_or(0);
        if r > c { return true; }
        if r < c { return false; }
    }
    false
}

#[tauri::command]
pub async fn check_update(manifest_url: String) -> Result<UpdateInfo, String> {
    let resp = reqwest::get(&manifest_url).await.map_err(|e| e.to_string())?;
    let manifest: UpdateManifest = resp.json().await.map_err(|e| e.to_string())?;

    let current_version = env!("CARGO_PKG_VERSION");
    let need_update = compare_versions(current_version, &manifest.version);

    let download_url = match &manifest.download_proxy {
        Some(proxy) if !proxy.is_empty() && proxy != "false" => {
            format!("{}{}", proxy, manifest.url)
        }
        _ => manifest.url,
    };

    Ok(UpdateInfo {
        need_update,
        version: manifest.version,
        download_url,
    })
}

#[tauri::command]
pub async fn download_and_install(app: AppHandle, url: String) -> Result<(), String> {
    let temp_dir = std::env::temp_dir();
    let filename = url.split('/').last().unwrap_or("FlowNote-setup.exe");
    let installer_path = temp_dir.join(filename);

    // Download with progress
    let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = resp.bytes_stream();

    use futures_util::StreamExt;
    let mut file = std::fs::File::create(&installer_path).map_err(|e| e.to_string())?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        std::io::Write::write_all(&mut file, &chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percent = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
            let _ = app.emit("update-download-progress", DownloadProgress { percent });
        }
    }

    // Emit 100% before running installer
    let _ = app.emit("update-download-progress", DownloadProgress { percent: 100 });

    // Close file before running installer (Windows locks open files)
    drop(file);

    // Run installer directly (open::that uses cmd /c start which fails for NSIS)
    std::process::Command::new(&installer_path)
        .spawn()
        .map_err(|e| format!("Failed to launch installer: {}", e))?;

    // Exit app immediately — spawn() is synchronous, installer is already running
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub async fn show_update_confirm(app: AppHandle, version: String) -> bool {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
    app.dialog()
        .message(format!("发现新版本 v{}，是否立即更新？", version))
        .title("更新提示")
        .buttons(MessageDialogButtons::OkCancelCustom("确认更新".into(), "稍后提醒".into()))
        .blocking_show()
}
