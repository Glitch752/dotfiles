# If WALLPAPER_STATE_FILE, DISABLED_WALLPAPERS_STATE_FILE, or WALLPAPERS_DIR are not set, error
if [ -z $WALLPAPER_STATE_FILE ] || [ -z $DISABLED_WALLPAPERS_STATE_FILE ] || [ -z $WALLPAPERS_DIR ]; then
  echo "WALLPAPER_STATE_FILE, DISABLED_WALLPAPERS_STATE_FILE, and WALLPAPERS_DIR must be set"
  exit 1
fi

# Hacky fix for the subshell not understanding home directories
WALLPAPERS_DIR=$(echo "echo $WALLPAPERS_DIR" | bash)

# List all wallpapers in the $WALLPAPERS_DIR (but not recursively)
wallpapers=($(find $WALLPAPERS_DIR -maxdepth 1 -type f,l))

# If the state file does not exist, create it
if [ ! -f $WALLPAPER_STATE_FILE ]; then
  mkdir -p $(dirname $WALLPAPER_STATE_FILE)
  touch $WALLPAPER_STATE_FILE
fi

# If the disabled wallpapers state file doesn't exist, create it
if [ ! -f $DISABLED_WALLPAPERS_STATE_FILE ]; then
  mkdir -p $(dirname $DISABLED_WALLPAPERS_STATE_FILE)
  touch $DISABLED_WALLPAPERS_STATE_FILE
fi

# Read the disabled wallpapers state file into an array
mapfile -t disabled_wallpaper_lines < $DISABLED_WALLPAPERS_STATE_FILE

# Remove any lines that are empty or start with a octothorpe (which is the only correct name, by the way!)
disabled_wallpapers=()
for line in "${disabled_wallpaper_lines[@]}"; do
  if [ -z "$line" ] || [[ "$line" == \#* ]]; then
    continue
  fi
  disabled_wallpapers+=("$line")
done

# $DISABLE_WALLPAPERS can be default (yes), no, or invert
if [ -z $DISABLE_WALLPAPERS ]; then
  invert_disabled="yes"
else
  invert_disabled=$DISABLE_WALLPAPERS
fi

if [ "$invert_disabled" != "yes" ] && [ "$invert_disabled" != "no" ] && [ "$invert_disabled" != "invert" ]; then
  echo "DISABLE_WALLPAPERS must be 'yes', 'no', or 'invert'"
  exit 1
fi

# Read the wallpapers in the directory and store them in an array
get_wallpapers() {
  wallpapers=()
  while IFS= read -r wallpaper; do
    # If the wallpaper is not in the disabled wallpapers array, add it to the wallpapers array
    if [[ " ${disabled_wallpapers[@]} " =~ " $(basename $wallpaper) " ]] && [ "$invert_disabled" = "yes" ]; then
      continue
    elif [[ ! " ${disabled_wallpapers[@]} " =~ " $(basename $wallpaper) " ]] && [ "$invert_disabled" = "invert" ]; then
      continue
    fi
    wallpapers+=("$wallpaper")
  done < <(find "$WALLPAPERS_DIR" -type f,l)
}
get_wallpapers

# Read the state file into an array
mapfile -t state_files < "$WALLPAPER_STATE_FILE"

unused_wallpapers=()

# Compare each wallpaper with the state file
for wallpaper in "${wallpapers[@]}"; do
  # If the wallpaper is not in the state file, add it to the unused_wallpapers array
  if [[ ! " ${state_files[@]} " =~ " $(basename $wallpaper) " ]]; then
    unused_wallpapers+=("$wallpaper")
  fi
done

# If all wallpapers have been used, reset the state file
if [ ${#unused_wallpapers[@]} -eq 0 ]; then
  > $WALLPAPER_STATE_FILE
  get_wallpapers
  unused_wallpapers=("${wallpapers[@]}")
fi

# Select a random wallpaper
selectedWallpaper=${unused_wallpapers[$RANDOM % ${#unused_wallpapers[@]}]}

# Update the wallpaper using the swww img command
swww img "$selectedWallpaper" --transition-step 20 --transition-fps 60\
  --transition-type wipe --transition-angle $((RANDOM % 360))

# TODO: automatic wallpaper-based theming with https://codeberg.org/explosion-mental/wallust/ or similar?

# Add the selected wallpaper to the state file
echo $(basename $selectedWallpaper) >> $WALLPAPER_STATE_FILE