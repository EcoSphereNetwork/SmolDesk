// build.rs - Build script for SmolDesk Tauri application

use std::env;
use std::process::Command;

fn main() {
    // Run Tauri build
    tauri_build::build();
    
    // Print build information
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-env-changed=SMOLDESK_VERSION");
    
    // Check system dependencies
    check_system_dependencies();
    
    // Set conditional compilation flags
    set_compilation_flags();
    
    // Generate version information
    generate_version_info();
}

fn check_system_dependencies() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    if target_os == "linux" {
        // Check for essential Linux dependencies
        check_command_exists("ffmpeg", "FFmpeg is required for video encoding");
        
        // Check display server support
        if check_command_exists_silent("xrandr") {
            println!("cargo:rustc-cfg=feature=\"x11\"");
            println!("cargo:warning=X11 support enabled");
        }
        
        if check_command_exists_silent("wlr-randr") || check_command_exists_silent("swaymsg") {
            println!("cargo:rustc-cfg=feature=\"wayland\"");
            println!("cargo:warning=Wayland support enabled");
        }
        
        // Check input tools
        if check_command_exists_silent("xdotool") {
            println!("cargo:rustc-cfg=feature=\"x11_input\"");
        } else {
            println!("cargo:warning=xdotool not found - X11 input forwarding will be limited");
        }
        
        if check_command_exists_silent("ydotool") {
            println!("cargo:rustc-cfg=feature=\"wayland_input\"");
        } else {
            println!("cargo:warning=ydotool not found - Wayland input forwarding will be limited");
        }
        
        // Check clipboard tools
        if check_command_exists_silent("xclip") || check_command_exists_silent("xsel") {
            println!("cargo:rustc-cfg=feature=\"x11_clipboard\"");
        }
        
        if check_command_exists_silent("wl-copy") && check_command_exists_silent("wl-paste") {
            println!("cargo:rustc-cfg=feature=\"wayland_clipboard\"");
        }
        
        // Check hardware acceleration support
        check_hardware_acceleration();
    }
}

fn check_command_exists(command: &str, error_message: &str) {
    if !check_command_exists_silent(command) {
        println!("cargo:warning={}", error_message);
        eprintln!("Warning: {}", error_message);
    }
}

fn check_command_exists_silent(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn check_hardware_acceleration() {
    // Check for VAAPI support
    if std::path::Path::new("/dev/dri/renderD128").exists() {
        if check_ffmpeg_encoder("h264_vaapi") {
            println!("cargo:rustc-cfg=feature=\"vaapi\"");
            println!("cargo:warning=VAAPI hardware acceleration available");
        }
    }
    
    // Check for NVENC support
    if check_ffmpeg_encoder("h264_nvenc") {
        println!("cargo:rustc-cfg=feature=\"nvenc\"");
        println!("cargo:warning=NVENC hardware acceleration available");
    }
    
    // Check for QuickSync support
    if check_ffmpeg_encoder("h264_qsv") {
        println!("cargo:rustc-cfg=feature=\"quicksync\"");
        println!("cargo:warning=QuickSync hardware acceleration available");
    }
}

fn check_ffmpeg_encoder(encoder: &str) -> bool {
    Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .map(|output| {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains(encoder)
            } else {
                false
            }
        })
        .unwrap_or(false)
}

fn set_compilation_flags() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    
    // Enable platform-specific features
    match target_os.as_str() {
        "linux" => {
            println!("cargo:rustc-cfg=linux_platform");
            
            // Check if we're in a container or have limited capabilities
            if env::var("CONTAINER").is_ok() || env::var("FLATPAK_ID").is_ok() {
                println!("cargo:rustc-cfg=containerized");
                println!("cargo:warning=Building in containerized environment - some features may be limited");
            }
        },
        "windows" => {
            println!("cargo:rustc-cfg=windows_platform");
            println!("cargo:warning=Windows platform detected but SmolDesk is primarily designed for Linux");
        },
        "macos" => {
            println!("cargo:rustc-cfg=macos_platform");
            println!("cargo:warning=macOS platform detected but SmolDesk is primarily designed for Linux");
        },
        _ => {
            println!("cargo:warning=Unsupported platform: {}", target_os);
        }
    }
    
    // Architecture-specific optimizations
    match target_arch.as_str() {
        "x86_64" => {
            println!("cargo:rustc-cfg=arch_x86_64");
        },
        "aarch64" => {
            println!("cargo:rustc-cfg=arch_aarch64");
            println!("cargo:warning=ARM64 architecture - some optimizations may differ");
        },
        _ => {
            println!("cargo:warning=Untested architecture: {}", target_arch);
        }
    }
    
    // Development vs release builds
    let profile = env::var("PROFILE").unwrap_or_default();
    if profile == "debug" {
        println!("cargo:rustc-cfg=debug_build");
        println!("cargo:warning=Debug build - performance may be reduced");
    } else {
        println!("cargo:rustc-cfg=release_build");
    }
}

fn generate_version_info() {
    // Get version from Cargo.toml
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=SMOLDESK_VERSION={}", version);
    
    // Get git information if available
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=SMOLDESK_GIT_HASH={}", git_hash);
        }
    }
    
    if let Ok(output) = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output() {
        if output.status.success() {
            let git_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=SMOLDESK_GIT_BRANCH={}", git_branch);
        }
    }
    
    // Build timestamp
    let build_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    println!("cargo:rustc-env=SMOLDESK_BUILD_TIMESTAMP={}", build_timestamp);
    
    // Build host information
    if let Ok(hostname) = env::var("HOSTNAME") {
        println!("cargo:rustc-env=SMOLDESK_BUILD_HOST={}", hostname);
    }
    
    // Rust version
    if let Ok(output) = Command::new("rustc").arg("--version").output() {
        if output.status.success() {
            let rust_version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=SMOLDESK_RUST_VERSION={}", rust_version);
        }
    }
    
    // Feature flags summary
    let mut features = Vec::new();
    
    if check_command_exists_silent("xrandr") {
        features.push("x11");
    }
    if check_command_exists_silent("wlr-randr") || check_command_exists_silent("swaymsg") {
        features.push("wayland");
    }
    if std::path::Path::new("/dev/dri/renderD128").exists() {
        features.push("vaapi");
    }
    if check_ffmpeg_encoder("h264_nvenc") {
        features.push("nvenc");
    }
    if check_ffmpeg_encoder("h264_qsv") {
        features.push("quicksync");
    }
    
    let features_str = features.join(",");
    println!("cargo:rustc-env=SMOLDESK_FEATURES={}", features_str);
    println!("cargo:warning=Enabled features: {}", features_str);
}

// Helper function to check if we're in a cross-compilation environment
fn is_cross_compiling() -> bool {
    let host = env::var("HOST").unwrap_or_default();
    let target = env::var("TARGET").unwrap_or_default();
    !host.is_empty() && !target.is_empty() && host != target
}

// Custom build configuration for different environments
fn configure_build_environment() {
    if is_cross_compiling() {
        println!("cargo:warning=Cross-compilation detected - some runtime checks will be skipped");
        println!("cargo:rustc-cfg=cross_compiling");
    }
    
    // CI/CD environment detection
    if env::var("CI").is_ok() {
        println!("cargo:rustc-cfg=ci_build");
        println!("cargo:warning=CI environment detected");
    }
    
    if env::var("GITHUB_ACTIONS").is_ok() {
        println!("cargo:rustc-cfg=github_actions");
    }
    
    if env::var("GITLAB_CI").is_ok() {
        println!("cargo:rustc-cfg=gitlab_ci");
    }
}
