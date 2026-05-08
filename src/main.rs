use std::env;
use std::fs;
use std::process::Command;
use sysinfo::{Disks, System};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Путь к файлу с логотипом (ASCII-art)
    let logo_path = if args.len() > 1 {
        args[1].clone()
    } else {
        "logo.txt".to_string()
    };

    // Читаем логотип из файла
    let logo = match fs::read_to_string(&logo_path) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Не удалось прочитать файл: {}", logo_path);
            eprintln!("Использую встроенный логотип");
            DEFAULT_LOGO.to_string()
        }
    };

    let sys_info = get_system_info();
    print_neofetch(&logo, &sys_info);
}

struct SysInfo {
    user: String,
    hostname: String,
    os: String,
    kernel: String,
    uptime: String,
    packages: String,
    shell: String,
    resolution: String,
    de: String,
    theme: String,
    icons: String,
    terminal: String,
    cpu: String,
    gpu: String,
    memory: String,
    disk: String,
    ip_local: String,
}

fn get_system_info() -> SysInfo {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Имя пользователя и хоста
    let user = whoami::username();
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());

    // ОС
    let os = System::name().unwrap_or_else(|| "Unknown".to_string());

    // Ядро
    let kernel = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

    // Uptime
    let uptime_secs = System::uptime();
    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let minutes = (uptime_secs % 3600) / 60;
    let uptime = format!("{}d {}h {}m", days, hours, minutes);

    // Количество пакетов
    let packages = get_package_count();

    // Shell
    let shell = env::var("SHELL")
        .unwrap_or_else(|_| "unknown".to_string())
        .split('/')
        .last()
        .unwrap_or("unknown")
        .to_string();

    // Разрешение экрана
    let resolution = get_resolution();

    // DE / WM
    let de = env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "Unknown".to_string());

    // Тема и иконки
    let theme = get_gsettings("org.gnome.desktop.interface", "gtk-theme");
    let icons = get_gsettings("org.gnome.desktop.interface", "icon-theme");

    // Терминал
    let terminal = env::var("TERM_PROGRAM")
        .or_else(|_| env::var("TERM"))
        .unwrap_or_else(|_| "unknown".to_string());

    // CPU
    let cpu_cpus = sys.cpus();
    let cpu = format!(
        "{} ({} cores)",
        cpu_cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
        cpu_cpus.len()
    );

    // GPU
    let gpu = get_gpu();

    // Память
    let total_mem = sys.total_memory() / 1024 / 1024;
    let used_mem = sys.used_memory() / 1024 / 1024;
    let memory = format!("{}MiB / {}MiB", used_mem, total_mem);

    // Диск
    let disk = get_disk_usage();

    // Локальный IP
    let ip_local = local_ip_address::local_ip()
        .map(|ip| ip.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    SysInfo {
        user,
        hostname,
        os,
        kernel,
        uptime,
        packages,
        shell,
        resolution,
        de,
        theme,
        icons,
        terminal,
        cpu,
        gpu,
        memory,
        disk,
        ip_local,
    }
}

fn get_package_count() -> String {
    let managers = [
        ("dpkg", "dpkg-query -f '.\n' -W 2>/dev/null | wc -l"),
        ("rpm", "rpm -qa 2>/dev/null | wc -l"),
        ("pacman", "pacman -Q 2>/dev/null | wc -l"),
        ("flatpak", "flatpak list 2>/dev/null | wc -l"),
        ("snap", "snap list 2>/dev/null | tail -n +2 | wc -l"),
        ("nix", "nix-store -q --installed 2>/dev/null | wc -l"),
    ];

    for (name, cmd) in managers {
        if let Ok(output) = Command::new("sh").arg("-c").arg(cmd).output() {
            if output.status.success() {
                let count = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse::<u32>()
                    .unwrap_or(0);
                if count > 0 {
                    return format!("{} ({})", count, name);
                }
            }
        }
    }
    "unknown".to_string()
}

fn get_resolution() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("xrandr --current 2>/dev/null | grep '*' | awk '{print $1}' | head -1")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let res = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !res.is_empty() {
                res
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

fn get_gsettings(schema: &str, key: &str) -> String {
    let output = Command::new("gsettings")
        .arg("get")
        .arg(schema)
        .arg(key)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let val = String::from_utf8_lossy(&o.stdout)
                .trim()
                .trim_matches('\'')
                .to_string();
            if !val.is_empty() && val != "''" {
                val
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

fn get_gpu() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("lspci 2>/dev/null | grep -E 'VGA|3D' | head -1 | sed 's/.*: //'")
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let gpu = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !gpu.is_empty() {
                gpu
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}

fn get_disk_usage() -> String {
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        if disk.mount_point().to_str() == Some("/") {
            let total = disk.total_space() / 1024 / 1024 / 1024;
            let available = disk.available_space() / 1024 / 1024 / 1024;
            let used = total - available;
            return format!("{}GiB / {}GiB", used, total);
        }
    }
    "unknown".to_string()
}
fn print_neofetch(logo: &str, info: &SysInfo) {
    let logo_lines: Vec<&str> = logo.lines().collect();
    let info_lines: Vec<String> = vec![
        format!("{}@{}", info.user, info.hostname),
        format!("OS: {}", info.os),
        format!("Kernel: {}", info.kernel),
        format!("Uptime: {}", info.uptime),
        format!("Packages: {}", info.packages),
        format!("Shell: {}", info.shell),
        format!("Resolution: {}", info.resolution),
        format!("DE: {}", info.de),
        format!("Theme: {}", info.theme),
        format!("Icons: {}", info.icons),
        format!("Terminal: {}", info.terminal),
        format!("CPU: {}", info.cpu),
        format!("GPU: {}", info.gpu),
        format!("Memory: {}", info.memory),
        format!("Disk: {}", info.disk),
        format!("Local IP: {}", info.ip_local),
    ];

    let max_logo_width = logo_lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    let max_lines = std::cmp::max(logo_lines.len(), info_lines.len());

    for i in 0..max_lines {
        let logo_part = if i < logo_lines.len() {
            logo_lines[i]
        } else {
            ""
        };

        let info_part = if i < info_lines.len() {
            &info_lines[i]
        } else {
            ""
        };

        // Выравнивание и цветной вывод
        print!(
            "\x1b[36m{: <width$}\x1b[0m",
            logo_part,
            width = max_logo_width
        );
        println!(" \x1b[33m▶\x1b[0m {}", info_part);
    }
}
const DEFAULT_LOGO: &str = r#"
       .,:;:;:;;:;:;:;:;;:;:;,.
   .,;==========================;;,.
  ;===============================;
 ;=================================;
 ;=================================;
 ;,===============================;;
  '===============================;'
   ';,==========================;,'
      '';,................,;''  
            ':::::::::::'

               🦀 RUST
"#;
