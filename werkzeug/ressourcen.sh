#!/bin/sh
# Erzeugt data/resource/rechner.gresource aus rechner.gresource.xml (Bauzeit-
# Artefakt, eingecheckt, damit der Bau ohne glib-compile-resources geht).
# Das Fenstersymbol ist eine Kopie von data/icons/hicolor/scalable/apps/
# org.querencia.Rechner.svg unter eigenem Namen (nur SVG, siehe docs/building.md).
set -e
cd "$(dirname "$0")/.."
cp data/icons/hicolor/scalable/apps/org.querencia.Rechner.svg data/resource/icons/scalable/apps/org.querencia.Rechner-fenster.svg
glib-compile-resources --sourcedir=data/resource --target=data/resource/rechner.gresource data/resource/rechner.gresource.xml
