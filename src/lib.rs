use std::{ffi::CString, path::Path};
use windows::{
    Storage::StorageFile,
    System::UserProfile::LockScreen,
    Win32::{
        Foundation::*,
        System::Registry::*,
        UI::WindowsAndMessaging::{
            self, HWND_BROADCAST, SMTO_ABORTIFHUNG, SYSTEM_PARAMETERS_INFO_ACTION,
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SendMessageTimeoutW, WM_SETTINGCHANGE,
        },
    },
    core::*,
};

/**
 * Get the current wallpaper path.
 * return: Result<String, Error> - The path to the current wallpaper
 */
pub fn get_wallpaper() -> Result<String> {
    const MAX_PATH: usize = 260;
    let mut path_buf = vec![0u16; MAX_PATH + 1]; // +1 for null terminator

    unsafe {
        let result = WindowsAndMessaging::SystemParametersInfoW(
            SYSTEM_PARAMETERS_INFO_ACTION(0x0073), // SPI_GETDESKWALLPAPER
            MAX_PATH as u32,
            Some(path_buf.as_mut_ptr() as *mut std::ffi::c_void),
            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );

        if result.is_ok() {
            // Find the null terminator and convert to String
            let len = path_buf.iter().position(|&c| c == 0).unwrap_or(MAX_PATH);
            let path_str = String::from_utf16(&path_buf[..len])
                .map_err(|_| Error::new(E_FAIL, "Invalid UTF-16 path"))?;
            if path_str.contains("TranscodedWallpaper")
                && Path::new(&path_str).extension().is_none()
            {
                let cache_path = std::env::var("APPDATA")
                    .map(std::path::PathBuf::from)
                    .map_err(|_| "Failed get APPDATA PATH")
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs();
                let temp_path = format!("{}\\{}.jpg", cache_path, time);
                let result = std::fs::copy(path_str.clone(), temp_path.clone());
                if let Ok(_r) = result {
                    return Ok(temp_path);
                } else {
                    return Err(Error::new(E_FAIL, "Failed to get wallpaper path"));
                }
            }
            Ok(path_str)
        } else {
            Err(Error::new(E_FAIL, "Failed to get wallpaper path"))
        }
    }
}

/**
 * Set the wallpaper.
 * param path: Path to the image file.
 * return: Result<(), Error>
 */
pub fn set_wallpaper<P: AsRef<Path>>(path: P) -> Result<()> {
    let path_str = path
        .as_ref()
        .to_str()
        .ok_or_else(|| Error::new(E_FAIL, "Invalid path"))?;

    let c_path =
        CString::new(path_str).map_err(|_| Error::new(E_FAIL, "Failed to convert path"))?;

    unsafe {
        WindowsAndMessaging::SystemParametersInfoA(
            SYSTEM_PARAMETERS_INFO_ACTION(0x0014), // SPI_SETDESKWALLPAPER
            0,
            Some(c_path.as_ptr() as *mut std::ffi::c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0x01 | 0x02),
        )
        .ok()
        .ok_or_else(|| Error::new(E_FAIL, "Failed to set wallpaper"))?;
    }

    Ok(())
}

async fn set_lockscreen_winrt<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    // Convert the path to a StorageFile
    let image_file = StorageFile::GetFileFromPathAsync(&HSTRING::from(
        path.to_str()
            .ok_or_else(|| Error::new(E_FAIL, "Invalid path"))?,
    ))?
    .await?;

    // Set the lock screen image
    LockScreen::SetImageFileAsync(&image_file)?.await?;

    Ok(())
}
/**
 * Set the lock screen image.
 * param path: Path to the image file.
 * return: Result<(), Error>
 */
pub fn set_lockscreen<P: AsRef<Path>>(path: P) -> Result<()> {
    use futures::executor::block_on;

    block_on(set_lockscreen_winrt(path))
}

/**
 * Set the system theme to dark or light mode.
 * param enable: True to enable dark mode, false to enable light mode.
 * return: Result<(), Error>
 */
pub fn set_dark_mode(enable: bool) -> Result<()> {
    let value = if enable { 0u32 } else { 1u32 }; // 0 = dark, 1 = light

    unsafe {
        let mut hkey = HKEY::default();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            Some(0),
            KEY_WRITE | KEY_READ,
            &mut hkey,
        );

        if result != ERROR_SUCCESS {
            return Err(Error::new(E_FAIL, "Failed to open registry key"));
        }

        struct CloseKey(HKEY);
        impl Drop for CloseKey {
            fn drop(&mut self) {
                unsafe {
                    _ = RegCloseKey(self.0);
                }
            }
        }
        let _close_key = CloseKey(hkey);

        let result = RegSetValueExW(
            hkey,
            w!("AppsUseLightTheme"),
            Some(0),
            REG_DWORD,
            Some(&value.to_ne_bytes()),
        );

        if result != ERROR_SUCCESS {
            return Err(Error::new(E_FAIL, "Failed to set AppsUseLightTheme"));
        }

        let result = RegSetValueExW(
            hkey,
            w!("SystemUsesLightTheme"),
            Some(0),
            REG_DWORD,
            Some(&value.to_ne_bytes()),
        );

        if result != ERROR_SUCCESS {
            return Err(Error::new(E_FAIL, "Failed to set SystemUsesLightTheme"));
        }

        // Broadcast settings change notification with corrected parameters
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(w!("ImmersiveColorSet").as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            1000,
            None,
        );
    }
    Ok(())
}
