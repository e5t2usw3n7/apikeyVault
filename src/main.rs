mod config;
mod core;
mod crypto;
mod error;
mod gui;
mod import_export;
mod shell;
mod storage;
mod validation;
mod cli;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    // 如果是 GUI 命令，启动桌面应用
    if let Commands::Gui { .. } = &cli.command {
        if let Err(e) = start_desktop_gui(&cli) {
            eprintln!("启动 GUI 失败: {}", e);
            std::process::exit(1);
        }
        return;
    }

    match cli::commands::execute(cli) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    }
}

/// 配置中文字体，从 Windows 系统字体目录加载微软雅黑
fn configure_chinese_font(ctx: &eframe::egui::Context) {
    use eframe::egui::{FontData, FontDefinitions, FontFamily};

    let mut fonts = FontDefinitions::default();

    // 尝试加载 Windows 系统中文字体
    let font_paths = [
        // 微软雅黑
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyhbd.ttc",
        // 微软正黑（备选）
        r"C:\Windows\Fonts\msjh.ttc",
        // 思源黑体（如果安装了）
        r"C:\Windows\Fonts\SourceHanSansSC-Regular.otf",
    ];

    let mut font_loaded = false;
    for path in &font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "chinese".to_owned(),
                FontData::from_owned(font_data),
            );
            // 将中文字体作为首选的 Proportional 和 Monospace 字体
            fonts.families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese".to_owned());
            fonts.families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "chinese".to_owned());
            font_loaded = true;
            break;
        }
    }

    if !font_loaded {
        // 如果找不到系统字体，尝试从 Windows/fonts 资源中搜索
        if let Ok(entries) = std::fs::read_dir(r"C:\Windows\Fonts") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("msyh") || name.contains("simsun") || name.contains("simhei") {
                    if let Ok(font_data) = std::fs::read(entry.path()) {
                        fonts.font_data.insert(
                            "chinese".to_owned(),
                            FontData::from_owned(font_data),
                        );
                        fonts.families
                            .entry(FontFamily::Proportional)
                            .or_default()
                            .insert(0, "chinese".to_owned());
                        fonts.families
                            .entry(FontFamily::Monospace)
                            .or_default()
                            .insert(0, "chinese".to_owned());
                        break;
                    }
                }
            }
        }
    }

    ctx.set_fonts(fonts);
}

fn create_app_icon() -> eframe::egui::IconData {
    // 创建一个 32x32 的锁形图标（简化像素画）
    let width = 32u32;
    let height = 32u32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    // 绘制简单的锁/密钥形状图标
    // 背景色: 透明
    // 主色: #6C5CE7 (紫色)
    let (r, g, b) = (108u8, 92u8, 231u8);

    // 锁体 - 中下方矩形 (x:8-23, y:14-25)
    for y in 14..26 {
        for x in 8..24 {
            let idx = ((y * width + x) * 4) as usize;
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = 255;
        }
    }

    // 锁孔 - 中间小圆
    for y in 18..22 {
        for x in 14..18 {
            let idx = ((y * width + x) * 4) as usize;
            rgba[idx] = 255;
            rgba[idx + 1] = 255;
            rgba[idx + 2] = 255;
            rgba[idx + 3] = 255;
        }
    }

    // 锁环 - 上方拱形 (x:10-21, y:4-14)
    for y in 4..15 {
        for x in 10..22 {
            // 只画边框
            if y == 4 || y == 14 || x == 10 || x == 21 {
                let idx = ((y * width + x) * 4) as usize;
                rgba[idx] = r;
                rgba[idx + 1] = g;
                rgba[idx + 2] = b;
                rgba[idx + 3] = 255;
            }
            // 清空内部
            if y > 4 && y < 14 && x > 10 && x < 21 {
                let idx = ((y * width + x) * 4) as usize;
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }

    // 高光 - 锁体左上角
    for y in 15..17 {
        for x in 9..12 {
            let idx = ((y * width + x) * 4) as usize;
            rgba[idx] = 150;
            rgba[idx + 1] = 140;
            rgba[idx + 2] = 245;
            rgba[idx + 3] = 255;
        }
    }

    eframe::egui::IconData {
        rgba,
        width,
        height,
    }
}

fn start_desktop_gui(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = config::AppConfig::load();
    if let Some(ref path) = cli.vault_path {
        config.vault_path = path.clone();
    }
    let vault_path = config.vault_path;

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("API Key Vault")
            .with_icon(create_app_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "API Key Vault",
        native_options,
        Box::new(move |cc| {
            // 配置中文字体
            configure_chinese_font(&cc.egui_ctx);
            Ok(Box::new(gui::VaultApp::new(vault_path)))
        }),
    )?;

    Ok(())
}