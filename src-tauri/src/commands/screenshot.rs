use crate::AppState;
use chrono::Utc;
use std::fs;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;
use xcap::Monitor;

#[cfg(target_os = "windows")]
use windows::core::w;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::*;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::*;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{SetCapture, ReleaseCapture, SetFocus};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::*;

// Timer ID for cursor tracking
#[cfg(target_os = "windows")]
const CURSOR_TRACK_TIMER: usize = 9999;

// Global list of overlay hwnds (as isize) for cursor tracking
#[cfg(target_os = "windows")]
static OVERLAY_HWNDS: std::sync::Mutex<Vec<(isize, i32, i32, i32, i32)>> = std::sync::Mutex::new(Vec::new());

// ── Public commands ──────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct MonitorInfo {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub name: String,
}

#[tauri::command]
pub async fn get_all_monitors() -> Result<Vec<MonitorInfo>, String> {
    tokio::task::spawn_blocking(|| {
        let monitors = Monitor::all().map_err(|e| e.to_string())?;
        Ok(monitors
            .iter()
            .map(|m| MonitorInfo {
                x: m.x(),
                y: m.y(),
                width: m.width(),
                height: m.height(),
                scale_factor: m.scale_factor(),
                name: m.name().to_string(),
            })
            .collect())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Show native overlay and return the selected region coordinates (physical pixels).
/// Used by the settings page to let the user pick a fixed screenshot area.
#[tauri::command]
pub async fn select_screen_area() -> Result<(i32, i32, u32, u32), String> {
    let (x, y, w, h) = {
        let handle = std::thread::spawn(show_native_overlay);
        handle.join().map_err(|_| "Overlay thread panicked".to_string())??
    };
    if w < 10 || h < 10 {
        return Err("Selection too small".to_string());
    }
    Ok((x, y, w, h))
}

/// Capture a fixed area using physical pixel coordinates (stored from select_screen_area).
#[tauri::command]
pub async fn capture_fixed_area(
    app: AppHandle,
    state: State<'_, AppState>,
    note_id: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let assets_dir = app
        .path()
        .app_local_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("assets");
    fs::create_dir_all(&assets_dir).map_err(|e| e.to_string())?;

    let filename = format!("screenshot_{}.png", id);
    let dest_path = assets_dir.join(&filename);
    let dest_clone = dest_path.clone();

    let captured_path = tokio::task::spawn_blocking(move || {
        capture_physical_region(x, y, width as u32, height as u32, &dest_clone)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO images (id, note_id, path, width, height, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&note_id)
    .bind(&captured_path)
    .bind(width)
    .bind(height)
    .bind(now)
    .execute(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(captured_path)
}

/// Native Win32 overlay → user selects region → capture → save → return path.
/// The overlay uses physical pixel coordinates from GetCursorPos, so
/// multi-monitor + DPI works correctly without any webview involvement.
#[tauri::command]
pub async fn select_and_capture_screenshot(
    app: AppHandle,
    state: State<'_, AppState>,
    note_id: String,
) -> Result<String, String> {
    // Show native overlay on a dedicated thread
    let (x, y, w, h) = {
        let handle = std::thread::spawn(show_native_overlay);
        handle.join().map_err(|_| "Overlay thread panicked".to_string())??
    };

    if w < 10 || h < 10 {
        return Err("Selection too small".to_string());
    }

    // Wait for overlay to fully close and screen to refresh
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Capture the selected region using physical pixel coords directly
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let assets_dir = app
        .path()
        .app_local_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("assets");
    fs::create_dir_all(&assets_dir).map_err(|e| e.to_string())?;

    let filename = format!("screenshot_{}.png", id);
    let dest_path = assets_dir.join(&filename);

    let dest_clone = dest_path.clone();
    let captured_path = tokio::task::spawn_blocking(move || {
        capture_physical_region(x, y, w, h, &dest_clone)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    // Save to database
    sqlx::query(
        "INSERT INTO images (id, note_id, path, width, height, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&note_id)
    .bind(&captured_path)
    .bind(w as i32)
    .bind(h as i32)
    .bind(now)
    .execute(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(captured_path)
}

/// Original capture command (used by fixed-area screenshots with logical coords).
#[tauri::command]
pub async fn capture_screenshot(
    app: AppHandle,
    state: State<'_, AppState>,
    note_id: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<String, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().timestamp();

    let assets_dir = app
        .path()
        .app_local_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("assets");
    fs::create_dir_all(&assets_dir).map_err(|e| e.to_string())?;

    let filename = format!("screenshot_{}.png", id);
    let dest_path = assets_dir.join(&filename);

    let dest_clone = dest_path.clone();
    let captured_path = tokio::task::spawn_blocking(move || {
        capture_region(x, y, width as u32, height as u32, &dest_clone)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "INSERT INTO images (id, note_id, path, width, height, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&note_id)
    .bind(&captured_path)
    .bind(width)
    .bind(height)
    .bind(now)
    .execute(&state.db.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(captured_path)
}

#[tauri::command]
pub async fn capture_fullscreen_screenshot(
    app: AppHandle,
    state: State<'_, AppState>,
    note_id: String,
) -> Result<String, String> {
    let (x, y, width, height) = tokio::task::spawn_blocking(|| {
        let monitors = Monitor::all().map_err(|e| e.to_string())?;
        if monitors.is_empty() {
            return Err("No monitor found".to_string());
        }
        let min_x = monitors.iter().map(|m| m.x()).min().unwrap();
        let min_y = monitors.iter().map(|m| m.y()).min().unwrap();
        let max_x = monitors.iter().map(|m| m.x() + m.width() as i32).max().unwrap();
        let max_y = monitors.iter().map(|m| m.y() + m.height() as i32).max().unwrap();
        Ok::<(i32, i32, i32, i32), String>((min_x, min_y, max_x - min_x, max_y - min_y))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    capture_screenshot(app, state, note_id, x, y, width, height).await
}

// ── Internal helpers ─────────────────────────────────────────────────

/// Capture a region using PHYSICAL pixel coordinates (from Win32 overlay).
/// Uses Win32 API to find the monitor, then xcap to capture and crop.
#[cfg(target_os = "windows")]
fn capture_physical_region(
    phys_x: i32,
    phys_y: i32,
    phys_w: u32,
    phys_h: u32,
    dest_path: &std::path::Path,
) -> Result<String, String> {
    unsafe {
        use windows::Win32::Graphics::Gdi::*;

        // Find the monitor at the selection's top-left corner using Win32
        let pt = POINT { x: phys_x, y: phys_y };
        let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let _ = GetMonitorInfoW(hmon, &mut mi);
        let mr = mi.rcMonitor;
        let mon_phys_x = mr.left;
        let mon_phys_y = mr.top;
        let mon_phys_w = (mr.right - mr.left) as u32;
        let mon_phys_h = (mr.bottom - mr.top) as u32;

        // Find matching xcap monitor (by position) to get scale_factor
        let xcap_monitors = Monitor::all().map_err(|e| e.to_string())?;
        let xcap_mon = xcap_monitors.iter().find(|m| {
            m.x() == mon_phys_x && m.y() == mon_phys_y
        }).or_else(|| xcap_monitors.first())
        .ok_or("No xcap monitor found")?;
        let scale = xcap_mon.scale_factor();
        let _xcap_w = xcap_mon.width();
        let _xcap_h = xcap_mon.height();

        // Capture the entire monitor
        let monitor_image = xcap_mon.capture_image().map_err(|e| e.to_string())?;
        let img_w = monitor_image.width();
        let img_h = monitor_image.height();

        // Compute crop in the captured image's pixel space
        // offset within monitor (physical pixels)
        let off_x = (phys_x - mon_phys_x).max(0).min(mon_phys_w as i32);
        let off_y = (phys_y - mon_phys_y).max(0).min(mon_phys_h as i32);
        let crop_w = phys_w.min(mon_phys_w as u32 - off_x as u32);
        let crop_h = phys_h.min(mon_phys_h as u32 - off_y as u32);

        // Map physical offset to image pixel coordinates
        let ix = (off_x as f32 * img_w as f32 / mon_phys_w as f32) as u32;
        let iy = (off_y as f32 * img_h as f32 / mon_phys_h as f32) as u32;
        let iw = (crop_w as f32 * img_w as f32 / mon_phys_w as f32).round() as u32;
        let ih = (crop_h as f32 * img_h as f32 / mon_phys_h as f32).round() as u32;

        // Clamp to image bounds
        let ix = ix.min(img_w);
        let iy = iy.min(img_h);
        let iw = iw.min(img_w.saturating_sub(ix));
        let ih = ih.min(img_h.saturating_sub(iy));
        if iw == 0 || ih == 0 {
            return Err("Crop region is empty".to_string());
        }

        let cropped = image::imageops::crop_imm(&monitor_image, ix, iy, iw, ih).to_image();

        // Resize to logical pixel output size
        let out_w = (crop_w as f32 / scale).round() as u32;
        let out_h = (crop_h as f32 / scale).round() as u32;
        let final_image = if (scale - 1.0f32).abs() > 0.001 && out_w > 0 && out_h > 0 {
            image::imageops::resize(&cropped, out_w, out_h, image::imageops::FilterType::Lanczos3)
        } else {
            cropped
        };

        let dyn_image = image::DynamicImage::ImageRgba8(final_image);
        dyn_image.save(dest_path).map_err(|e| e.to_string())?;
        Ok(dest_path.to_string_lossy().to_string())
    }
}

#[cfg(not(target_os = "windows"))]
fn capture_physical_region(
    _phys_x: i32, _phys_y: i32, _phys_w: u32, _phys_h: u32,
    _dest_path: &std::path::Path,
) -> Result<String, String> {
    Err("Physical capture only supported on Windows".to_string())
}

/// Convert physical pixel coordinates (from GetCursorPos) to logical pixels
/// by finding the monitor that contains the point and applying its scale factor.
#[allow(dead_code)]
fn physical_to_logical(
    phys_x: i32,
    phys_y: i32,
    phys_w: u32,
    phys_h: u32,
) -> Result<(i32, i32, u32, u32), String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    // Find which monitor contains the top-left corner
    let mon = monitors
        .iter()
        .find(|m| {
            let mx = (m.x() as f32 * m.scale_factor()) as i32;
            let my = (m.y() as f32 * m.scale_factor()) as i32;
            let mw = (m.width() as f32 * m.scale_factor()) as i32;
            let mh = (m.height() as f32 * m.scale_factor()) as i32;
            phys_x >= mx && phys_x < mx + mw && phys_y >= my && phys_y < my + mh
        })
        .or_else(|| monitors.first())
        .ok_or("No monitor found")?;

    let scale = mon.scale_factor();
    let log_x = (phys_x as f32 / scale) as i32;
    let log_y = (phys_y as f32 / scale) as i32;
    let log_w = (phys_w as f32 / scale).round() as u32;
    let log_h = (phys_h as f32 / scale).round() as u32;
    Ok((log_x, log_y, log_w, log_h))
}

/// Capture a region (logical pixel coordinates) spanning one or more monitors.
fn capture_region(
    area_x: i32,
    area_y: i32,
    area_w: u32,
    area_h: u32,
    dest_path: &std::path::Path,
) -> Result<String, String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    if monitors.is_empty() {
        return Err("No monitor found".to_string());
    }

    let mut canvas =
        image::RgbaImage::from_raw(area_w, area_h, vec![0u8; (area_w * area_h * 4) as usize])
            .ok_or("Failed to create image buffer")?;

    for monitor in &monitors {
        let mon_x = monitor.x();
        let mon_y = monitor.y();
        let mon_w = monitor.width() as i32;
        let mon_h = monitor.height() as i32;
        let scale = monitor.scale_factor();

        let ix1 = area_x.max(mon_x);
        let iy1 = area_y.max(mon_y);
        let ix2 = (area_x + area_w as i32).min(mon_x + mon_w);
        let iy2 = (area_y + area_h as i32).min(mon_y + mon_h);

        if ix1 >= ix2 || iy1 >= iy2 {
            continue;
        }

        let crop_lx = (ix1 - mon_x) as u32;
        let crop_ly = (iy1 - mon_y) as u32;
        let crop_lw = (ix2 - ix1) as u32;
        let crop_lh = (iy2 - iy1) as u32;

        let px = (crop_lx as f32 * scale) as u32;
        let py = (crop_ly as f32 * scale) as u32;
        let pw = (crop_lw as f32 * scale).round() as u32;
        let ph = (crop_lh as f32 * scale).round() as u32;

        let monitor_image = monitor.capture_image().map_err(|e| e.to_string())?;
        let img_w = monitor_image.width();
        let img_h = monitor_image.height();

        let px = px.min(img_w);
        let py = py.min(img_h);
        let pw = pw.min(img_w.saturating_sub(px));
        let ph = ph.min(img_h.saturating_sub(py));
        if pw == 0 || ph == 0 {
            continue;
        }

        let cropped = image::imageops::crop_imm(&monitor_image, px, py, pw, ph).to_image();
        let resized = if (scale - 1.0f32).abs() > 0.001 {
            image::imageops::resize(&cropped, crop_lw, crop_lh, image::imageops::FilterType::Lanczos3)
        } else {
            cropped
        };

        let dst_x = (ix1 - area_x) as i32;
        let dst_y = (iy1 - area_y) as i32;
        paste_rgba(&mut canvas, &resized, dst_x, dst_y);
    }

    let dyn_image = image::DynamicImage::ImageRgba8(canvas);
    dyn_image.save(dest_path).map_err(|e| e.to_string())?;
    Ok(dest_path.to_string_lossy().to_string())
}

fn paste_rgba(dst: &mut image::RgbaImage, src: &image::RgbaImage, ox: i32, oy: i32) {
    let dw = dst.width() as i32;
    let dh = dst.height() as i32;
    let sw = src.width() as i32;
    let sh = src.height() as i32;
    let x_start = ox.max(0);
    let y_start = oy.max(0);
    let x_end = (ox + sw).min(dw);
    let y_end = (oy + sh).min(dh);
    for dy in y_start..y_end {
        for dx in x_start..x_end {
            let sx = (dx - ox) as u32;
            let sy = (dy - oy) as u32;
            dst.put_pixel(dx as u32, dy as u32, *src.get_pixel(sx, sy));
        }
    }
}

// ── Native Win32 overlay ─────────────────────────────────────────────

#[cfg(target_os = "windows")]
struct OverlayState {
    start_x: i32,
    start_y: i32,
    cur_x: i32,
    cur_y: i32,
    dragging: bool,
    done: bool,
    cancelled: bool,
    win_w: i32,
    win_h: i32,
}

/// Render the overlay using UpdateLayeredWindow (per-pixel alpha, zero flicker).
/// Overlay area: semi-transparent black. Selection: fully transparent (shows real screen).
#[cfg(target_os = "windows")]
unsafe fn update_overlay(hwnd: HWND, s: &OverlayState) {
    let w = s.win_w;
    let h = s.win_h;
    if w <= 0 || h <= 0 { return; }

    let screen_dc = GetDC(None);
    let mem_dc = CreateCompatibleDC(screen_dc);

    let bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h, // top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let bitmap = CreateDIBSection(mem_dc, &bmi, DIB_RGB_COLORS, &mut bits, None, 0);
    let bitmap = match bitmap {
        Ok(b) => b,
        Err(_) => { let _ = DeleteDC(mem_dc); let _ = ReleaseDC(None, screen_dc); return; }
    };
    let old_bmp = SelectObject(mem_dc, bitmap);

    // Fill entire bitmap: semi-transparent black (premultiplied alpha)
    // alpha=100 out of 255 → visible dimming
    let alpha: u8 = 100;
    let premul = |c: u8| -> u8 { ((c as u16 * alpha as u16) / 255) as u8 };
    let overlay_pixel: u32 = (alpha as u32) << 24
        | (premul(0) as u32) << 16
        | (premul(0) as u32) << 8
        | premul(0) as u32;

    let pixels = std::slice::from_raw_parts_mut(bits as *mut u32, (w * h) as usize);
    pixels.fill(overlay_pixel);

    if s.dragging || s.done {
        let x1 = s.start_x.min(s.cur_x).max(0) as usize;
        let y1 = s.start_y.min(s.cur_y).max(0) as usize;
        let x2 = (s.start_x.max(s.cur_x) as usize).min(w as usize);
        let y2 = (s.start_y.max(s.cur_y) as usize).min(h as usize);

        // Selection area: fully transparent (alpha=0) → shows real screen
        for y in y1..y2 {
            let row = y * w as usize;
            for x in x1..x2 {
                pixels[row + x] = 0x00000000;
            }
        }

        // Draw selection border (3px, solid color, fully opaque)
        let border_color: u32 = 0xFF93F3C5; // ARGB: #93F3C5
        let bw = 3usize;
        // Top/bottom borders
        for y in y1..y1.min(y2).checked_add(bw).unwrap_or(y2).min(y2) {
            for x in x1..x2 { pixels[y * w as usize + x] = border_color; }
        }
        for y in y2.saturating_sub(bw)..y2 {
            for x in x1..x2 { pixels[y * w as usize + x] = border_color; }
        }
        // Left/right borders
        for y in y1..y2 {
            for x in x1..x1.min(x2).checked_add(bw).unwrap_or(x2).min(x2) {
                pixels[y * w as usize + x] = border_color;
            }
            for x in x2.saturating_sub(bw)..x2 {
                pixels[y * w as usize + x] = border_color;
            }
        }

        // Size label (simple text at top-left of selection)
        let sel_w = x2.saturating_sub(x1);
        let sel_h = y2.saturating_sub(y1);
        if sel_w > 30 && sel_h > 20 {
            let label = format!("{} \u{00d7} {}", sel_w, sel_h);
            let text_color: u32 = 0xFFFFFFFF;
            // Draw text background (small dark rect behind text)
            let tw = (label.len() * 8 + 8).min(sel_w);
            let th = 18usize.min(sel_h);
            let ty = y1.saturating_sub(th + 2);
            let tx = x1 + 2;
            for y in ty..ty + th {
                if y < h as usize {
                    for x in tx..tx + tw {
                        if x < w as usize {
                            pixels[y * w as usize + x] = 0xCC000000; // dark bg
                        }
                    }
                }
            }
            // Draw each character (very basic bitmap font - just mark pixels)
            let wide: Vec<u16> = label.encode_utf16().collect();
            let text_dc = CreateCompatibleDC(mem_dc);
            let text_bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: tw as i32,
                    biHeight: -(th as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut text_bits: *mut std::ffi::c_void = std::ptr::null_mut();
            if let Ok(text_bmp) = CreateDIBSection(text_dc, &text_bmi, DIB_RGB_COLORS, &mut text_bits, None, 0) {
                let old_text_bmp = SelectObject(text_dc, text_bmp);
                let text_pixels = std::slice::from_raw_parts_mut(text_bits as *mut u32, tw * th);
                text_pixels.fill(0x00000000);
                SetTextColor(text_dc, COLORREF(0x00FFFFFF));
                SetBkMode(text_dc, TRANSPARENT);
                let _ = TextOutW(text_dc, 2, 1, &wide);
                // Copy text pixels to main bitmap
                for y in 0..th {
                    let dest_y = ty + y;
                    if dest_y < h as usize {
                        for x in 0..tw {
                            let dest_x = tx + x;
                            if dest_x < w as usize {
                                let tp = text_pixels[y * tw + x];
                                let ta = (tp >> 24) & 0xFF;
                                if ta == 0 && (tp & 0xFFFFFF) != 0 {
                                    // GDI drew white text on black bg (alpha stripped)
                                    pixels[dest_y * w as usize + dest_x] = text_color;
                                }
                            }
                        }
                    }
                }
                SelectObject(text_dc, old_text_bmp);
                let _ = DeleteObject(text_bmp);
            }
            let _ = DeleteDC(text_dc);
        }
    }

    // Push bitmap to window via UpdateLayeredWindow
    let pt_dst = POINT { x: 0, y: 0 };
    let sz = SIZE { cx: w, cy: h };
    let blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as u8,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA as u8,
    };
    let _ = UpdateLayeredWindow(hwnd, screen_dc, None, Some(&sz), mem_dc, Some(&pt_dst), COLORREF(0), Some(&blend), ULW_ALPHA);

    SelectObject(mem_dc, old_bmp);
    let _ = DeleteObject(bitmap);
    let _ = DeleteDC(mem_dc);
        let _ = ReleaseDC(None, screen_dc);
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn overlay_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut OverlayState;
        if state_ptr.is_null() && msg != WM_NCCREATE {
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }
        let state = if !state_ptr.is_null() {
            Some(&mut *state_ptr)
        } else {
            None
        };

        match msg {
            WM_LBUTTONDOWN => {
                if let Some(s) = state {
                    s.start_x = ((lparam.0 & 0xFFFF) as i16) as i32;
                    s.start_y = (((lparam.0 >> 16) & 0xFFFF) as i16) as i32;
                    s.cur_x = s.start_x;
                    s.cur_y = s.start_y;
                    s.dragging = true;
                    SetCapture(hwnd);
                    update_overlay(hwnd, s);
                }
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                if let Ok(c) = LoadCursorW(None, IDC_CROSS) { SetCursor(c); }
                if let Some(s) = state {
                    if s.dragging {
                        s.cur_x = ((lparam.0 & 0xFFFF) as i16) as i32;
                        s.cur_y = (((lparam.0 >> 16) & 0xFFFF) as i16) as i32;
                        update_overlay(hwnd, s);
                    }
                }
                LRESULT(0)
            }
            WM_TIMER => {
                if wparam.0 == CURSOR_TRACK_TIMER {
                    if let Ok(c) = LoadCursorW(None, IDC_CROSS) { SetCursor(c); }
                    if let Ok(windows) = OVERLAY_HWNDS.lock() {
                        let mut cursor = POINT::default();
                        let _ = GetCursorPos(&mut cursor);
                        for &(wh_raw, mx, my, mw, mh) in windows.iter() {
                            if cursor.x >= mx && cursor.x < mx + mw
                                && cursor.y >= my && cursor.y < my + mh
                            {
                                SetCapture(HWND(wh_raw as *mut _));
                                break;
                            }
                        }
                    }
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                if let Some(s) = state {
                    s.cur_x = ((lparam.0 & 0xFFFF) as i16) as i32;
                    s.cur_y = (((lparam.0 >> 16) & 0xFFFF) as i16) as i32;
                    s.dragging = false;
                    s.done = true;
                    let _ = ReleaseCapture();
                    PostQuitMessage(0);
                }
                LRESULT(0)
            }
            WM_KEYDOWN | WM_KEYUP => {
                if wparam.0 == 27 {
                    // VK_ESCAPE
                    if let Some(s) = state {
                        s.cancelled = true;
                        s.done = true;
                        PostQuitMessage(0);
                    }
                }
                LRESULT(0)
            }
            WM_PAINT => {
                // Layered windows use UpdateLayeredWindow, not WM_PAINT
                let mut ps = PAINTSTRUCT::default();
                BeginPaint(hwnd, &mut ps);
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_SETCURSOR => {
                if let Ok(c) = LoadCursorW(None, IDC_CROSS) { SetCursor(c); }
                LRESULT(1)
            }
            WM_NCHITTEST => {
                // Layered windows default to HTTRANSPARENT; force HTCLIENT
                // so that WM_SETCURSOR is dispatched to this window
                LRESULT(HTCLIENT as isize)
            }
            WM_ERASEBKGND => LRESULT(1), // prevent flicker
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Show native overlay on ALL monitors, let user select, return physical pixel coords.
#[cfg(target_os = "windows")]
fn show_native_overlay() -> Result<(i32, i32, u32, u32), String> {
    unsafe {
        // Use xcap to enumerate all monitors
        let xcap_monitors = Monitor::all().map_err(|e| e.to_string())?;
        if xcap_monitors.is_empty() {
            return Err("No monitors found".to_string());
        }

        // Collect monitor rects in physical pixels using Win32 API
        let mut monitor_rects: Vec<(i32, i32, i32, i32)> = Vec::new();
        for xm in &xcap_monitors {
            let pt = POINT { x: xm.x() + 1, y: xm.y() + 1 };
            let hmon = MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            let _ = GetMonitorInfoW(hmon, &mut mi);
            let r = mi.rcMonitor;
            monitor_rects.push((r.left, r.top, r.right - r.left, r.bottom - r.top));
        }

        // Register window class
        let class_name = w!("FlowNoteScreenshotOverlay");
        let _ = UnregisterClassW(class_name, None);
        let cross_cursor = LoadCursorW(None, IDC_CROSS).unwrap_or_default();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(overlay_wndproc),
            lpszClassName: class_name.into(),
            hCursor: cross_cursor,
            ..Default::default()
        };
        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            let err = GetLastError();
            return Err(format!("RegisterClassExW failed: {:?}", err));
        }

        // Create one overlay window per monitor
        let mut hwnds: Vec<HWND> = Vec::new();
        let mut states: Vec<*mut OverlayState> = Vec::new();

        for (_i, &(mx, my, mw, mh)) in monitor_rects.iter().enumerate() {
            let hwnd = match CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name,
                w!(""),
                WS_POPUP,
                mx, my, mw, mh,
                None, None, None, None,
            ) {
                Ok(h) => h,
                Err(_) => {
                    continue;
                }
            };

            let state = Box::into_raw(Box::new(OverlayState {
                start_x: 0, start_y: 0, cur_x: 0, cur_y: 0,
                dragging: false, done: false, cancelled: false,
                win_w: mw, win_h: mh,
            }));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize);

            let _ = ShowWindow(hwnd, SW_SHOW);
            // Initial render via UpdateLayeredWindow
            update_overlay(hwnd, &*state);
            hwnds.push(hwnd);
            states.push(state);
        }

        if hwnds.is_empty() {
            let _ = UnregisterClassW(class_name, None);
            return Err("Failed to create any overlay windows".to_string());
        }

        // Set focus and capture on the first window initially
        let _ = SetForegroundWindow(hwnds[0]);
        let _ = SetFocus(hwnds[0]);
        let _ = SetCapture(hwnds[0]);

        // Set crosshair cursor at system level
        if let Ok(cross) = LoadCursorW(None, IDC_CROSS) {
            if let Ok(copy) = CopyImage(HANDLE(cross.0), IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE | LR_COPYFROMRESOURCE) {
                let _ = SetSystemCursor(HCURSOR(copy.0), OCR_CROSS);
            }
            SetCursor(cross);
        }

        // Store hwnds globally for cursor tracking timer
        {
            let mut global = OVERLAY_HWNDS.lock().unwrap();
            *global = monitor_rects.iter().zip(hwnds.iter()).map(|(&(mx, my, mw, mh), h)| {
                (h.0 as isize, mx, my, mw, mh)
            }).collect();
        }

        // Start timer to track cursor position across monitors
        let _ = SetTimer(hwnds[0], CURSOR_TRACK_TIMER, 16, None);

        // Single message loop handles ALL windows on this thread
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Restore default arrow cursor
        if let Ok(arrow) = LoadCursorW(None, IDC_ARROW) {
            if let Ok(copy) = CopyImage(HANDLE(arrow.0), IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE | LR_COPYFROMRESOURCE) {
                let _ = SetSystemCursor(HCURSOR(copy.0), OCR_NORMAL);
            }
        }

        // Find which window was used for selection
        let mut result = Err("Cancelled".to_string());
        for (i, &state_ptr) in states.iter().enumerate() {
            let s = &*state_ptr;
            if s.done && !s.cancelled {
                let (mx, my, _, _) = monitor_rects[i];
                let x1 = s.start_x.min(s.cur_x) + mx;
                let y1 = s.start_y.min(s.cur_y) + my;
                let x2 = s.start_x.max(s.cur_x) + mx;
                let y2 = s.start_y.max(s.cur_y) + my;
                result = Ok((x1, y1, (x2 - x1) as u32, (y2 - y1) as u32));
                break;
            }
        }

        // Cleanup all windows and states
        let _ = KillTimer(hwnds[0], CURSOR_TRACK_TIMER);
        {
            let mut global = OVERLAY_HWNDS.lock().unwrap();
            global.clear();
        }
        for (hwnd, state_ptr) in hwnds.iter().zip(states.iter()) {
            drop(Box::from_raw(*state_ptr));
            let _ = ShowWindow(*hwnd, SW_HIDE);
            let _ = DestroyWindow(*hwnd);
        }
        let _ = UnregisterClassW(class_name, None);

        result
    }
}

#[cfg(not(target_os = "windows"))]
fn show_native_overlay() -> Result<(i32, i32, u32, u32), String> {
    Err("Native overlay only supported on Windows".to_string())
}
