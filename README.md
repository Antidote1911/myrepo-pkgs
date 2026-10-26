# myrepo-pkgs

Sources des paquets du dépôt **[packarch]** (binaires dans
[myrepo](https://github.com/Antidote1911/myrepo), servis par
`https://antidote1911.github.io/myrepo/$arch`).

## Paquets

Chaque paquet de thèmes contient **tout** ce qu'il faut : rien d'autre à
installer.

| Paquet | Contenu |
|---|---|
| `packarch-backgrounds` | les fonds d'écran du bureau XFCE (`/usr/share/backgrounds`) : ceux du sélecteur de thèmes (même dossier, sans sous-dossier ; liste par thème dans `theme-switcher.sh` de `packarch-xfce-settings`), dont nos `pastelmount` et `satanic`, `xfce-default.png` et `wallpaper-alt.png` (écran de connexion) |
| `packarch-backgrounds-openbox` | les autres fonds d'écran (thèmes Openbox, dont `packarch-default` et `sylvanas`), sortis de `packarch-backgrounds` en 2.0-5 pour que l'ISO XFCE ne les embarque pas ; dépendance de `packarch-openbox` |
| `packarch-icons` | les thèmes d'icônes du bureau XFCE : Luna (et nos variantes `Luna-Antidote-Dark`, `Luna-Green-Dark`, `Luna-Purple-Dark`, `Luna-Red-Dark`), Luna-Dark, Vimix-Black(-Dark), et ceux dont ils héritent : Packarch, Packarch-Dark, Papirus-Apps. Qogir vient de `qogir-icon-theme`, Papirus de `papirus-icon-theme` (dépendances) ; Zafiro retiré en 2.0-4 (utilisé par aucun thème) |
| `packarch-icons-openbox` | les thèmes d'icônes utilisés seulement par les thèmes Openbox : Hack, Luv-Folders(-Dark), `Qogir-Pink-Dark` (dossiers roses), et BeautyLine par `aur/beautyline` (dépendance) ; sortis de `packarch-icons` en 2.0-6 pour que l'ISO XFCE ne les embarque pas ; dépendance de `packarch-openbox` |
| `packarch-icons-extra` | les thèmes d'icônes utilisés par aucun thème (sortis de `packarch-icons` en 2.0-5 pour alléger l'ISO, qui ne l'installe pas) : Arc-Circle, Luna-Light, Nordic-Folders(-Bluish, -Green), Numix-Apps(-White), Vimix-White(-Dark), White-Folders, Win11(-Dark) ; dépend de `packarch-icons` (héritage et liens) |
| `packarch-themes` | les thèmes GTK du bureau XFCE (sélecteur de thèmes et écran de connexion) : Arc-Dark, Dracula, Greybird-dark, Gruvbox, Juno-Mirage, Nordic, Orchis-Red-Dark |
| `packarch-themes-openbox` | les autres thèmes GTK (Adapta, Arc, Blade, Catppuccin, Cyberpunk, Everforest, Groot, Hack, Juno, Kripton, Manhattan, Rick, Slime, Spark, Wave, White, Windows-10, et les nôtres : Satanic, Adapta-Bitmap, Arc-Bitmap ; Sweet par `aur/sweet-gtk-theme`, dépendance) et les thèmes de fenêtres Openbox (`OB-*`), sortis de `packarch-themes` en 2.0-5 pour que l'ISO XFCE ne les embarque pas ; dépendance de `packarch-openbox` |
| `packarch-cursors` | les curseurs : Bibata, Fluentt, Future, Lyra, Pear, Qogirr, Sweet, Vimix |
| `packarch-fonts` | toutes les polices (`/usr/share/fonts/packarch`), dont la police d'icônes `packarch` |
| `packarch-openbox` | bureau Openbox : polybar, menus rofi, dunst, picom, kitty, 10 thèmes (`~/.config/openbox/themes/*/theme.toml`), menus et scripts (`/usr/share/packarch/openbox`), icônes de dunst, `packarch-tcma` (actions de Thunar) et `packarch-reload-theme`, en français. La configuration va dans `/etc/skel` |
| `packarch-skin` | sélecteur de thèmes du bureau Openbox (Rust, sources dans `packarch-skin/skin`) : `packarch-skin apply THÈME`, `bar`, `list`, `check`, `openbox-menu`… |
| `packarch-system-settings` | réglages système communs aux bureaux XFCE et Openbox : greeter LightDM sombre (`lightdm-gtk-greeter.conf.d`), pavé numérique sur l'écran de connexion, pas d'extinction de l'écran par X, hooks pacman (Geany en nouvelle instance, entrées inutiles cachées des menus : `packarch-hide-desktop-entries`), cachy-update installé, son applet démarrée à chaque session et sa vérification quotidienne (`arch-update.timer`) activée pour tous les utilisateurs |
| `packarch-xfce-settings` | réglages XFCE de l'ISO (`cachyos-xfce-settings` 1.0.9 avec nos modifications : thème Arc-Dark/Luna-Purple-Dark/Qogir-Dark, Firefox, Geany, LilouCalc, Thunar, pas de veille ni de verrouillage, pavé numérique) et sélecteur de thèmes XFCE (`/usr/share/xfcethemes`) ; remplace `cachyos-xfce-settings`, en conflit avec `packarch-openbox` (fichiers communs de `/etc/skel`) |
| `packarch-zsh-config` | configuration zsh de CachyOS (`cachyos-zsh-config`) sans la dépendance inutile à vim, avec notre configuration powerlevel10k par défaut (utilisée sans `~/.p10k.zsh`) et l’alias `cleanup` corrigé ; fournit `cachyos-zsh-config` |
| `packarch-mirrorlist` | `/etc/pacman.d/packarch-mirrorlist` |
| `apps/*` | nos applications, compilées depuis le dernier tag de leur dépôt GitHub : `cryptyrust`, `bombicat`, `deadpool`, `liloucalc`, `nounours-player`, `packarch-rmatrix` (`apps/rmatrix`, effet « Matrix » en console) |
| `aur/*` | copies de PKGBUILD AUR : `gtkmm`, `nitrogen`, `xfce-polkit`, `lead-git`, `skippy-xd-git`, `ksuperkey`, `betterlockscreen`, `networkmanager-dmenu-git`, `light`, `beautyline` (thème d’icônes BeautyLine, dépendance de `packarch-icons-openbox`), `sweet-gtk-theme` (thème GTK Sweet, dépendance de `packarch-themes-openbox`) |

Les paquets de thèmes remplacent les anciens paquets séparés
(`packarch-icons-arc`, `packarch-icons-luna`…, `packarch-gtk-theme-*` : `replaces`).

**Crédits** : la plupart des thèmes, icônes, curseurs, polices, fonds d'écran
et scripts ont été rassemblés ou créés par Aditya Shakya (adi1090x), sous
licence GPL3 ; les mentions d'auteur des fichiers sont conservées.

Les `PKGBUILD` de nos applications sont uniquement ici (`apps/`) : leurs
dépôts n'ont pas à en contenir pour Arch Linux. Pour en ajouter une : créer
`apps/<nom>/PKGBUILD` avec `url=` pointant vers son dépôt GitHub (tags
`vX.Y.Z`) et la source `…/archive/refs/tags/v${pkgver}.tar.gz`.

## Utilisation

```sh
./build.sh                     # construit ce qui a changé, copie dans ../myrepo/x86_64
./build.sh -f packarch-icons   # reconstruit un paquet
../myrepo/update.sh            # base de données, commit et push du dépôt binaire

./update.sh                    # reprend les PKGBUILD de l'AUR dans aur/
./update.sh --apps             # met pkgver de apps/* au dernier tag GitHub
```

Mise à jour complète : `./update.sh --apps`, `./update.sh`, `./build.sh`,
puis `../myrepo/update.sh`.

Après une modification, incrémenter `pkgrel` (ou `pkgver`) des paquets
modifiés : `build.sh` ne reconstruit que les versions absentes du dépôt
binaire (`update.sh --apps` le fait pour les nouvelles versions des
applications). `makepkg.conf` : code x86-64 générique (le `makepkg.conf` de
CachyOS peut être en `-march=native`), sans LTO (le code C embarqué par les
crates Rust, zstd ou sqlite, ne se lierait pas) et compression forte (limite
de 100 Mo par fichier de GitHub).
