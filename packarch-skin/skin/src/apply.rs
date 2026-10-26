//! Application of a theme (what the apply.sh script of each theme did), to
//! the running session or, at package build time, to the files of /etc/skel.

use crate::color::{Color, Palette};
use crate::edit::{set_block, set_line, set_xml_text, truncate_at};
use crate::theme::{Entry, Theme};
use crate::{bar, run, spawn, xml_escape, Dirs};
use std::fs;
use std::path::Path;

/// Where a theme is applied
#[derive(Clone, Copy)]
pub enum Target<'a> {
    /// The running session: the files, then xfconf, gsettings, nitrogen and
    /// the reload of Openbox, dunst, kitty and the bar
    Session,
    /// The files only, under the package root ROOT (packarch-openbox build):
    /// HOME is ROOT/etc/skel, the xfconf settings go to xsettings.xml, the
    /// wallpaper to nitrogen/bg-saved.cfg and the color scheme to a GSettings
    /// override in ROOT/usr/share/glib-2.0/schemas. Nothing is run.
    Package(&'a Path),
}

/// Colors of the roles, resolved
struct Colors {
    term_bg: Color,
    term_fg: Color,
    palette: Vec<Color>,
    bar_bg: Color,
    bar_fg: Color,
    bar_alt_bg: Color,
    bar_alt_fg: Color,
    bar_accent: Color,
    gradient: Vec<Color>,
    extra: Vec<(String, Color)>,
    rofi: [Color; 6],
    dunst: [Color; 4],
    border: Color,
}

fn resolve(theme: &Theme) -> Result<Colors, String> {
    let mut defs = theme.colors.clone();
    // Defaults of the derived colors (as apply.sh computed them)
    defs.entry("alt_background".into()).or_insert("lighten(background, 0.05)".into());
    defs.entry("alt_foreground".into()).or_insert("darken(foreground, 0.30)".into());
    let mut p = Palette::new(&defs);
    let r = &theme.roles;

    let bar_bg = p.eval(&r.bar_background)?;
    let bar_alt_bg = p.eval(&r.bar_alt_background)?;
    if theme.polybar.gradient {
        p.gradient = bar_bg
            .rgb
            .gradient(bar_alt_bg.rgb, 7)
            .into_iter()
            .map(|rgb| Color { text: rgb.hex(), rgb })
            .collect();
    }
    let mut palette = Vec::with_capacity(16);
    for i in 0..16 {
        palette.push(p.get(&format!("color{i}"))?);
    }
    let mut extra = Vec::new();
    for (k, v) in &theme.polybar.extra {
        extra.push((k.clone(), p.eval(v)?));
    }
    Ok(Colors {
        term_bg: p.get("background")?,
        term_fg: p.get("foreground")?,
        palette,
        bar_fg: p.eval(&r.bar_foreground)?,
        bar_alt_fg: p.eval(&r.bar_alt_foreground)?,
        bar_accent: p.eval(&r.bar_accent)?,
        gradient: p.gradient.clone(),
        extra,
        rofi: [
            p.eval(&r.rofi_background)?,
            p.eval(&r.rofi_alt_background)?,
            p.eval(&r.rofi_foreground)?,
            p.eval(&r.rofi_selected)?,
            p.eval(&r.rofi_active)?,
            p.eval(&r.rofi_urgent)?,
        ],
        dunst: [
            p.eval(&r.dunst_background)?,
            p.eval(&r.dunst_foreground)?,
            p.eval(&r.dunst_frame)?,
            p.eval(&r.dunst_urgent)?,
        ],
        border: p.eval(&r.screenshot_border)?,
        bar_bg,
        bar_alt_bg,
    })
}

/// Checks that every color of THEME resolves
pub fn check(theme: &Theme) -> Result<(), String> {
    resolve(theme).map(|_| ())
}

/// Reads FILE, applies F, writes it back if it changed. A missing file is
/// skipped (the user may have removed it).
fn edit_file(file: &Path, f: impl FnOnce(&mut String)) {
    let Ok(old) = fs::read_to_string(file) else { return };
    let mut text = old.clone();
    f(&mut text);
    if text != old {
        if let Err(e) = fs::write(file, text) {
            eprintln!("{} : {e}", file.display());
        }
    }
}

fn write_file(file: &Path, text: &str) {
    if let Some(d) = file.parent() {
        let _ = fs::create_dir_all(d);
    }
    if let Err(e) = fs::write(file, text) {
        eprintln!("{} : {e}", file.display());
    }
}

fn xfconf(channel: &str, prop: &str, value: &str) {
    run("xfconf-query", &["-c", channel, "-p", prop, "-s", value]);
}

/// Sets the value of the xfconf property NAME (string) in the channel file
/// TEXT (xfce-perchannel-xml)
fn set_xfconf_file(text: &mut String, name: &str, value: &str) {
    set_line(
        text,
        &format!("<property name=\"{name}\" "),
        &format!("<property name=\"{name}\" type=\"string\" value=\"{}\"/>", xml_escape(value)),
    );
}

pub fn apply(dirs: &Dirs, entry: &Entry, target: Target) -> Result<(), String> {
    let theme = entry.load()?;
    let c = resolve(&theme)?;
    let id = &entry.id;
    let tdir = &entry.dir;
    let conf = &dirs.config;
    let obox = conf.join("openbox");
    let session = matches!(target, Target::Session);

    if session {
        run(
            "dunstify",
            &[
                "-u", "normal", "-h", "string:x-dunst-stack-tag:applytheme",
                "-i", "/usr/share/packarch/icons/dunst/themes.png",
                &format!("Application du style : {}", entry.display_name(Some(&theme))),
            ],
        );
    }
    write_file(&dirs.themes.join(".current"), &format!("{id}\n"));

    // Wallpaper
    let wallpaper = tdir.join(&theme.wallpaper);
    let wallpaper = wallpaper.to_string_lossy();
    if session {
        for head in 0..=10 {
            let h = format!("--head={head}");
            run("nitrogen", &[h.as_str(), "--save", "--set-zoom-fill", &wallpaper]);
        }
    } else {
        // The theme directory of the package is not the one of the user
        if !Path::new(&theme.wallpaper).is_absolute() {
            return Err(format!("{id} : le fond d'écran doit être un chemin absolu : {}", theme.wallpaper));
        }
        let mut cfg = String::new();
        for head in 0..=10 {
            cfg += &format!("[xin_{head}]\nfile={wallpaper}\nmode=5\nbgcolor=#000000\n\n");
        }
        write_file(&conf.join("nitrogen/bg-saved.cfg"), &cfg);
    }

    // Polybar
    let pbar = tdir.join("polybar");
    edit_file(&pbar.join("config.ini"), |t| {
        set_line(t, "font-0 = ", &format!("font-0 = \"{}\"", theme.fonts.polybar));
    });
    let mut colors = format!(
        "[color]\n\nBACKGROUND = {}\nFOREGROUND = {}\nALTBACKGROUND = {}\nALTFOREGROUND = {}\nACCENT = {}\n\n",
        c.bar_bg.text, c.bar_fg.text, c.bar_alt_bg.text, c.bar_alt_fg.text, c.bar_accent.text
    );
    let names = ["BLACK", "RED", "GREEN", "YELLOW", "BLUE", "MAGENTA", "CYAN", "WHITE"];
    for (i, col) in c.palette.iter().enumerate() {
        let n = if i < 8 { names[i].to_string() } else { format!("ALT{}", names[i - 8]) };
        colors += &format!("{n} = {}\n", col.text);
    }
    if !c.gradient.is_empty() {
        colors += "\n";
        for (i, g) in c.gradient.iter().enumerate().take(7).skip(1) {
            colors += &format!("BACKGROUND{i} = {}\n", g.text);
        }
    }
    if !c.extra.is_empty() {
        colors += "\n";
        for (k, v) in &c.extra {
            colors += &format!("{k} = {}\n", v.text);
        }
    }
    write_file(&pbar.join("colors.ini"), &colors);
    if theme.polybar.polywins {
        edit_file(&pbar.join("scripts/polywins.sh"), |t| {
            set_line(t, "active_text_color=", &format!("active_text_color='{}'", c.bar_accent.text));
            set_line(t, "inactive_text_color=", &format!("inactive_text_color='{}'", c.bar_fg.text));
            set_line(t, "minimized_text_color=", &format!("minimized_text_color='{}'", c.bar_alt_fg.text));
        });
    }

    // Rofi (and the scripts that use the rofi themes of the current theme)
    let border = c.border.rgb.rgb_float();
    for s in ["rofi-screenshot", "ob-screenshot"] {
        edit_file(&obox.join("scripts").join(s), |t| {
            set_line(t, "border=", &format!("border='{border}'"));
        });
    }
    for s in ["rofi-askpass", "rofi-bluetooth", "rofi-launcher", "rofi-powermenu", "rofi-runner", "rofi-screenshot"] {
        edit_file(&obox.join("scripts").join(s), |t| {
            set_line(t, "STYLE=", &format!("STYLE=\"{id}\""));
        });
    }
    let rofi = tdir.join("rofi");
    edit_file(&rofi.join("shared/fonts.rasi"), |t| {
        set_line(t, "font:", &format!("font: \"{}\";", theme.fonts.rofi));
    });
    let r = &c.rofi;
    write_file(
        &rofi.join("shared/colors.rasi"),
        &format!(
            "* {{\n    background:     {};\n    background-alt: {};\n    foreground:     {};\n    selected:       {};\n    active:         {};\n    urgent:         {};\n}}\n",
            r[0].text, r[1].text, r[2].text, r[3].text, r[4].text, r[5].text
        ),
    );
    edit_file(&conf.join("rofi/config.rasi"), |t| {
        set_line(t, "icon-theme:", &format!("icon-theme: \"{}\";", theme.appearance.rofi_icons));
    });
    // Package: ~/... (the home of the package is not the one of the user;
    // rofi expands ~ in -theme)
    let nm_theme = rofi.join("networkmenu.rasi");
    let nm_theme = match (target, nm_theme.strip_prefix(&dirs.home)) {
        (Target::Package(_), Ok(rel)) => format!("~/{}", rel.display()),
        _ => nm_theme.display().to_string(),
    };
    edit_file(&conf.join("networkmanager-dmenu/config.ini"), |t| {
        set_line(t, "dmenu_command = ", &format!("dmenu_command = rofi -dmenu -theme {nm_theme}"));
    });

    // Terminal (kitty)
    let f = &theme.fonts;
    let kitty = conf.join("kitty");
    if kitty.join("fonts.conf").is_file() && kitty.join("colors.conf").is_file() {
        edit_file(&kitty.join("fonts.conf"), |t| {
            let n = &f.terminal;
            set_block(
                t,
                "#-fn-start",
                "#-fn-end",
                &format!("font_family {n}\nbold_font {n}\nitalic_font {n}\nbold_italic_font {n}"),
            );
            set_line(t, "font_size ", &format!("font_size {}", f.terminal_size));
        });
        let mut t = format!(
            "## Colors configuration\nbackground {bg}\nforeground {fg}\nselection_background {fg}\nselection_foreground {bg}\ncursor {fg}\n\n",
            bg = c.term_bg.text,
            fg = c.term_fg.text
        );
        for i in 0..8 {
            t += &format!("color{i} {}\ncolor{} {}\n", c.palette[i].text, i + 8, c.palette[i + 8].text);
        }
        write_file(&kitty.join("colors.conf"), &t);
        if session {
            run("pkill", &["-USR1", "-x", "kitty"]);
        }
    }

    // Geany
    let a = &theme.appearance;
    edit_file(&conf.join("geany/geany.conf"), |t| {
        set_line(t, "color_scheme=", &format!("color_scheme={}", a.geany_colors));
        set_line(t, "editor_font=", &format!("editor_font={}", f.geany));
    });

    // GTK theme, icons, cursor and fonts
    let scheme = format!("'{}'", a.color_scheme);
    match target {
        Target::Session => {
            xfconf("xsettings", "/Gtk/FontName", &f.gtk);
            xfconf("xsettings", "/Net/ThemeName", &a.gtk_theme);
            xfconf("xsettings", "/Net/IconThemeName", &a.icon_theme);
            xfconf("xsettings", "/Gtk/CursorThemeName", &a.cursor_theme);
            run("gsettings", &["set", "org.gnome.desktop.interface", "color-scheme", &scheme]);
            run("gsettings", &["set", "org.x.apps.portal", "color-scheme", &scheme]);
        }
        Target::Package(root) => {
            edit_file(&conf.join("xfce4/xfconf/xfce-perchannel-xml/xsettings.xml"), |t| {
                set_xfconf_file(t, "FontName", &f.gtk);
                set_xfconf_file(t, "ThemeName", &a.gtk_theme);
                set_xfconf_file(t, "IconThemeName", &a.icon_theme);
                set_xfconf_file(t, "CursorThemeName", &a.cursor_theme);
            });
            // A schema that is not installed (org.x.apps.portal without
            // xapp) is ignored by glib-compile-schemas
            write_file(
                &root.join("usr/share/glib-2.0/schemas/25_packarch-openbox.gschema.override"),
                &format!(
                    "# Color scheme of the default theme ({id}), written by packarch-skin render\n\
                     [org.gnome.desktop.interface]\ncolor-scheme={scheme}\n\n[org.x.apps.portal]\ncolor-scheme={scheme}\n"
                ),
            );
        }
    }
    // Read by the GTK applications started without xfsettingsd
    edit_file(&conf.join("gtk-3.0/settings.ini"), |t| {
        set_line(t, "gtk-theme-name=", &format!("gtk-theme-name={}", a.gtk_theme));
        set_line(t, "gtk-icon-theme-name=", &format!("gtk-icon-theme-name={}", a.icon_theme));
        set_line(t, "gtk-cursor-theme-name=", &format!("gtk-cursor-theme-name={}", a.cursor_theme));
        set_line(t, "gtk-font-name=", &format!("gtk-font-name={}", f.gtk));
    });
    edit_file(&dirs.home.join(".gtkrc-2.0"), |t| {
        set_line(t, "gtk-theme-name=", &format!("gtk-theme-name=\"{}\"", a.gtk_theme));
        set_line(t, "gtk-icon-theme-name=", &format!("gtk-icon-theme-name=\"{}\"", a.icon_theme));
        set_line(t, "gtk-cursor-theme-name=", &format!("gtk-cursor-theme-name=\"{}\"", a.cursor_theme));
        set_line(t, "gtk-font-name=", &format!("gtk-font-name=\"{}\"", f.gtk));
    });
    edit_file(&dirs.home.join(".icons/default/index.theme"), |t| {
        set_line(t, "Inherits=", &format!("Inherits={}", a.cursor_theme));
    });

    // Openbox
    let o = &theme.openbox;
    edit_file(&obox.join("rc.xml"), |x| {
        let th = ("theme", None);
        set_xml_text(x, &[th, ("name", None)], &o.theme);
        set_xml_text(x, &[th, ("titleLayout", None)], &o.layout);
        // Menu items: feather first for the glyphs of menu-glyphs.xml (their
        // codepoints are also in other icon fonts, with other widths)
        let menu_font = format!("{},feather", f.openbox);
        for (place, weight) in [("ActiveWindow", "Bold"), ("InactiveWindow", "Normal"), ("MenuHeader", "Bold"), ("MenuItem", "Normal"), ("ActiveOnScreenDisplay", "Bold"), ("InactiveOnScreenDisplay", "Normal")] {
            let font = ("font", Some(("place", place)));
            let name = if place == "MenuItem" { &menu_font } else { &f.openbox };
            set_xml_text(x, &[th, font, ("name", None)], name);
            set_xml_text(x, &[th, font, ("size", None)], &f.openbox_size.to_string());
            set_xml_text(x, &[th, font, ("weight", None)], weight);
            set_xml_text(x, &[th, font, ("slant", None)], "Normal");
        }
        set_xml_text(x, &[("menu", None), ("file", None)], &o.menu);
        for (side, v) in ["top", "bottom", "left", "right"].iter().zip(o.margins) {
            set_xml_text(x, &[("margins", None), (side, None)], &v.to_string());
        }
    });
    if session {
        run("openbox", &["--reconfigure"]);
    }

    // Dunst
    let d = &theme.dunst;
    edit_file(&conf.join("dunst/dunstrc"), |t| {
        for (k, v) in [
            ("width", d.width.to_string()),
            ("height", d.height.to_string()),
            ("offset", d.offset.clone()),
            ("origin", d.origin.clone()),
            ("font", f.dunst.clone()),
            ("frame_width", d.border.to_string()),
            ("separator_height", d.separator.to_string()),
            ("line_height", d.separator.to_string()),
        ] {
            set_line(t, &format!("{k} = "), &format!("{k} = {v}"));
        }
        truncate_at(t, "urgency_low");
        let [bg, fg, frame, urgent] = &c.dunst;
        t.push_str(&format!(
            "[urgency_low]\ntimeout = 2\nbackground = \"{bg}\"\nforeground = \"{fg}\"\nframe_color = \"{frame}\"\n\n\
             [urgency_normal]\ntimeout = 5\nbackground = \"{bg}\"\nforeground = \"{fg}\"\nframe_color = \"{frame}\"\n\n\
             [urgency_critical]\ntimeout = 0\nbackground = \"{bg}\"\nforeground = \"{urgent}\"\nframe_color = \"{urgent}\"\n",
            bg = bg.text, fg = fg.text, frame = frame.text, urgent = urgent.text
        ));
    });
    if session && run("pkill", &["-x", "dunst"]) {
        spawn("dunst", &[], &[]);
    }

    // Compositor
    let pc = &theme.picom;
    edit_file(&conf.join("picom.conf"), |t| {
        set_line(t, "backend = ", &format!("backend = \"{}\";", pc.backend));
        set_block(t, "#-cr-start", "#-cr-end", &format!("corner-radius = {};", pc.corner_radius));
        set_line(t, "shadow-radius = ", &format!("shadow-radius = {};", pc.shadow_radius));
        set_line(t, "shadow-opacity = ", &format!("shadow-opacity = {};", pc.shadow_opacity));
        set_line(t, "shadow-offset-x = ", &format!("shadow-offset-x = {};", pc.shadow_offset_x));
        set_line(t, "shadow-offset-y = ", &format!("shadow-offset-y = {};", pc.shadow_offset_y));
        set_line(t, "method = ", &format!("method = \"{}\";", pc.blur_method));
        set_line(t, "strength = ", &format!("strength = {};", pc.blur_strength));
    });

    if session {
        bar::launch(dirs)?;
        run("xsetroot", &["-cursor_name", "left_ptr"]);
    }
    Ok(())
}
