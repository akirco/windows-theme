use windows::core::Result;
#[allow(unused_imports)]
use windows_theme::{get_wallpaper, set_dark_mode, set_lockscreen, set_wallpaper};
fn main() -> Result<()> {
    // let wallpaper_path = r"E:\resources\Pictures\2020-04-18_05-48-48_UTC.jpg";

    // set_wallpaper(wallpaper_path)?;

    // set_lockscreen(wallpaper_path)?;

    // // 切换暗色模式 (true = 暗色模式, false = 亮色模式)
    // set_dark_mode(true)?;

    // let wallreg = get_wallpaper_from_registry();
    // if let Ok(w) = wallreg {
    //     println!("ref:{}", w)
    // }
    let wall = get_wallpaper();
    if let Ok(w) = wall {
        println!("winapi:{}", w)
    }

    Ok(())
}
