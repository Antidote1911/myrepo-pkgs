//! Polybar of the current theme (what launch-bar.sh, polybar.sh and the
//! polybar/launch.sh of each theme did).
//!
//! Before the first launch, themes/system.ini receives the battery, adapter, backlight
//! and network interface of the computer; before the first launch of a theme,
//! its config.ini gets the modules matching the hardware (no backlight or
//! battery module without one, "brightness" for non-Intel backlights,
//! "ethernet" for a wired connection).

use crate::edit::set_line;
use crate::{current, spawn, theme, Dirs};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

/// Stops every instance of PROC and waits for them to exit (5 s at most)
fn stop(proc: &str) {
    if crate::dry() {
        println!("[dry] pkill -x {proc}");
        return;
    }
    let _ = Command::new("pkill").args(["-x", proc]).status();
    for _ in 0..50 {
        let running = Command::new("pgrep")
            .args(["-u", &users_uid(), "-x", proc])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !running {
            return;
        }
        sleep(Duration::from_millis(100));
    }
}

fn users_uid() -> String {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1).map(str::to_string))
        })
        .unwrap_or_else(|| "0".into())
}

/// Name of the first entry of DIR whose name satisfies F
fn first_entry(dir: &str, f: impl Fn(&str, &Path) -> bool) -> Option<String> {
    let mut names: Vec<_> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    names.sort();
    names.into_iter().find(|(n, p)| f(n, p)).map(|(n, _)| n)
}

/// First power supply of type KIND ("Battery", "Mains") of the computer
/// itself: the batteries of mice, keyboards... have the scope "Device"
fn power_supply(kind: &str) -> Option<String> {
    first_entry("/sys/class/power_supply", |_, p| {
        let read = |f: &str| fs::read_to_string(p.join(f)).map(|t| t.trim().to_string()).unwrap_or_default();
        read("type") == kind && read("scope") != "Device"
    })
}

fn backlight() -> Option<String> {
    first_entry("/sys/class/backlight", |_, _| true)
}

fn interface() -> Option<String> {
    first_entry("/sys/class/net", |n, p| {
        n != "lo" && fs::read_to_string(p.join("operstate")).map(|s| s.trim() == "up").unwrap_or(false)
    })
}

/// themes/system.ini, once
fn system_values(themes: &Path) {
    let marker = themes.join(".system");
    if marker.exists() {
        return;
    }
    let file = themes.join("system.ini");
    if let Ok(mut t) = fs::read_to_string(&file) {
        for (key, value) in [
            ("sys_adapter", power_supply("Mains")),
            ("sys_battery", power_supply("Battery")),
            ("sys_graphics_card", backlight()),
            ("sys_network_interface", interface()),
        ] {
            if let Some(v) = value {
                set_line(&mut t, &format!("{key} = "), &format!("{key} = {v}"));
            }
        }
        let _ = fs::write(&file, t);
    }
    let _ = fs::write(marker, "");
}

/// Modules of the polybar config.ini of a theme matching the hardware, once
fn fix_modules(pbar: &Path) {
    let marker = pbar.join(".module");
    if marker.exists() {
        return;
    }
    let file = pbar.join("config.ini");
    if let Ok(mut t) = fs::read_to_string(&file) {
        match backlight() {
            None => t = t.replace("backlight", "bna"),
            Some(card) if !card.contains("intel_") => t = t.replace("backlight", "brightness"),
            _ => {}
        }
        if power_supply("Battery").is_none() {
            t = t.replace("battery", "btna");
        }
        if interface().is_some_and(|i| i.starts_with('e')) {
            t = t.replace("network", "ethernet");
        }
        let _ = fs::write(&file, t);
    }
    let _ = fs::write(marker, "");
}

fn monitors() -> Vec<String> {
    if crate::dry() {
        return vec!["MONITOR-1".into()];
    }
    Command::new("polybar")
        .arg("--list-monitors")
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|l| l.split(':').next().map(str::to_string))
                .filter(|m| !m.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Launches the polybar of the current theme on each monitor
pub fn launch(dirs: &Dirs) -> Result<(), String> {
    let id = current(dirs);
    let entry = theme::list(&dirs.themes)
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("thème courant introuvable : {id}"))?;

    system_values(&dirs.themes);
    let pbar = entry.dir.join("polybar");
    fix_modules(&pbar);
    let bars = entry.load().map(|t| t.polybar.bars).unwrap_or_else(|_| vec!["main".into()]);
    stop("polybar");
    let config = pbar.join("config.ini").to_string_lossy().into_owned();
    for mon in monitors() {
        for bar in &bars {
            spawn("polybar", &["-q", bar, "-c", &config], &[("MONITOR", &mon)]);
        }
    }
    Ok(())
}
