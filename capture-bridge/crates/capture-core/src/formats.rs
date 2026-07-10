use capture_common::{FormatPreset, ScanMode};

pub fn recommended_formats() -> Vec<FormatPreset> {
    vec![
        preset("720p25", 1280, 720, "25", "3500-5000 kbps", false),
        preset("720p29.97", 1280, 720, "29.97", "3500-5000 kbps", false),
        preset("720p30", 1280, 720, "30", "3500-5000 kbps", false),
        preset("720p50", 1280, 720, "50", "5000-7500 kbps", false),
        preset("720p59.94", 1280, 720, "59.94", "5000-7500 kbps", false),
        preset("720p60", 1280, 720, "60", "5000-7500 kbps", false),
        preset("1080p23.976", 1920, 1080, "23.976", "6000-9000 kbps", false),
        preset("1080p24", 1920, 1080, "24", "6000-9000 kbps", false),
        preset("1080p25", 1920, 1080, "25", "7000-10000 kbps", false),
        preset("1080p29.97", 1920, 1080, "29.97", "8000-12000 kbps", false),
        preset("1080p30", 1920, 1080, "30", "8000-12000 kbps", true),
        preset("1080p50", 1920, 1080, "50", "12000-18000 kbps", false),
        preset("1080p59.94", 1920, 1080, "59.94", "14000-20000 kbps", false),
        preset("1080p60", 1920, 1080, "60", "14000-20000 kbps", false),
    ]
}

pub fn recommended_formats_table() -> String {
    let mut lines = vec![
        "Campus Connect Capture Bridge recommended formats".to_string(),
        String::new(),
        "name          size        fps     scan          typical bitrate".to_string(),
    ];
    for format in recommended_formats() {
        lines.push(format!(
            "{:<13} {:>4}x{:<5} {:<7} {:<13} {}",
            format.name,
            format.width,
            format.height,
            format.frame_rate,
            "progressive",
            format.typical_bitrate_kbps
        ));
    }
    lines.push(String::new());
    lines.push("Production default: 1080p30. Use 59.94/60 only when every capture, network, and receiver path is proven stable.".to_string());
    lines.join("\n")
}

fn preset(
    name: &str,
    width: u32,
    height: u32,
    frame_rate: &str,
    typical_bitrate_kbps: &str,
    production_default: bool,
) -> FormatPreset {
    FormatPreset {
        name: name.to_string(),
        width,
        height,
        frame_rate: frame_rate.to_string(),
        scan: ScanMode::Progressive,
        typical_bitrate_kbps: typical_bitrate_kbps.to_string(),
        production_default,
    }
}
