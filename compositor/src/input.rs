use std::process::Command;

const BACKLIGHT_PATH: &str = "/sys/class/backlight";
const PANIC_SCRIPT: &str = "/usr/local/bin/panic";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FnAction {
    None,
    Panic,
    Stealth,
    Launcher,
    Terminal,
    KillWindow,
    OpenCode,
    Lock,
    NmapQuick,
    BtScan,
    ShellListen,
    WifiToggle,
    CamSnap,
    Ide,
    IrScan,
    C6L,
    MeshChat,
    ZigbeeScan,
    OpenCodeAsk,
    Doom,
    Retro,
    Youtube,
    WebUI,
    MediaBox,
    HttpProbe,
    PrivescCheck,
    PostHarvest,
    SysTui,
    NetTui,
    LootTui,
    ScanTui,
}

mod keycodes {
    pub const ESC: u32 = 1;
    pub const TAB: u32 = 15;
    pub const ENTER: u32 = 28;
    pub const SPACE: u32 = 57;
    pub const LEFTALT: u32 = 56;
    pub const Q: u32 = 16;
    pub const P: u32 = 25;
    pub const O: u32 = 24;
    pub const L: u32 = 38;
    pub const N: u32 = 49;
    pub const B: u32 = 48;
    pub const S: u32 = 31;
    pub const W: u32 = 17;
    pub const C: u32 = 46;
    pub const I: u32 = 23;
    pub const A: u32 = 30;
    pub const G: u32 = 34;
    pub const H: u32 = 35;
    pub const R: u32 = 19;
    pub const Y: u32 = 21;
    pub const M: u32 = 50;
    pub const U: u32 = 22;
    pub const D: u32 = 32;
    pub const X: u32 = 45;
    pub const Z: u32 = 44;
    pub const K: u32 = 37;
    pub const V: u32 = 47;
    pub const E: u32 = 18;
    pub const F: u32 = 33;   // sys-tui
    pub const T: u32 = 20;   // scan-tui
    pub const J_KEY: u32 = 36; // net-tui (not 'j' nav — compositor intercepts only with Fn held)
}

#[derive(Debug, Clone)]
pub struct InputHandler {
    fn_held: bool,
    fn_keycode: u32,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            fn_held: false,
            fn_keycode: keycodes::LEFTALT,
        }
    }

    pub fn handle_key_press(&mut self, keycode: u32) -> Option<FnAction> {
        if keycode == self.fn_keycode {
            self.fn_held = true;
            return None;
        }

        if !self.fn_held {
            return None;
        }

        let action = match keycode {
            keycodes::P => Some(FnAction::Panic),
            keycodes::SPACE => Some(FnAction::Stealth),
            keycodes::TAB => Some(FnAction::Launcher),
            keycodes::ENTER => Some(FnAction::Terminal),
            keycodes::Q => Some(FnAction::KillWindow),
            keycodes::O => Some(FnAction::OpenCode),
            keycodes::L => Some(FnAction::Lock),
            keycodes::N => Some(FnAction::NmapQuick),
            keycodes::B => Some(FnAction::BtScan),
            keycodes::S => Some(FnAction::ShellListen),
            keycodes::W => Some(FnAction::WifiToggle),
            keycodes::C => Some(FnAction::CamSnap),
            keycodes::I => Some(FnAction::Ide),
            keycodes::H => Some(FnAction::IrScan),
            keycodes::D => Some(FnAction::C6L),
            keycodes::X => Some(FnAction::MeshChat),
            keycodes::Z => Some(FnAction::ZigbeeScan),
            keycodes::A => Some(FnAction::OpenCodeAsk),
            keycodes::G => Some(FnAction::Doom),
            keycodes::R => Some(FnAction::Retro),
            keycodes::Y => Some(FnAction::Youtube),
            keycodes::U => Some(FnAction::WebUI),
            keycodes::M => Some(FnAction::MediaBox),
            keycodes::K => Some(FnAction::HttpProbe),
            keycodes::V => Some(FnAction::PrivescCheck),
            keycodes::E => Some(FnAction::PostHarvest),
            keycodes::F => Some(FnAction::SysTui),
            keycodes::T => Some(FnAction::ScanTui),
            keycodes::J_KEY => Some(FnAction::NetTui),
            _ => None,
        };

        if let Some(a) = action {
            self.execute_action(a);
        }

        action
    }

    pub fn handle_key_release(&mut self, keycode: u32) {
        if keycode == self.fn_keycode {
            self.fn_held = false;
        }
    }

    fn execute_action(&self, action: FnAction) {
        let (program, args): (&str, Vec<&str>) = match action {
            FnAction::Panic => {
                spawn_bg(PANIC_SCRIPT, &[]);
                return;
            }
            FnAction::Stealth => {
                toggle_backlight();
                return;
            }
            FnAction::Launcher => ("cyber_launcher", vec![]),
            FnAction::Terminal => ("st", vec!["-e", "tmux"]),
            FnAction::KillWindow => return,
            FnAction::OpenCode => ("opencode-session", vec![]),
            FnAction::Lock => ("device-lock", vec!["lock"]),
            FnAction::NmapQuick => ("st", vec!["-e", "sudo", "net-quickscan"]),
            FnAction::BtScan => ("st", vec!["-e", "sudo", "bt-scan"]),
            FnAction::ShellListen => ("st", vec!["-e", "quick-c2", "listen"]),
            FnAction::WifiToggle => ("cardputer-wifi-toggle", vec![]),
            FnAction::CamSnap => ("cam-snap", vec![]),
            FnAction::Ide => ("st", vec!["-e", "micro"]),
            FnAction::IrScan => ("st", vec!["-e", "sudo", "ir-scan"]),
            FnAction::C6L => ("st", vec!["-e", "c6l-ctl", "serial"]),
            FnAction::MeshChat => ("st", vec!["-e", "mesh-chat"]),
            FnAction::ZigbeeScan => ("st", vec!["-e", "sudo", "c6l-ctl", "zigbee", "scan"]),
            FnAction::OpenCodeAsk => return,
            FnAction::Doom => ("st", vec!["-e", "doom-play", "play"]),
            FnAction::Retro => ("st", vec!["-e", "retro-play"]),
            FnAction::Youtube => ("st", vec!["-e", "yt", "search"]),
            FnAction::WebUI => ("webui-toggle", vec![]),
            FnAction::MediaBox => ("jellyfin-tv", vec![]),
            FnAction::HttpProbe   => ("st", vec!["-e", "http-probe", "auto"]),
            FnAction::PrivescCheck => ("st", vec!["-e", "privesc-check", "quick"]),
            FnAction::PostHarvest  => ("st", vec!["-e", "post-harvest", "quick"]),
            FnAction::SysTui       => ("st", vec!["-e", "sys-tui"]),
            FnAction::NetTui       => ("st", vec!["-e", "net-tui"]),
            FnAction::LootTui      => ("st", vec!["-e", "loot-tui"]),
            FnAction::ScanTui      => ("st", vec!["-e", "scan-tui"]),
            FnAction::None => return,
        };

        spawn_bg(program, &args);
    }

    pub fn set_fn_keycode(&mut self, code: u32) {
        self.fn_keycode = code;
    }
}

fn spawn_bg(program: &str, args: &[&str]) {
    let prog = program.to_string();
    let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let hdmi = if std::path::Path::new("/sys/class/drm/card0-HDMI-A-1/status")
        .exists()
    {
        match std::fs::read_to_string("/sys/class/drm/card0-HDMI-A-1/status") {
            Ok(s) if s.trim() == "connected" => "1",
            _ => "0",
        }
    } else {
        "0"
    };
    std::thread::spawn(move || {
        let mut cmd = Command::new(&prog);
        for arg in &args_owned {
            cmd.arg(arg);
        }
        cmd.env("WAYLAND_DISPLAY", "wayland-0")
            .env("SDL_VIDEODRIVER", "wayland")
            .env("SDL_RENDER_DRIVER", "opengles2")
            .env("ZERODAY_HDMI", hdmi)
            .env("ZERODAY_DISPLAY", if hdmi == "1" { "hdmi" } else { "lcd" });
        match cmd.spawn() {
            Ok(_) => log::debug!("Spawned {}", prog),
            Err(e) => log::warn!("Failed to spawn {}: {}", prog, e),
        }
    });
}

fn toggle_backlight() {
    use std::fs;
    if let Ok(entries) = fs::read_dir(BACKLIGHT_PATH) {
        for entry in entries.flatten() {
            let bl_path = entry.path();
            let cur_path = bl_path.join("brightness");
            let max_path = bl_path.join("max_brightness");
            let power_path = bl_path.join("bl_power");

            if let (Ok(cur_str), Ok(max_str)) = (
                fs::read_to_string(&cur_path),
                fs::read_to_string(&max_path),
            ) {
                let cur: u32 = cur_str.trim().parse().unwrap_or(0);
                let max: u32 = max_str.trim().parse().unwrap_or(1);

                if cur > 0 {
                    let _ = fs::write(&power_path, "1");
                } else {
                    let _ = fs::write(&power_path, "0");
                    let _ = fs::write(&cur_path, format!("{}", max * 3 / 4));
                }
            }
        }
    }
}