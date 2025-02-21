## windows-theme


```rs
use windows_theme::{ set_dark_mode, set_lockscreen, set_wallpaper };
use windows::core::Result;
fn main() -> Result<()> {
    let wallpaper_path = r"E:\TEMP\Pictures\wall\1.jpg";

    set_wallpaper(wallpaper_path)?;

    set_lockscreen(wallpaper_path)?;

    // 切换暗色模式 (true = 暗色模式, false = 亮色模式)
    set_dark_mode(false)?;

    Ok(())
}
```