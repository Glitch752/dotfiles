# Builds our custom stylesheet by modifying adwaita

USE_POSTCSS="false"

if [ ! -d "libadwaita" ]; then
    git clone https://gitlab.gnome.org/GNOME/libadwaita/ libadwaita
else
    cd libadwaita
    git pull
    cd ..
fi
mkdir -p src/adwaita-stylesheet/
cp -r libadwaita/src/stylesheet/* src/adwaita-stylesheet/

if [ ! -d "adw-gtk3" ]; then
    git clone https://github.com/lassekongo83/adw-gtk3/ adw-gtk3
else
    cd adw-gtk3
    git pull
    cd ..
fi
mkdir -p src/adw-gtk3-stylesheet/
cp -r adw-gtk3/src/* src/adw-gtk3-stylesheet/

# Copy src/_sass/adw-gtk3-colors.scss to src/adw-gtk3-stylesheet/sass/_colors.scss to "patch it" and let us use our own colors
rm -f src/adw-gtk3-stylesheet/sass/_defaults.scss
cp src/_sass/adw-gtk3-colors.scss src/adw-gtk3-stylesheet/sass/_colors.scss

SASSC_OPT="-M -t expanded"
DART_SASS_OPT="--style=expanded --no-source-map"

SRC_DIR=$(cd $(dirname $0) && pwd)

echo "Compiling GTK css"
sassc $SASSC_OPT src/gtk/theme-4.0/gtk.{scss,css}

if [ "$USE_POSTCSS" = "true" ]; then
    sassc $SASSC_OPT src/gtk/theme-3.0/gtk{.scss,-unsupported.css}
    pnpm i
    pnpm postcss src/gtk/theme-3.0/gtk-unsupported.css -o src/gtk/theme-3.0/gtk.css
else
    sass $DART_SASS_OPT src/gtk/theme-3.0/gtk.{scss,css}
fi

echo "Installing"
sh ./install.sh