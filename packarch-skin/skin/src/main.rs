//! packarch-skin — theme (skin) chooser of the Packarch Openbox desktop.
//!
//! The themes are the directories of ~/.config/openbox/themes that hold a
//! theme.toml (colors, fonts, GTK/icon/cursor themes, Openbox, dunst,
//! picom settings) next to their polybar/ and rofi/ layouts. Applying
//! a theme writes these settings into the configuration of each program and
//! relaunches the bar.

mod apply;
mod bar;
mod color;
mod edit;
mod theme;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

const USAGE: &str = "\
Usage : packarch-skin COMMANDE

  list                    thèmes disponibles (identifiant et nom)
  current                 thème courant
  apply [THÈME]           applique THÈME (par défaut le thème courant)
  bar                     relance la barre du thème courant
  wallpaper [THÈME]       chemin du fond d'écran de THÈME
  check [THÈME...]        vérifie les fichiers theme.toml
  openbox-menu            menu Openbox (pipemenu) des thèmes";

pub struct Dirs {
    pub home: PathBuf,
    pub config: PathBuf,
    pub themes: PathBuf,
}

impl Dirs {
    fn new() -> Dirs {
        let home = PathBuf::from(env::var_os("HOME").unwrap_or_else(|| "/".into()));
        let config = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home.join(".config"));
        let themes = config.join("openbox/themes");
        Dirs { home, config, themes }
    }
}

/// PACKARCH_SKIN_DRY=1: the external commands are printed, not run (tests)
pub fn dry() -> bool {
    env::var_os("PACKARCH_SKIN_DRY").is_some_and(|v| v == "1")
}

/// Runs a command, waits for it; false (with a message if it is missing)
/// when it fails
pub fn run(cmd: &str, args: &[&str]) -> bool {
    run_input(cmd, args, None)
}

/// Same as run, with INPUT written to the standard input of the command
pub fn run_input(cmd: &str, args: &[&str], input: Option<&str>) -> bool {
    if dry() {
        println!("[dry] {cmd} {}", args.join(" "));
        return true;
    }
    let mut c = Command::new(cmd);
    c.args(args).stdout(Stdio::null()).stderr(Stdio::null());
    if input.is_some() {
        c.stdin(Stdio::piped());
    }
    let child = c.spawn().and_then(|mut ch| {
        if let (Some(text), Some(mut stdin)) = (input, ch.stdin.take()) {
            use std::io::Write;
            let _ = stdin.write_all(text.as_bytes());
        }
        ch.wait()
    });
    match child {
        Ok(s) => s.success(),
        Err(_) => {
            eprintln!("packarch-skin : commande introuvable : {cmd}");
            false
        }
    }
}

/// Starts a command in the background (it keeps running after us)
pub fn spawn(cmd: &str, args: &[&str], envs: &[(&str, &str)]) {
    if dry() {
        let e: Vec<String> = envs.iter().map(|(k, v)| format!("{k}={v} ")).collect();
        println!("[dry] {}{cmd} {} &", e.concat(), args.join(" "));
        return;
    }
    let mut c = Command::new(cmd);
    c.args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    for (k, v) in envs {
        c.env(k, v);
    }
    if c.spawn().is_err() {
        eprintln!("packarch-skin : commande introuvable : {cmd}");
    }
}

/// Identifier of the current theme (themes/.current, "default" if none)
pub fn current(dirs: &Dirs) -> String {
    fs::read_to_string(dirs.themes.join(".current"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "default".into())
}

fn find(dirs: &Dirs, id: &str) -> Result<theme::Entry, String> {
    theme::list(&dirs.themes)
        .into_iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("thème inconnu : {id} (voir « packarch-skin list »)"))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn openbox_menu(dirs: &Dirs) {
    let themes = theme::list(&dirs.themes);
    let cur = current(dirs);
    let mut out = String::from("<openbox_pipe_menu>\n");
    out += &format!("  <separator label=\"⁌ Thèmes ({}) ⁍\"/>\n", themes.len());
    for (i, e) in themes.iter().enumerate() {
        let name = e.display_name(e.load().ok().as_ref());
        let mark = if e.id == cur { "  ✓" } else { "" };
        out += &format!(
            "  <item label=\"{}. {}{}\">\n    <action name=\"Execute\">\n      <command>packarch-skin apply {}</command>\n    </action>\n  </item>\n",
            i + 1,
            xml_escape(&name).replace('_', "__"),
            mark,
            xml_escape(&e.id)
        );
    }
    out += "</openbox_pipe_menu>\n";
    print!("{out}");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let dirs = Dirs::new();
    let arg = |i: usize| args.get(i).map(String::as_str);
    let result = match arg(0) {
        Some("list") => {
            for e in theme::list(&dirs.themes) {
                let name = e.display_name(e.load().ok().as_ref());
                println!("{}\t{}", e.id, name);
            }
            Ok(())
        }
        Some("current") => {
            println!("{}", current(&dirs));
            Ok(())
        }
        Some("apply") => {
            let id = arg(1).map(str::to_string).unwrap_or_else(|| current(&dirs));
            find(&dirs, &id).and_then(|e| apply::apply(&dirs, &e))
        }
        Some("bar") => bar::launch(&dirs),
        Some("wallpaper") => {
            let id = arg(1).map(str::to_string).unwrap_or_else(|| current(&dirs));
            find(&dirs, &id).and_then(|e| {
                let t = e.load()?;
                println!("{}", e.dir.join(t.wallpaper).display());
                Ok(())
            })
        }
        Some("check") => {
            let all = theme::list(&dirs.themes);
            let wanted: Vec<&str> = args[1..].iter().map(String::as_str).collect();
            let mut errors = 0;
            for e in all.iter().filter(|e| wanted.is_empty() || wanted.contains(&e.id.as_str())) {
                match e.load().and_then(|t| apply::check(&t)) {
                    Ok(()) => println!("{} : ok", e.id),
                    Err(err) => {
                        println!("{} : {err}", e.id);
                        errors += 1;
                    }
                }
            }
            if errors > 0 { Err(format!("{errors} thème(s) en erreur")) } else { Ok(()) }
        }
        Some("openbox-menu") => {
            openbox_menu(&dirs);
            Ok(())
        }
        Some("-h") | Some("--help") | Some("help") => {
            println!("{USAGE}");
            Ok(())
        }
        _ => {
            eprintln!("{USAGE}");
            exit(2);
        }
    };
    if let Err(e) = result {
        eprintln!("packarch-skin : {e}");
        exit(1);
    }
}
