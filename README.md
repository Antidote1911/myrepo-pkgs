# myrepo-pkgs

Sources des paquets du dépôt **[packarch]** (binaires dans
[myrepo](https://github.com/Antidote1911/myrepo), servis par
`https://antidote1911.github.io/myrepo/$arch`).

## Paquets

Chaque paquet de thèmes contient **tout** ce qu'il faut : rien d'autre à
installer.

| Paquet | Contenu |
|---|---|
| `packarch-backgrounds` | tous les fonds d'écran (`/usr/share/backgrounds`), dont les nôtres (`pastelmount`, `satanic`, `packarch-default`) et ceux du sélecteur de thèmes XFCE (`/usr/share/backgrounds/xfcethemes/<couleur>/`, liens vers les images communes) |
| `packarch-icons` | tous les thèmes d'icônes : Packarch, Packarch-Dark, Arc-Circle, Hack, Luna (et nos variantes `Luna-Antidote-Dark`, `Luna-Green-Dark`, `Luna-Purple-Dark`, `Luna-Red-Dark`), Luv, Nordic, Numix, Papirus, Vimix, White, Win11, Zafiro. Qogir vient de `qogir-icon-theme` (dépendance, avec ses curseurs) |
| `packarch-themes` | tous les thèmes GTK (Adapta, Arc, Blade, Catppuccin, Cyberpunk, Dracula, Everforest, Groot, Gruvbox, Hack, Juno, Kripton, Manhattan, Nordic, Rick, Slime, Spark, Sweet, Wave, White, Windows-10, et les nôtres : Satanic, Greybird-dark, Orchis-Red-Dark, Adapta-Bitmap, Arc-Bitmap) et les thèmes de fenêtres Openbox (`OB-*`) |
| `packarch-cursors` | les curseurs : Bibata, Fluentt, Future, Lyra, Pear, Qogirr, Sweet, Vimix |
| `packarch-fonts` | toutes les polices (`/usr/share/fonts/packarch`), dont la police d'icônes `packarch` |
| `packarch-openbox` | bureau Openbox : polybar, menus rofi, dunst, plank, picom, kitty, 15 thèmes (`~/.config/openbox/themes/*/theme.toml`), menus et scripts (`/usr/share/packarch/openbox`), icônes de dunst, `packarch-tcma` (actions de Thunar) et `packarch-reload-theme`, en français. La configuration va dans `/etc/skel` |
| `packarch-skin` | sélecteur de thèmes du bureau Openbox (Rust, sources dans `packarch-skin/skin`) : `packarch-skin apply THÈME`, `bar`, `list`, `check`, `openbox-menu`… |
| `packarch-zsh-config` | configuration zsh de CachyOS (`cachyos-zsh-config`) sans la dépendance inutile à vim, avec notre configuration powerlevel10k par défaut (utilisée sans `~/.p10k.zsh`) et l’alias `cleanup` corrigé ; fournit `cachyos-zsh-config` |
| `packarch-mirrorlist` | `/etc/pacman.d/packarch-mirrorlist` |
| `packarch-rmatrix` | effet « Matrix » en console |
| `aur/*` | copies de PKGBUILD AUR : `gtkmm`, `nitrogen`, `xfce-polkit`, `lead-git`, `skippy-xd-git`, `ksuperkey`, `betterlockscreen`, `networkmanager-dmenu-git`, `light` |

Les paquets de thèmes remplacent les anciens paquets séparés
(`packarch-icons-*`, `packarch-gtk-theme-*` : `replaces`).

**Crédits** : la plupart des thèmes, icônes, curseurs, polices, fonds d'écran
et scripts ont été rassemblés ou créés par Aditya Shakya (adi1090x), sous
licence GPL3 ; les mentions d'auteur des fichiers sont conservées.

Les applications (`cryptyrust`, `liloucalc`, `nounours-player`, `bombicat`)
sont empaquetées depuis leur propre dépôt (dossier `packaging/`).

## Utilisation

```sh
./build.sh                     # construit ce qui a changé, copie dans ../myrepo/x86_64
./build.sh -f packarch-icons   # reconstruit un paquet
../myrepo/update.sh            # base de données, commit et push du dépôt binaire

./update.sh                    # reprend les PKGBUILD de l'AUR dans aur/
```

Après une mise à jour, incrémenter `pkgrel` (ou `pkgver`) des paquets
modifiés : `build.sh` ne reconstruit que les versions absentes du dépôt
binaire. `makepkg.conf` : code x86-64 générique (le `makepkg.conf` de CachyOS
peut être en `-march=native`) et compression forte (limite de 100 Mo par
fichier de GitHub).
