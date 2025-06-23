DEST_DIR="$HOME/.themes"

SRC_DIR=$(cd $(dirname $0) && pwd)

THEME_NAME=CustomDesktopTheme

usage() {
cat << EOF
Usage: $0 [-rh]
Options:
  -r: Uninstall/Remove installed themes
  -h: Show help
EOF
}

install() {
  local THEME_DIR=${DEST_DIR}/${THEME_NAME}

  echo "Installing '${THEME_DIR}'..."
  mkdir -p ${THEME_DIR}

  # GTK 3.0
  mkdir -p ${THEME_DIR}/gtk-3.0
  cp -r ${SRC_DIR}/src/adwaita-stylesheet/assets ${THEME_DIR}/gtk-3.0/assets
  cp -r ${SRC_DIR}/src/gtk/theme-3.0/gtk.css ${THEME_DIR}/gtk-3.0/gtk.css
  cp -r ${SRC_DIR}/src/gtk/theme-3.0/gtk.css ${THEME_DIR}/gtk-3.0/gtk-dark.css

  # GTK 4.0
  mkdir -p ${THEME_DIR}/gtk-4.0
  cp -r ${SRC_DIR}/src/adwaita-stylesheet/assets ${THEME_DIR}/gtk-4.0/assets
  cp -r ${SRC_DIR}/src/gtk/theme-4.0/gtk.css ${THEME_DIR}/gtk-4.0/gtk.css
  cp -r ${SRC_DIR}/src/gtk/theme-4.0/gtk.css ${THEME_DIR}/gtk-4.0/gtk-dark.css
}

while [[ $# -gt 0 ]]; do
  case "${1}" in
    -r)
      remove='true'
      shift
      ;;
    -h)
      usage
      exit 0
      ;;
    *)
      echo -e "Unrecognized option '$1'."
      exit 1
      ;;
  esac
done

uninstall() {
  local THEME_DIR="${DEST_DIR}/${THEME_NAME}"

  if [[ -d "$THEME_DIR" ]]; then
    rm -rf "$THEME_DIR"
    echo -e "Uninstalling "$THEME_DIR" ..."
  fi
}

uninstall_link() {
  rm -rf "${HOME}/.config/gtk-4.0"/{assets,gtk.css,gtk-dark.css}
}

link_libadwaita() {
  local THEME_DIR=${DEST_DIR}/${THEME_NAME}

  echo -e "Link '$THEME_DIR/gtk-4.0' to '${HOME}/.config/gtk-4.0' for libadwaita..."

  mkdir -p "${HOME}/.config/gtk-4.0"
  ln -sf "${THEME_DIR}/gtk-4.0/assets" "${HOME}/.config/gtk-4.0/assets"
  ln -sf "${THEME_DIR}/gtk-4.0/gtk.css" "${HOME}/.config/gtk-4.0/gtk.css"
  ln -sf "${THEME_DIR}/gtk-4.0/gtk-dark.css" "${HOME}/.config/gtk-4.0/gtk-dark.css"

  echo -e "Link '${THEME_DIR}/gtk-3.0' to '${HOME}/.config/gtk-3.0' for GTK 3..."

  mkdir -p "${HOME}/.config/gtk-3.0"
  ln -sf "${THEME_DIR}/gtk-3.0/gtk.css" "${HOME}/.config/gtk-3.0/gtk.css"
}

if [[ "${remove:-}" != 'true' ]]; then
  uninstall && install
  uninstall_link && link_libadwaita
fi

if [[ "${remove:-}" == 'true' ]]; then
  echo -e "Uninstall ${HOME}/.config/gtk-4.0 links ..."
  uninstall
  uninstall_link
fi

echo -e "\nDone."
