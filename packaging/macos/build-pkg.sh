#!/bin/bash
# Builds the Dive Lens installer for macOS (.pkg).
#
#   build-pkg.sh <plug-in folder> <version> <output .pkg> [installer identity]
#
# The installer puts the plug-in folder (DiveLens.lrplugin, with the app
# inside) in the current user's Lightroom Modules folder,
# ~/Library/Application Support/Adobe/Lightroom/Modules, where Lightroom
# loads it by itself, after removing the previous Dive Lens plug-in folder
# there (scripts/preinstall). With an identity ("Developer ID Installer: …", in the
# keychain search list) the installer is signed; notarization is a separate
# step (release workflow).
#
# INSTALL_LOCATION, relative to the user's home folder, overrides the Modules
# folder: only for trying the installer without touching Lightroom.
#
# Part of Dive Lens, GPL-3.0-or-later.

set -euo pipefail

if [ $# -lt 3 ]; then
    echo "usage: $0 <plug-in folder> <version> <output .pkg> [installer identity]" >&2
    exit 2
fi
plugin=$1
version=$2
out=$3
identity=${4:-}
location=${INSTALL_LOCATION:-/Library/Application Support/Adobe/Lightroom/Modules}
here=$(cd "$(dirname "$0")" && pwd)

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir "$work/root"
ditto "$plugin" "$work/root/$(basename "$plugin")"

# For every bundle in the payload (the app):
# - not relocatable: otherwise Installer may find an app with the same bundle
#   id elsewhere (a development build, an old download) and update that copy
#   instead of installing here;
# - not version-checked: otherwise installing an older version over a newer
#   one would skip the app and leave it out of step with the plug-in.
pkgbuild --analyze --root "$work/root" "$work/components.plist"
plist=$work/components.plist
set_bool() {
    /usr/libexec/PlistBuddy -c "Delete :$1" "$plist" > /dev/null 2>&1 || true
    /usr/libexec/PlistBuddy -c "Add :$1 bool $2" "$plist"
}
i=0
while /usr/libexec/PlistBuddy -c "Print :$i" "$plist" > /dev/null 2>&1; do
    set_bool "$i:BundleIsRelocatable" false
    set_bool "$i:BundleIsVersionChecked" false
    i=$((i + 1))
done

pkgbuild \
    --root "$work/root" \
    --component-plist "$work/components.plist" \
    --scripts "$here/scripts" \
    --identifier com.querton.divelens.plugin \
    --version "$version" \
    --install-location "$location" \
    "$work/DiveLens-component.pkg"

sed "s/@VERSION@/$version/" "$here/distribution.xml" > "$work/distribution.xml"

if [ -n "$identity" ]; then
    productbuild --distribution "$work/distribution.xml" --package-path "$work" \
        --resources "$here/resources" --sign "$identity" "$out"
else
    productbuild --distribution "$work/distribution.xml" --package-path "$work" \
        --resources "$here/resources" "$out"
fi
