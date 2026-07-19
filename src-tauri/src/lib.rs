use windows::core::Interface;
use windows::core::PCWSTR;
use windows::core::PWSTR;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::Diagnostics::ToolHelp::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::Shell::SHGetFileInfoW;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::Threading::*;
use tauri::menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;

// Function to find PID of a process by name
fn get_process_pids_by_name(target_name: &str) -> Vec<u32> {
    let mut pids = Vec::new();
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if let Ok(snapshot) = snapshot {
            let mut entry = PROCESSENTRY32W::default();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    // Convert WCHAR array to String
                    let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                    let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                    if name.eq_ignore_ascii_case(target_name) {
                        pids.push(entry.th32ProcessID);
                    }
                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
        }
    }
    pids
}

// Function to check if the process is running
#[tauri::command]
fn is_process_running(process_name: String) -> bool {
    !get_process_pids_by_name(&process_name).is_empty()
}

// Helper to get mute status of PIDs
fn get_mute_status_for_pids(pids: &[u32]) -> Result<Option<bool>, windows::core::Error> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
        let session_enumerator = session_manager.GetSessionEnumerator()?;
        let count = session_enumerator.GetCount()?;

        for i in 0..count {
            if let Ok(session_control) = session_enumerator.GetSession(i) {
                if let Ok(session_control2) = session_control.cast::<IAudioSessionControl2>() {
                    if let Ok(pid) = session_control2.GetProcessId() {
                        if pids.contains(&pid) {
                            if let Ok(simple_volume) = session_control.cast::<ISimpleAudioVolume>() {
                                if let Ok(muted) = simple_volume.GetMute() {
                                    return Ok(Some(muted.into()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(None)
}

// Helper to set mute status for PIDs
fn set_mute_for_pids(pids: &[u32], mute: bool) -> Result<(), windows::core::Error> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)?;
        let session_enumerator = session_manager.GetSessionEnumerator()?;
        let count = session_enumerator.GetCount()?;

        for i in 0..count {
            if let Ok(session_control) = session_enumerator.GetSession(i) {
                if let Ok(session_control2) = session_control.cast::<IAudioSessionControl2>() {
                    if let Ok(pid) = session_control2.GetProcessId() {
                        if pids.contains(&pid) {
                            if let Ok(simple_volume) = session_control.cast::<ISimpleAudioVolume>() {
                                simple_volume.SetMute(mute, std::ptr::null())?;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn get_mute_status(process_name: String) -> Result<bool, String> {
    let pids = get_process_pids_by_name(&process_name);
    if pids.is_empty() {
        return Ok(false);
    }
    let status = get_mute_status_for_pids(&pids).map_err(|e| e.to_string())?;
    Ok(status.unwrap_or(false))
}

#[tauri::command]
fn toggle_mute(process_name: String) -> Result<bool, String> {
    let pids = get_process_pids_by_name(&process_name);
    if pids.is_empty() {
        return Err(format!("Process '{}' is not running", process_name));
    }
    let current_mute = get_mute_status_for_pids(&pids)
        .map_err(|e| e.to_string())?
        .unwrap_or(false);
    let next_mute = !current_mute;
    set_mute_for_pids(&pids, next_mute).map_err(|e| e.to_string())?;
    Ok(next_mute)
}

// Function to get list of unique process names that currently have audio sessions
#[tauri::command]
fn get_audio_processes() -> Result<Vec<String>, String> {
    let mut processes = std::collections::HashSet::new();
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|e| e.to_string())?;
        let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| e.to_string())?;
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)
            .map_err(|e| e.to_string())?;
        let session_enumerator = session_manager.GetSessionEnumerator()
            .map_err(|e| e.to_string())?;
        let count = session_enumerator.GetCount()
            .map_err(|e| e.to_string())?;

        // Get running process map (PID -> ExeName) using ToolHelp
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| e.to_string())?;
        let mut process_map = std::collections::HashMap::new();
        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                process_map.insert(entry.th32ProcessID, name);
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);

        let current_exe_name = std::env::current_exe()
            .ok()
            .and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "earmuff.exe".to_string());

        for i in 0..count {
            if let Ok(session_control) = session_enumerator.GetSession(i) {
                if let Ok(session_control2) = session_control.cast::<IAudioSessionControl2>() {
                    if let Ok(pid) = session_control2.GetProcessId() {
                        if pid != 0 {
                            if let Some(name) = process_map.get(&pid) {
                                if !name.is_empty() 
                                    && !name.eq_ignore_ascii_case(&current_exe_name) 
                                    && !name.eq_ignore_ascii_case("Idle") 
                                {
                                    processes.insert(name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut list: Vec<String> = processes.into_iter().collect();
    list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    Ok(list)
}

// Get full executable path of a process from PID
fn get_process_path(pid: u32) -> Result<String, windows::core::Error> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)?;
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), PWSTR(buffer.as_mut_ptr()), &mut size)?;
        let _ = CloseHandle(handle);
        Ok(String::from_utf16_lossy(&buffer[..size as usize]))
    }
}

// Convert HICON to BMP file bytes in memory
unsafe fn hicon_to_bmp_bytes(hicon: HICON) -> Option<Vec<u8>> {
    let mut icon_info = ICONINFO::default();
    if GetIconInfo(hicon, &mut icon_info).is_err() {
        return None;
    }
    
    let hbm_color = icon_info.hbmColor;
    let hbm_mask = icon_info.hbmMask;
    
    let hdc_screen = CreateCompatibleDC(None);
    let mut bmp = BITMAP::default();
    
    let get_object_res = GetObjectW(
        hbm_color.into(),
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bmp as *mut _ as *mut _),
    );
    
    if get_object_res == 0 {
        let _ = DeleteObject(hbm_color.into());
        let _ = DeleteObject(hbm_mask.into());
        let _ = DeleteDC(hdc_screen);
        return None;
    }
    
    let width = bmp.bmWidth;
    let height = bmp.bmHeight;
    
    let mut bmi = BITMAPINFO::default();
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height; // negative height for top-down bitmap
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = 0; // BI_RGB = 0

    
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    
    let res = GetDIBits(
        hdc_screen,
        hbm_color,
        0,
        height as u32,
        Some(pixels.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
    );
    
    let _ = DeleteObject(hbm_color.into());
    let _ = DeleteObject(hbm_mask.into());
    let _ = DeleteDC(hdc_screen);
    
    if res == 0 {
        return None;
    }
    
    // BMP encoding (54 bytes header + raw pixels)
    let file_header_size = 14;
    let info_header_size = 40;
    let pixel_data_offset = file_header_size + info_header_size;
    let file_size = pixel_data_offset + pixels.len();
    
    let mut bmp_file = Vec::with_capacity(file_size);
    
    // File Type (BM)
    bmp_file.extend_from_slice(b"BM");
    // File Size
    bmp_file.extend_from_slice(&(file_size as u32).to_le_bytes());
    // Reserved
    bmp_file.extend_from_slice(&[0u8; 4]);
    // Offset
    bmp_file.extend_from_slice(&(pixel_data_offset as u32).to_le_bytes());
    
    // Info Header
    bmp_file.extend_from_slice(&(info_header_size as u32).to_le_bytes());
    bmp_file.extend_from_slice(&width.to_le_bytes());
    bmp_file.extend_from_slice(&(-height).to_le_bytes());
    bmp_file.extend_from_slice(&1u16.to_le_bytes());
    bmp_file.extend_from_slice(&32u16.to_le_bytes());
    bmp_file.extend_from_slice(&0u32.to_le_bytes());
    bmp_file.extend_from_slice(&(pixels.len() as u32).to_le_bytes());
    bmp_file.extend_from_slice(&0i32.to_le_bytes());
    bmp_file.extend_from_slice(&0i32.to_le_bytes());
    bmp_file.extend_from_slice(&0u32.to_le_bytes());
    bmp_file.extend_from_slice(&0u32.to_le_bytes());
    
    // Pixels
    bmp_file.extend_from_slice(&pixels);
    
    Some(bmp_file)
}

// Simple Base64 Encoder
fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut i = 0;
    while i < data.len() {
        let chunk = &data[i..std::cmp::min(i + 3, data.len())];
        let mut b = 0u32;
        for (idx, &val) in chunk.iter().enumerate() {
            b |= (val as u32) << (16 - idx * 8);
        }
        
        let c0 = CHARSET[((b >> 18) & 0x3F) as usize];
        let c1 = CHARSET[((b >> 12) & 0x3F) as usize];
        let c2 = if chunk.len() > 1 { CHARSET[((b >> 6) & 0x3F) as usize] } else { b'=' };
        let c3 = if chunk.len() > 2 { CHARSET[(b & 0x3F) as usize] } else { b'=' };
        
        result.push(c0 as char);
        result.push(c1 as char);
        result.push(c2 as char);
        result.push(c3 as char);
        
        i += 3;
    }
    result
}

// Tauri command to extract a process icon and return as a Base64 data URL
#[tauri::command]
fn get_process_icon(process_name: String) -> Result<String, String> {
    let pids = get_process_pids_by_name(&process_name);
    if pids.is_empty() {
        return Err(format!("Process '{}' not running", process_name));
    }
    
    let path = get_process_path(pids[0]).map_err(|e| e.to_string())?;
    
    unsafe {
        let path_u16: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let mut shfi = SHFILEINFOW::default();
        let hr = SHGetFileInfoW(
            PCWSTR(path_u16.as_ptr()),
            Default::default(),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON
        );
        
        if hr != 0 && !shfi.hIcon.is_invalid() {
            if let Some(bmp_bytes) = hicon_to_bmp_bytes(shfi.hIcon) {
                let _ = DestroyIcon(shfi.hIcon);
                let b64 = base64_encode(&bmp_bytes);
                return Ok(format!("data:image/bmp;base64,{}", b64));
            }
            let _ = DestroyIcon(shfi.hIcon);
        }
    }
    Err("Failed to extract icon".to_string())
}

// Tauri command to exit the application from frontend
#[tauri::command]
fn exit_app(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

// Tauri command to unmute all active audio sessions on the system
#[tauri::command]
fn unmute_all_processes() -> Result<(), String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(|e| e.to_string())?;
        let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)
            .map_err(|e| e.to_string())?;
        let session_manager: IAudioSessionManager2 = device.Activate(CLSCTX_ALL, None)
            .map_err(|e| e.to_string())?;
        let session_enumerator = session_manager.GetSessionEnumerator()
            .map_err(|e| e.to_string())?;
        let count = session_enumerator.GetCount()
            .map_err(|e| e.to_string())?;

        for i in 0..count {
            if let Ok(session_control) = session_enumerator.GetSession(i) {
                if let Ok(simple_volume) = session_control.cast::<ISimpleAudioVolume>() {
                    let _ = simple_volume.SetMute(false, std::ptr::null());
                }
            }
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Build the system tray icon with "Unmute All" (전체 음소거 해제), Separator, and "Quit" (종료)
            let unmute_item = MenuItemBuilder::new("전체 음소거 해제")
                .id("unmute_all")
                .build(app)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::new("종료")
                .id("quit")
                .build(app)?;
                
            let menu = MenuBuilder::new(app)
                .items(&[&unmute_item, &separator, &quit_item])
                .build()?;
                
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id().as_ref() == "quit" {
                        app.exit(0);
                    } else if event.id().as_ref() == "unmute_all" {
                        if let Err(e) = unmute_all_processes() {
                            eprintln!("Failed to unmute all sessions: {}", e);
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_process_running,
            get_mute_status,
            toggle_mute,
            get_audio_processes,
            get_process_icon,
            exit_app,
            unmute_all_processes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
