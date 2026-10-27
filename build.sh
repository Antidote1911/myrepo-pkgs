#!/usr/bin/env bash
#
# build.sh — builds the [packarch] packages of this directory and copies them
# to the binary repository (../myrepo/x86_64), replacing their older versions.
# Then run ../myrepo/update.sh (database, commit and push).
#
#   packarch-*/   packages of this repository (themes, icons, wallpapers...)
#   apps/*/       our applications, built from their GitHub tag
#                 (./update.sh --apps sets their version)
#   aur/*/        copies of AUR PKGBUILDs (./update.sh refreshes them)
#
# A package already in the binary repository with the same version is skipped
# (-f: rebuild). Packages without build() (themes, configuration) are built
# with --nodeps. Packages built here that an AUR package needs to be built
# (gtkmm for nitrogen), or that a package without build() needs in
# makedepends (packarch-skin for packarch-openbox, built first), are installed
# on this system as dependencies, then removed at the end. Run as a normal
# user.
#
# Usage: ./build.sh [-f] [PACKAGE...]   (default: all of them)

set -euo pipefail

cd "$(dirname "$(readlink -f "$0")")"
top="$(pwd)"
repo="$(readlink -f ../myrepo)/x86_64"
conf="${top}/makepkg.conf"

# AUR packages, dependencies first
aur_order=(gtkmm nitrogen xfce-polkit lead-git skippy-xd-git
    ksuperkey betterlockscreen networkmanager-dmenu-git light beautyline)

force=false
[[ ${1:-} == -f ]] && { force=true; shift; }
[[ -d $repo ]] || { echo "Dépôt binaire introuvable : $repo" >&2; exit 1; }

# Package directories to build, in order
dirs=()
if (( $# )); then
    for p in "$@"; do
        if [[ -f $p/PKGBUILD ]]; then dirs+=("${p%/}")
        elif [[ -f apps/$p/PKGBUILD ]]; then dirs+=("apps/$p")
        elif [[ -f aur/$p/PKGBUILD ]]; then dirs+=("aur/$p")
        else echo "Paquet inconnu : $p" >&2; exit 1
        fi
    done
else
    # packarch-skin first: makedepends of packarch-openbox
    dirs+=(packarch-skin)
    for d in */ apps/*/; do
        [[ -f $d/PKGBUILD && $d != packarch-skin/ ]] && dirs+=("${d%/}")
    done
    for p in "${aur_order[@]}"; do dirs+=("aur/$p"); done
    for d in aur/*/; do
        [[ " ${aur_order[*]} " == *" $(basename "$d") "* ]] ||
            { echo "aur/$(basename "$d") absent de aur_order dans build.sh" >&2; exit 1; }
    done
fi

installed_deps=()
remove_installed_deps() {
    (( ${#installed_deps[@]} )) || return 0
    echo "==> Suppression des dépendances de compilation : ${installed_deps[*]}"
    sudo pacman -Rns --noconfirm "${installed_deps[@]}" ||
        echo "À supprimer à la main : sudo pacman -Rns ${installed_deps[*]}" >&2
}
trap 'status=$?; remove_installed_deps; exit ${status}' EXIT

pkg_name() {
    local f="${1##*/}"
    echo "${f%-*-*-*}"
}

# Package file of NAME in the binary repository (empty if none)
repo_file() {
    local f
    for f in "${repo}"/*.pkg.tar.zst; do
        [[ -f ${f} && $(pkg_name "${f}") == "$1" ]] && { echo "${f}"; return 0; }
    done
    return 0
}

# Copies a package file to the binary repository, removing its other versions
add_pkg() {
    local file="$1" name f
    name="$(pkg_name "${file}")"
    for f in "${repo}"/*.pkg.tar.zst; do
        [[ -f ${f} && $(pkg_name "${f}") == "${name}" ]] && rm -f "${f}" "${f}.sig"
    done
    cp "${file}" "${repo}/"
    echo "  + ${file##*/}"
}

# Installs the packages of the binary repository that the PKGBUILD of the
# current directory needs, and that this system lacks (makedepends: only the
# build dependencies)
install_local_deps() {
    local dep file kinds='\(make\)\?depends'
    [[ ${1:-} == makedepends ]] && kinds=makedepends
    while read -r dep; do
        dep="${dep%%[<>=]*}"
        pacman -Qq "${dep}" &>/dev/null && continue
        file="$(repo_file "${dep}")"
        [[ -n ${file} ]] || continue
        sudo pacman -U --asdeps --noconfirm "${file}"
        installed_deps+=("${dep}")
    done < <(makepkg --printsrcinfo | sed -n "s/^\s*${kinds} = //p" | sort -u)
}

# PGP keys of the sources: keys/pgp/*.asc of the AUR repository, or keyserver
import_pgp_keys() {
    local key
    [[ -d keys/pgp ]] && gpg --quiet --import keys/pgp/*.asc 2>/dev/null
    while read -r key; do
        gpg --list-keys "${key}" &>/dev/null && continue
        gpg --recv-keys "${key}" ||
            gpg --keyserver hkps://keyserver.ubuntu.com --recv-keys "${key}"
    done < <(makepkg --printsrcinfo | sed -n 's/^\s*validpgpkeys = //p' | sort -u)
}

for d in "${dirs[@]}"; do
    echo "==> ${d}"
    cd "${top}/${d}"
    mapfile -t pkgs < <(makepkg --config "${conf}" --packagelist | grep -v -e '-debug-' -e '-docs-')
    if ! ${force}; then
        uptodate=true
        for p in "${pkgs[@]}"; do [[ -f ${repo}/${p##*/} ]] || uptodate=false; done
        if ${uptodate}; then echo "  à jour"; cd "${top}"; continue; fi
    fi
    if grep -q '^build()' PKGBUILD; then
        # Compiled package: its build dependencies are needed (the packages
        # of this repository only for the AUR ones, e.g. gtkmm for nitrogen)
        [[ ${d} == aur/* ]] && install_local_deps
        import_pgp_keys
        deps=(--syncdeps --rmdeps)
    else
        # Files only: the dependencies are only needed at installation, the
        # build dependencies of this repository are installed
        install_local_deps makedepends
        deps=(--nodeps)
    fi
    makepkg --config "${conf}" "${deps[@]}" --cleanbuild --clean --force --noconfirm
    # Read again: pkgver() of the -git packages updates the version
    mapfile -t pkgs < <(makepkg --config "${conf}" --packagelist | grep -v -e '-debug-' -e '-docs-')
    for p in "${pkgs[@]}"; do add_pkg "${p}"; rm -f "${p}"; done
    rm -f ./*-debug-*.pkg.tar.zst ./*-docs-*.pkg.tar.zst
    cd "${top}"
done

echo "Paquets copiés dans ${repo}. Ensuite : ../myrepo/update.sh"
