#!/usr/bin/env bash
#
# update.sh — refreshes the PKGBUILDs before ./build.sh:
#   - aur/*/: copies of the AUR packages (PKGBUILD and its files), taken from
#     the AUR as is (their pkgver/pkgrel are kept)
#   - apps/*/ (--apps): our applications. pkgver is set to the latest vX.Y.Z
#     tag of the GitHub repository of url= (pkgrel back to 1 when it
#     changes); the packages without tags (pkgver() function, e.g. rmatrix)
#     get the version of their last commit
#
# Usage: ./update.sh [PACKAGE...]          (default: all of aur/)
#        ./update.sh --apps [PACKAGE...]   (default: all of apps/)

set -euo pipefail

cd "$(dirname "$(readlink -f "$0")")"
top="$(pwd)"
conf="${top}/makepkg.conf"

update_aur() {
    local p="$1" tmp
    echo "==> aur/${p}"
    tmp="$(mktemp -d)"
    git clone -q "https://aur.archlinux.org/${p}.git" "${tmp}/${p}"
    rm -rf "aur/${p}"
    mkdir -p "aur/${p}"
    (cd "${tmp}/${p}" && git ls-files | grep -vx '\.gitignore' |
        while IFS= read -r f; do install -Dm644 "${f}" "${top}/aur/${p}/${f}"; done)
    rm -rf "${tmp}"
    echo "  $(sed -n 's/^\s*pkgver = //p' "aur/${p}/.SRCINFO" | head -1)"
}

update_app() {
    local p="$1" pkgbuild="apps/$1/PKGBUILD" url old new
    echo "==> apps/${p}"
    [[ -f ${pkgbuild} ]] || { echo "Paquet inconnu : ${p}" >&2; return 1; }
    old="$(sed -n 's/^pkgver=//p' "${pkgbuild}")"
    if grep -q '^pkgver()' "${pkgbuild}"; then
        # VCS package: makepkg fetches the sources and writes pkgver itself
        (cd "apps/${p}" && makepkg --config "${conf}" --nodeps --nobuild \
            --noprepare --skipinteg >/dev/null 2>&1) ||
            { echo "  échec de makepkg dans apps/${p}" >&2; return 1; }
        new="$(sed -n 's/^pkgver=//p' "${pkgbuild}")"
    else
        url="$(sed -n "s/^url=[\"']\?\([^\"']*\)[\"']\?$/\1/p" "${pkgbuild}")"
        new="$(git ls-remote --tags --refs "${url}.git" |
            sed -n 's|.*refs/tags/v\([0-9][0-9.]*\)$|\1|p' | sort -V | tail -1)"
        [[ -n ${new} ]] || { echo "  aucun tag vX.Y.Z sur ${url}" >&2; return 1; }
        sed -i "s/^pkgver=.*/pkgver=${new}/" "${pkgbuild}"
    fi
    if [[ ${new} != "${old}" ]]; then
        sed -i 's/^pkgrel=.*/pkgrel=1/' "${pkgbuild}"
        echo "  ${old} -> ${new}"
    else
        echo "  ${new} (à jour)"
    fi
}

if [[ ${1:-} == --apps ]]; then
    shift
    pkgs=("$@")
    (( ${#pkgs[@]} )) || mapfile -t pkgs < <(ls apps)
    for p in "${pkgs[@]}"; do update_app "${p}"; done
    git status --short apps | head -20
else
    pkgs=("$@")
    (( ${#pkgs[@]} )) || mapfile -t pkgs < <(ls aur)
    for p in "${pkgs[@]}"; do update_aur "${p}"; done
    git status --short aur | head -20
fi
