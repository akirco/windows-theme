use std::{ ffi::CString, path::Path };
use windows::{
    Storage::StorageFile,
    System::UserProfile::LockScreen,
    Win32::{
        Foundation::*,
        System::Registry::*,
        UI::WindowsAndMessaging::{
            self,
            HWND_BROADCAST,
            SMTO_ABORTIFHUNG,
            SYSTEM_PARAMETERS_INFO_ACTION,
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
            SendMessageTimeoutW,
            WM_SETTINGCHANGE,
        },
    },
    core::*,
};
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

    let c_path = CString::new(path_str).map_err(|_| Error::new(E_FAIL, "Failed to convert path"))?;

    unsafe {
        WindowsAndMessaging::SystemParametersInfoA(
            SYSTEM_PARAMETERS_INFO_ACTION(0x0014), // SPI_SETDESKWALLPAPER
            0,
            Some(c_path.as_ptr() as *mut std::ffi::c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0x01 | 0x02)
        )
            .ok()
            .ok_or_else(|| Error::new(E_FAIL, "Failed to set wallpaper"))?;
    }

    Ok(())
}

async fn set_lockscreen_winrt<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    // Convert the path to a StorageFile
    let image_file = StorageFile::GetFileFromPathAsync(
        &HSTRING::from(path.to_str().ok_or_else(|| Error::new(E_FAIL, "Invalid path"))?)
    )?.await?;

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
            &mut hkey
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
            Some(&value.to_ne_bytes())
        );

        if result != ERROR_SUCCESS {
            return Err(Error::new(E_FAIL, "Failed to set AppsUseLightTheme"));
        }

        let result = RegSetValueExW(
            hkey,
            w!("SystemUsesLightTheme"),
            Some(0),
            REG_DWORD,
            Some(&value.to_ne_bytes())
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
            None
        );
    }
    Ok(())
}
