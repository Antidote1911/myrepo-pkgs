#!/usr/bin/env bash
#
# update.sh — refreshes the copies of the AUR packages (aur/*/: PKGBUILD and
# its files) from the AUR. Then bump nothing: the AUR pkgver/pkgrel are kept;
# run ./build.sh.
#
# Usage: ./update.sh [PACKAGE...]   (default: all of aur/)

set -euo pipefail

cd "$(dirname "$(readlink -f "$0")")"
top="$(pwd)"

pkgs=("$@")
(( ${#pkgs[@]} )) || mapfile -t pkgs < <(ls aur)

for p in "${pkgs[@]}"; do
    echo "==> aur/${p}"
    tmp="$(mktemp -d)"
    git clone -q "https://aur.archlinux.org/${p}.git" "${tmp}/${p}"
    rm -rf "aur/${p}"
    mkdir -p "aur/${p}"
    (cd "${tmp}/${p}" && git ls-files | grep -vx '\.gitignore' |
        while IFS= read -r f; do install -Dm644 "${f}" "${top}/aur/${p}/${f}"; done)
    rm -rf "${tmp}"
    echo "  $(sed -n 's/^\s*pkgver = //p' "aur/${p}/.SRCINFO" | head -1)"
done
git status --short aur | head -20
