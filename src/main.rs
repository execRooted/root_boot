use std::process::Command;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use std::env;

#[derive(Debug)]
struct BootDevice {
    path: String,
    model: String,
    size: String,
}

impl std::fmt::Display for BootDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} ({})", self.model, self.size, self.path)
    }
}

#[cfg(target_os = "linux")]
fn get_bootable_devices() -> Vec<BootDevice> {
    
    let output = Command::new("lsblk")
        .arg("-d")
        .arg("-o")
        .arg("NAME,MODEL,SIZE")
        .output()
        .expect("Failed to execute lsblk");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines().skip(1) { 
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[0];
            let model = parts[1..parts.len()-1].join(" ");
            let size = parts.last().unwrap();

            
            let part_output = Command::new("lsblk")
                .arg(&format!("/dev/{}", name))
                .arg("-o")
                .arg("TYPE")
                .output()
                .expect("Failed to check partitions");

            let part_stdout = String::from_utf8_lossy(&part_output.stdout);
            let has_boot = part_stdout.lines().any(|l| l.contains("part"));

            if has_boot {
                devices.push(BootDevice {
                    path: format!("/dev/{}", name),
                    model: if model.is_empty() { "Unknown".to_string() } else { model },
                    size: size.to_string(),
                });
            }
        }
    }

    devices
}

#[cfg(target_os = "windows")]
fn get_bootable_devices() -> Vec<BootDevice> {
    
    let output = Command::new("wmic")
        .arg("diskdrive")
        .arg("get")
        .arg("DeviceID,Model,Size")
        .output()
        .expect("Failed to execute wmic");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines().skip(1) { 
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let device_id = parts[0];
            let size = parts.last().unwrap();
            let model = parts[1..parts.len()-1].join(" ");

            
            
            devices.push(BootDevice {
                path: device_id.to_string(),
                model: if model.is_empty() { "Unknown".to_string() } else { model },
                size: size.to_string(),
            });
        }
    }

    devices
}

#[cfg(target_os = "linux")]
fn set_boot_device(device: &BootDevice) {
    
    println!("Attempting to set boot device to {}...", device.path);

    
    let list_output = Command::new("efibootmgr")
        .output();

    if let Ok(output) = list_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("HD(") && line.contains(&device.path.replace("/dev/", "")) {
                if let Some(boot_num) = line.split('*').next().and_then(|s| s.split("Boot").nth(1)) {
                    
                    let _ = Command::new("efibootmgr")
                        .arg("-o")
                        .arg(boot_num)
                        .status();
                    println!("Boot device set successfully.");
                    return;
                }
            }
        }
    }

    println!("Could not automatically set boot device. Manual configuration may be required.");
}

#[cfg(target_os = "windows")]
fn set_boot_device(device: &BootDevice) {
    
    println!("Attempting to set boot device to {}...", device.path);

    
    
    let drive_letter = device.path.chars().last().unwrap_or('C');

    
    let result = Command::new("bcdedit")
        .arg("/set")
        .arg("{bootmgr}")
        .arg("device")
        .arg(format!("partition={}:", drive_letter))
        .status();

    match result {
        Ok(status) if status.success() => {
            println!("Boot device set successfully.");
        }
        _ => {
            println!("Could not set boot device automatically. You may need to:");
            println!("1. Run this program as Administrator");
            println!("2. Manually change boot order in BIOS/UEFI settings");
        }
    }
}

fn reboot_system() {
    #[cfg(target_os = "linux")]
    {
        Command::new("sudo")
            .arg("reboot")
            .spawn()
            .expect("Failed to reboot");
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("shutdown")
            .arg("/r")
            .arg("/t")
            .arg("0")
            .spawn()
            .expect("Failed to reboot");
    }
}

fn check_privileges() -> bool {
    #[cfg(target_os = "linux")]
    {
        
        std::process::id() == 1 || unsafe { libc::geteuid() } == 0
    }

    #[cfg(target_os = "windows")]
    {
        
        use std::ptr;
        use winapi::um::processthreadsapi::GetCurrentProcess;
        use winapi::um::processthreadsapi::OpenProcessToken;
        use winapi::um::securitybaseapi::GetTokenInformation;
        use winapi::um::winnt::TokenElevation;
        use winapi::um::winnt::TOKEN_ELEVATION;

        unsafe {
            let mut token: winapi::um::winnt::HANDLE = ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), winapi::um::winnt::TOKEN_QUERY, &mut token) == 0 {
                return false;
            }

            let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
            let mut size: u32 = 0;
            if GetTokenInformation(token, TokenElevation, &mut elevation as *mut _ as *mut _, std::mem::size_of::<TOKEN_ELEVATION>() as u32, &mut size) == 0 {
                return false;
            }

            elevation.TokenIsElevated != 0
        }
    }
}

fn elevate_privileges() {
    #[cfg(target_os = "linux")]
    {
        
        let current_exe = std::env::current_exe().expect("Failed to get current executable");
        let args: Vec<String> = std::env::args().skip(1).collect();
        
        let status = Command::new("sudo")
            .arg("-S") 
            .arg(current_exe)
            .args(&args)
            .status()
            .expect("Failed to run sudo");

        std::process::exit(status.code().unwrap_or(1));
    }

    #[cfg(target_os = "windows")]
    {
        
        
        println!("{}", "Please run this program as Administrator.".red());
        println!("Right-click the executable and select 'Run as administrator'");
        std::process::exit(1);
    }
}

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "-v" | "--version" => {
                println!("root_boot v{}", env!("CARGO_PKG_VERSION"));
                return;
            }
            _ => {
                println!(
                    "{}",
                    format!(
                        "{} {} {} {} {}",
                        "Usage:".bright_white().bold(),
                        "root_boot".cyan().bold(),
                        "[-v|--version]".yellow().bold(),
                        " -> ".blue().bold(),
                        "shows version".white()
                    )
                );

                println!(
                    "{}",
                    format!(
                        "       {} {} {}",
                        "root_boot".cyan().bold(),
                        " -> ".blue().bold(),
                        "runs the program".white()
                    )
                );

                return;
            }
        }
    }

    if !check_privileges() {
        elevate_privileges();
    }

    println!("{}", "Select a device to restart and boot into".bold().bright_blue());
    println!("");

    let devices = get_bootable_devices();

    if devices.is_empty() {
        println!("{}", "No bootable devices found.".red());
        return;
    }

    let mut options: Vec<String> = devices.iter().enumerate()
        .map(|(i, dev)| format!("{}) {}", i + 1, dev))
        .collect();

    options.push("0) Exit".to_string());

    loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a boot device")
            .default(0)
            .items(&options)
            .interact()
            .unwrap();

        if selection == options.len() - 1 {
            println!("{}", "Exiting...".yellow());
            break;
        } else {
            let device = &devices[selection];
            println!("Selected device: {}", device.to_string().green());

            set_boot_device(device);
            println!("{}", "Rebooting in 5 seconds...".red());

            std::thread::sleep(std::time::Duration::from_secs(5));
            reboot_system();
            break;
        }
    }
}

