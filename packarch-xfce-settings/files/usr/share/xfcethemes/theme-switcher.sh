#!/usr/bin/env bash
#
# theme-switcher.sh — applique un thème (fond d'écran) sur le tableau de
# bord XFCE4, sans menu : le thème est passé en argument.
#
# Usage :
#   theme-switcher.sh --red
#   theme-switcher.sh red
#
# Thèmes disponibles : blue gray green purple red yellow default
#
# Choisit un fond d'écran aléatoire parmi ceux du thème (WALLPAPERS)
# et l'applique à tous les écrans/workspaces XFCE.
#
# Pour l'ajouter au panneau : créer un Lanceur par thème (clic droit sur
# le panneau > Nouvel élément... > Lanceur), commande :
#   /usr/share/xfcethemes/theme-switcher.sh --red

set -euo pipefail

# Fonds d'écran : paquet packarch-backgrounds, tous dans le même dossier
# (pas de sous-dossier par thème : nitrogen les afficherait en double)
WALLPAPERS_DIR="/usr/share/backgrounds"
DEFAULT_WALLPAPER="$WALLPAPERS_DIR/xfce-default.png"
declare -A WALLPAPERS=(
    [blue]="Background-Wallpaper-HD-Wallpapers.jpg chat2.jpg pastelmount.jpg tealize.png wall6.jpg wave.jpg"
    [gray]="adaptive.png Black-Wallpaper-For-PC.jpg default.jpg dracula.png keyboards.jpg nord.jpg"
    [green]="extra-18.jpg forest.jpg nature-12.jpg nature-13.jpg nature-14.jpg rainy.jpeg"
    [purple]="1719702444938428.jpg cyberpunk1.jpg cyberpunk2.jpg tokyo.png wallpaper-alt.png"
    [red]="abstract2.jpg antidote3.jpg dragon-wallpapers-best-wallpapers.jpg redforest.jpg satanic.jpg"
    [yellow]="bouquet.jpg nature-11.jpg nature-9.jpg"
)
# L'état est propre à chaque utilisateur : le script peut être installé dans
# un dossier système (/usr/share/xfcethemes) non accessible en écriture.
STATE_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/xfcethemes"
STATE_FILE="$STATE_DIR/.current_theme"
LAST_WALLPAPER_DIR="$STATE_DIR/.last_wallpaper"

THEMES=(blue gray green purple red yellow default)

notify() {
    command -v notify-send >/dev/null 2>&1 && notify-send -a "Thème XFCE" "$1" "$2" || true
}

usage() {
    echo "Usage : $(basename "$0") --<theme>"
    echo "Thèmes disponibles : ${THEMES[*]}"
}

# Valide et normalise le thème passé en argument (accepte --red ou red).
parse_theme() {
    local arg="${1:-}"
    arg="${arg#--}"
    arg="${arg#-}"

    local t
    for t in "${THEMES[@]}"; do
        if [[ "$t" == "$arg" ]]; then
            echo "$t"
            return 0
        fi
    done

    return 1
}

# Choisit une image aléatoire parmi celles du thème donné.
# Le thème "default" est un cas particulier : il utilise toujours le même
# fichier fixe, xfce-default.png, plutôt qu'un choix aléatoire.
pick_random_wallpaper() {
    local theme="$1"

    if [[ "$theme" == "default" ]]; then
        if [[ ! -f "$DEFAULT_WALLPAPER" ]]; then
            notify "Erreur" "Fichier introuvable : $DEFAULT_WALLPAPER"
            exit 1
        fi
        echo "$DEFAULT_WALLPAPER"
        return 0
    fi

    local candidates="" file last_file last_image image
    for file in ${WALLPAPERS[$theme]}; do
        [[ -f "$WALLPAPERS_DIR/$file" ]] && candidates+="$WALLPAPERS_DIR/$file"$'\n'
    done
    candidates="${candidates%$'\n'}"
    [[ -n "$candidates" ]] || return 0

    last_file="$LAST_WALLPAPER_DIR/$theme"
    if [[ -f "$last_file" ]]; then
        last_image="$(cat "$last_file")"
        # N'exclut le dernier fond d'écran que s'il reste au moins un autre choix.
        local filtered
        filtered="$(grep -v -F -x "$last_image" <<< "$candidates")"
        [[ -n "$filtered" ]] && candidates="$filtered"
    fi

    image="$(shuf -n1 <<< "$candidates")"

    mkdir -p "$LAST_WALLPAPER_DIR"
    echo "$image" > "$last_file"

    echo "$image"
}

# Applique l'image donnée à tous les écrans/workspaces connus de xfce4-desktop.
apply_wallpaper() {
    local image="$1"
    local props monitor ws count base

    props="$(xfconf-query -c xfce4-desktop -l 2>/dev/null | grep -E '(image-path|last-image)$' || true)"

    while IFS= read -r prop; do
        [[ -z "$prop" ]] && continue
        xfconf-query -c xfce4-desktop -p "$prop" -s "$image"
    done <<< "$props"

    # xfdesktop 4.20 lit /backdrop/screen0/monitor<SORTIE>/workspace<N>/last-image.
    # Sur une session neuve (ISO live, nouvel utilisateur) ces propriétés
    # n'existent pas encore : on les crée pour chaque écran et chaque bureau.
    count="$(xfconf-query -c xfwm4 -p /general/workspace_count 2>/dev/null || echo 4)"
    while read -r monitor; do
        [[ -z "$monitor" ]] && continue
        for ((ws = 0; ws < count; ws++)); do
            base="/backdrop/screen0/monitor${monitor}/workspace${ws}"
            xfconf-query -c xfce4-desktop -p "$base/last-image" -n -t string -s "$image"
            xfconf-query -c xfce4-desktop -p "$base/image-style" -n -t int -s 5
        done
    done < <(xrandr --listmonitors 2>/dev/null | awk 'NR > 1 { print $NF }')

    # Force xfdesktop à recharger le fond d'écran immédiatement.
    xfdesktop --reload >/dev/null 2>&1 || true
}

# Associe chaque thème de couleur à un thème d'icônes GTK.
# Les thèmes sans correspondance ci-dessous conservent le thème d'icônes actuel.
apply_icon_theme() {
    local theme="$1"
    local icon_theme=""

    case "$theme" in
        yellow) icon_theme="Luna-Dark" ;;
        green)  icon_theme="Luna-Green-Dark" ;;
        purple) icon_theme="Luna-Purple-Dark" ;;
        red)    icon_theme="Luna-Red-Dark" ;;
        blue)   icon_theme="Luna-Antidote-Dark" ;;
        gray)   icon_theme="Vimix-Black-Dark" ;;
        default) icon_theme="Luna-Purple-Dark" ;;
    esac

    [[ -z "$icon_theme" ]] && return 0

    xfconf-query -c xsettings -p /Net/IconThemeName -s "$icon_theme"
}

# Associe chaque thème de couleur à un thème GTK.
# Les thèmes sans correspondance ci-dessous conservent le thème GTK actuel.
apply_gtk_theme() {
    local theme="$1"
    local gtk_theme=""

    case "$theme" in
        yellow) gtk_theme="Juno-Mirage" ;;
        blue)   gtk_theme="Greybird-dark" ;;
        purple) gtk_theme="Dracula" ;;
        green)  gtk_theme="Gruvbox" ;;
        red)    gtk_theme="Orchis-Red-Dark" ;;
        gray)   gtk_theme="Nordic" ;;
        default) gtk_theme="Arc-Dark" ;;
    esac

    [[ -z "$gtk_theme" ]] && return 0

    xfconf-query -c xsettings -p /Net/ThemeName -s "$gtk_theme"
    xfconf-query -c xfwm4 -p /general/theme -s "$gtk_theme" 2>/dev/null || true
}

# Applique une palette de couleurs assortie au thème dans xfce4-terminal.
# Depuis Xfce 4.20, ces réglages vivent dans le canal xfconf "xfce4-terminal"
# (et non plus dans le fichier terminalrc). La palette ANSI (16 couleurs)
# reste celle de Tango pour garder une bonne lisibilité ; seuls le fond, le
# texte, le curseur et le gras sont teintés.
apply_terminal_theme() {
    local theme="$1"
    local fg="#e8e8e8" bg accent
    local palette="#000000;#cc0000;#4e9a06;#c4a000;#3465a4;#75507b;#06989a;#d3d7cf;#555753;#ef2929;#8ae234;#fce94f;#729fcf;#ad7fa8;#34e2e2;#eeeeec"

    case "$theme" in
        blue)    bg="#10182a"; accent="#2563EB" ;;
        gray)    bg="#181818"; accent="#6B7280" ;;
        green)   bg="#0f1f14"; accent="#16A34A" ;;
        purple)  bg="#1c1024"; accent="#9333EA" ;;
        red)     bg="#220f10"; accent="#DC2626" ;;
        yellow)  bg="#221c08"; accent="#EAB308" ;;
        default) bg="#1c1024"; accent="#9333EA" ;;
    esac

    xfconf-query -c xfce4-terminal -p /color-use-theme -n -t bool -s false 2>/dev/null || \
        xfconf-query -c xfce4-terminal -p /color-use-theme -s false
    xfconf-query -c xfce4-terminal -p /color-foreground -n -t string -s "$fg" 2>/dev/null || \
        xfconf-query -c xfce4-terminal -p /color-foreground -s "$fg"
    xfconf-query -c xfce4-terminal -p /color-background -n -t string -s "$bg" 2>/dev/null || \
        xfconf-query -c xfce4-terminal -p /color-background -s "$bg"
    xfconf-query -c xfce4-terminal -p /color-cursor-use-default -n -t bool -s false 2>/dev/null || \
        xfconf-query -c xfce4-terminal -p /color-cursor-use-default -s false
    xfconf-query -c xfce4-terminal -p /color-cursor -n -t string -s "$accent" 2>/dev/null || \
        xfconf-query -c xfce4-terminal -p /color-cursor -s "$accent"
    xfconf-query -c xfce4-terminal -p /color-bold-use-default -n -t bool -s false 2>/dev/null || \
        xfconf-query -c xfce4-terminal -p /color-bold-use-default -s false
    xfconf-query -c xfce4-terminal -p /color-bold -n -t string -s "$accent" 2>/dev/null || \
        xfconf-query -c xfce4-terminal -p /color-bold -s "$accent"
    xfconf-query -c xfce4-terminal -p /color-palette -n -t string -s "$palette" 2>/dev/null || \
        xfconf-query -c xfce4-terminal -p /color-palette -s "$palette"
    xfconf-query -c xfce4-terminal -p /tab-activity-color -n -t string -s "$accent" 2>/dev/null || \
        xfconf-query -c xfce4-terminal -p /tab-activity-color -s "$accent"
}

# Place l'entrée du thème appliqué en tête du lanceur des thèmes dans le
# tableau de bord XFCE. Le lanceur affiche l'icône de sa première entrée et la
# liste est enregistrée dans xfconf : le bouton indique donc le thème actif,
# y compris après un redémarrage.
update_panel_button() {
    local theme="$1"
    local current="theme-$theme.desktop"
    local prop items item
    local args

    while IFS= read -r prop; do
        [[ -z "$prop" ]] && continue
        items="$(xfconf-query -c xfce4-panel -p "$prop" 2>/dev/null | grep -E '\.desktop$' || true)"
        grep -qxF "$current" <<< "$items" || continue

        args=(-t string -s "$current")
        while IFS= read -r item; do
            [[ -z "$item" || "$item" == "$current" ]] && continue
            args+=(-t string -s "$item")
        done <<< "$items"

        xfconf-query -c xfce4-panel -p "$prop" "${args[@]}"
    done < <(xfconf-query -c xfce4-panel -l 2>/dev/null | grep -E '^/plugins/plugin-[0-9]+/items$' || true)
}

main() {
    local theme image

    if [[ $# -ne 1 ]]; then
        usage
        exit 1
    fi

    if ! theme="$(parse_theme "$1")"; then
        echo "Thème inconnu : $1" >&2
        usage
        exit 1
    fi

    image="$(pick_random_wallpaper "$theme")"
    if [[ -z "$image" ]]; then
        notify "Erreur" "Aucune image trouvée pour le thème \"$theme\"."
        exit 1
    fi

    apply_wallpaper "$image"
    apply_icon_theme "$theme"
    apply_gtk_theme "$theme"
    apply_terminal_theme "$theme"
    update_panel_button "$theme"

    mkdir -p "$STATE_DIR"
    echo "$theme" > "$STATE_FILE"
    notify "Thème appliqué : $theme" "$(basename "$image")"
}

main "$@"
