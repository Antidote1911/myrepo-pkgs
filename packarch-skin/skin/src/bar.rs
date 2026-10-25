//! Bar of the current theme: polybar or tint2 (what launch-bar.sh,
//! polybar.sh, tint2.sh and the polybar/launch.sh of each theme did).
//!
//! themes/.panel holds the bar in use (polybar by default). Before the first
//! polybar launch, themes/system.ini receives the battery, adapter, backlight
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Panel {
    Polybar,
    Tint2,
}

impl Panel {
    pub fn parse(s: &str) -> Option<Panel> {
        match s.trim() {
            "polybar" => Some(Panel::Polybar),
            "tint2" => Some(Panel::Tint2),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Panel::Polybar => "polybar",
            Panel::Tint2 => "tint2",
        }
    }
}

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

/// Separators between modules, removed with the module next to them
const SEPARATORS: [&str; 5] = ["sep", "sep2", "dot", "dot-alt", "space"];

/// Removes the modules REMOVE from the "modules-* = ..." line LINE, with the
/// LD/RD decoration around them and the separator before them (after them
/// when there is none before). The powerline transitions (2RD, 3LD...) stay:
/// each one goes from the background of the previous module to the next.
fn remove_modules(line: &str, remove: &[&str]) -> String {
    let Some((key, value)) = line.split_once('=') else { return line.to_string() };
    let mut m: Vec<&str> = value.split_whitespace().collect();
    while let Some(i) = m.iter().position(|x| remove.contains(x)) {
        let (mut a, mut b) = (i, i + 1);
        if a > 0 && b < m.len() && m[a - 1] == "LD" && m[b] == "RD" {
            a -= 1;
            b += 1;
        }
        if a > 0 && SEPARATORS.contains(&m[a - 1]) {
            a -= 1;
        } else if b < m.len() && SEPARATORS.contains(&m[b]) {
            b += 1;
        }
        m.drain(a..b);
    }
    format!("{key}= {}", m.join(" ")).trim_end().to_string()
}

/// Modules of the polybar config.ini of a theme matching the hardware, once
fn fix_modules(pbar: &Path) {
    let marker = pbar.join(".module");
    if marker.exists() {
        return;
    }
    let file = pbar.join("config.ini");
    if let Ok(mut t) = fs::read_to_string(&file) {
        let mut remove = Vec::new();
        match backlight() {
            None => remove.extend(["backlight", "backlight-bar"]),
            Some(card) if !card.contains("intel_") => t = t.replace("backlight", "brightness"),
            _ => {}
        }
        if power_supply("Battery").is_none() {
            remove.extend(["battery", "battery_bar"]);
        }
        if interface().is_some_and(|i| i.starts_with('e')) {
            t = t.replace("network", "ethernet");
        }
        if !remove.is_empty() {
            t = t
                .split('\n')
                .map(|l| if l.starts_with("modules-") { remove_modules(l, &remove) } else { l.to_string() })
                .collect::<Vec<_>>()
                .join("\n");
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

/// Launches the bar of the current theme; PANEL changes the bar in use
pub fn launch(dirs: &Dirs, panel: Option<Panel>) -> Result<(), String> {
    let pfile = dirs.themes.join(".panel");
    if let Some(p) = panel {
        fs::write(&pfile, format!("{}\n", p.name())).map_err(|e| format!("{} : {e}", pfile.display()))?;
    }
    let panel = fs::read_to_string(&pfile)
        .ok()
        .and_then(|s| Panel::parse(&s))
        .unwrap_or(Panel::Polybar);
    let id = current(dirs);
    let entry = theme::list(&dirs.themes)
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("thème courant introuvable : {id}"))?;

    match panel {
        Panel::Polybar => {
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
        }
        Panel::Tint2 => {
            stop("tint2");
            let rc = entry.dir.join("tint2/tint2rc").to_string_lossy().into_owned();
            spawn("tint2", &["-c", &rc], &[]);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NONE: [&str; 4] = ["backlight", "backlight-bar", "battery", "battery_bar"];

    #[test]
    fn modules_removed() {
        // default: dots, battery in LD/RD
        assert_eq!(
            remove_modules(
                "modules-right = volume dot backlight dot bluetooth dot network dot LD battery RD dot LD date RD",
                &NONE
            ),
            "modules-right = volume dot bluetooth dot network dot LD date RD"
        );
        // easy: battery last inside a LD/RD group
        assert_eq!(
            remove_modules("modules-right = LD tray RD space LD volume dot backlight dot network dot battery RD space date", &NONE),
            "modules-right = LD tray RD space LD volume dot network RD space date"
        );
        // forest: first module, no separator
        assert_eq!(
            remove_modules("modules-right = battery bluetooth network sep volume-bar backlight-bar tray", &NONE),
            "modules-right = bluetooth network sep volume-bar tray"
        );
        // hack: second bar
        assert_eq!(remove_modules("modules-right = volume-bar sep backlight-bar", &NONE), "modules-right = volume-bar");
        // adaptive: the powerline transitions stay
        assert_eq!(
            remove_modules("modules-right = 2RD volume 3RD backlight 4RD battery 5RD bluetooth", &NONE),
            "modules-right = 2RD volume 3RD 4RD 5RD bluetooth"
        );
        // only the battery
        assert_eq!(
            remove_modules("modules-right = volume sep backlight sep battery sep date", &["battery"]),
            "modules-right = volume sep backlight sep date"
        );
    }
}
