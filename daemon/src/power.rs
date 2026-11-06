use std::process::Command;

pub fn execute_power_action(action: &str) -> String {
    match action {
        "lock" => {
            let _ = Command::new("loginctl").arg("lock-session").spawn();
            "Layar dikunci".to_string()
        }
        "suspend" => {
            let _ = Command::new("systemctl").arg("suspend").spawn();
            "Memasuki mode tidur".to_string()
        }
        "reboot" => {
            let _ = Command::new("systemctl").arg("reboot").spawn();
            "Memulai ulang sistem".to_string()
        }
        "poweroff" => {
            let _ = Command::new("systemctl").arg("poweroff").spawn();
            "Mematikan sistem".to_string()
        }
        _ => "Aksi tidak dikenal".to_string(),
    }
}
