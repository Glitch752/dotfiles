

# Thu Jun 12 01:42:19 PM CDT 2025 - Unknown packages


AddPackage alacritty # A cross-platform, GPU-accelerated terminal emulator
AddPackage base # Minimal package set to define a basic Arch Linux installation
AddPackage base-devel # Basic tools to build Arch Linux packages
AddPackage btrfs-progs # Btrfs filesystem utilities
AddPackage efibootmgr # Linux user-space application to modify the EFI Boot Manager
AddPackage fastfetch # A feature-rich and performance oriented neofetch like system information tool
AddPackage firefox # Fast, Private & Safe Web Browser
AddPackage fuzzel # Application launcher for wlroots based Wayland compositors
AddPackage fzf # Command-line fuzzy finder
AddPackage git # the fast distributed version control system
AddPackage grub # GNU GRand Unified Bootloader (2)
AddPackage gst-plugin-pipewire # Multimedia graph framework - pipewire plugin
AddPackage htop # Interactive process viewer
AddPackage intel-media-driver # Intel Media Driver for VAAPI — Broadwell+ iGPUs
AddPackage intel-ucode # Microcode update files for Intel CPUs
AddPackage iwd # Internet Wireless Daemon
AddPackage libpulse # A featureful, general-purpose sound server (client library)
AddPackage libva-intel-driver # VA-API implementation for Intel G45 and HD Graphics family
AddPackage lightdm # A lightweight display manager
AddPackage lightdm-slick-greeter # A slick-looking LightDM greeter
AddPackage linux # The Linux kernel and modules
AddPackage linux-firmware # Firmware files for Linux
AddPackage lynx # A text browser for the World Wide Web
AddPackage mako # Lightweight notification daemon for Wayland
AddPackage nano # Pico editor clone with enhancements
AddPackage network-manager-applet # Applet for managing network connections
AddPackage networkmanager # Network connection manager and user applications
AddPackage niri # A scrollable-tiling Wayland compositor
AddPackage noto-fonts-cjk # Google Noto CJK fonts
AddPackage noto-fonts-emoji # Google Noto emoji fonts
AddPackage noto-fonts-extra # Google Noto TTF fonts - additional variants
AddPackage os-prober # Utility to detect other OSes on a set of drives
AddPackage pipewire # Low-latency audio/video router and processor
AddPackage pipewire-alsa # Low-latency audio/video router and processor - ALSA configuration
AddPackage pipewire-jack # Low-latency audio/video router and processor - JACK replacement
AddPackage pipewire-pulse # Low-latency audio/video router and processor - PulseAudio replacement
AddPackage smartmontools # Control and monitor S.M.A.R.T. enabled ATA and SCSI Hard Drives
AddPackage sof-firmware # Sound Open Firmware
AddPackage swaybg # Wallpaper tool for Wayland compositors
AddPackage swayidle # Idle management daemon for Wayland
AddPackage swaylock # Screen locker for Wayland
AddPackage thefuck # Magnificent app which corrects your previous console command
AddPackage tree # A directory listing program displaying a depth indented list of files
AddPackage vim # Vi Improved, a highly configurable, improved version of the vi text editor
AddPackage vulkan-intel # Open-source Vulkan driver for Intel GPUs
AddPackage wakatime # Command line interface used by all WakaTime text editor plugins
AddPackage waybar # Highly customizable Wayland bar for Sway and Wlroots based compositors
AddPackage wget # Network utility to retrieve files from the web
AddPackage wireless_tools # Tools allowing to manipulate the Wireless Extensions
AddPackage wireplumber # Session / policy manager implementation for PipeWire
AddPackage xdg-desktop-portal-gnome # Backend implementation for xdg-desktop-portal for the GNOME desktop environment
AddPackage xdg-utils # Command line tools that assist applications with a variety of desktop integration tasks
AddPackage xorg-server # Xorg X server
AddPackage xorg-xinit # X.Org initialisation program
AddPackage xorg-xwayland # run X clients under wayland
AddPackage xwayland-satellite # Xwayland outside your Wayland
AddPackage zram-generator # Systemd unit generator for zram devices
AddPackage zsh # A very advanced and programmable command interpreter (shell) for UNIX


# Thu Jun 12 01:42:20 PM CDT 2025 - Unknown foreign packages


AddPackage --foreign aconfmgr-git # A configuration manager for Arch Linux
AddPackage --foreign visual-studio-code-bin # Visual Studio Code (vscode): Editor for building and debugging modern web and cloud applications (official binary version)
AddPackage --foreign yay-bin # Yet another yogurt. Pacman wrapper and AUR helper written in go. Pre-compiled.
AddPackage --foreign yay-bin-debug # Detached debugging symbols for yay-bin


# Thu Jun 12 01:42:20 PM CDT 2025 - New / changed files


CopyFile /boot/info.md 755
CreateFile /etc/.pwd.lock 600 > /dev/null
CopyFile /etc/X11/xorg.conf.d/00-keyboard.conf
CopyFile /etc/default/grub
CreateLink /etc/fonts/conf.d/10-hinting-slight.conf /usr/share/fontconfig/conf.default/10-hinting-slight.conf
CreateLink /etc/fonts/conf.d/10-scale-bitmap-fonts.conf /usr/share/fontconfig/conf.default/10-scale-bitmap-fonts.conf
CreateLink /etc/fonts/conf.d/10-sub-pixel-rgb.conf /usr/share/fontconfig/conf.default/10-sub-pixel-rgb.conf
CreateLink /etc/fonts/conf.d/10-yes-antialias.conf /usr/share/fontconfig/conf.default/10-yes-antialias.conf
CreateLink /etc/fonts/conf.d/11-lcdfilter-default.conf /usr/share/fontconfig/conf.default/11-lcdfilter-default.conf
CreateLink /etc/fonts/conf.d/20-unhint-small-vera.conf /usr/share/fontconfig/conf.default/20-unhint-small-vera.conf
CreateLink /etc/fonts/conf.d/30-metric-aliases.conf /usr/share/fontconfig/conf.default/30-metric-aliases.conf
CreateLink /etc/fonts/conf.d/40-nonlatin.conf /usr/share/fontconfig/conf.default/40-nonlatin.conf
CreateLink /etc/fonts/conf.d/45-generic.conf /usr/share/fontconfig/conf.default/45-generic.conf
CreateLink /etc/fonts/conf.d/45-latin.conf /usr/share/fontconfig/conf.default/45-latin.conf
CreateLink /etc/fonts/conf.d/46-noto-mono.conf /usr/share/fontconfig/conf.default/46-noto-mono.conf
CreateLink /etc/fonts/conf.d/46-noto-sans.conf /usr/share/fontconfig/conf.default/46-noto-sans.conf
CreateLink /etc/fonts/conf.d/46-noto-serif.conf /usr/share/fontconfig/conf.default/46-noto-serif.conf
CreateLink /etc/fonts/conf.d/48-spacing.conf /usr/share/fontconfig/conf.default/48-spacing.conf
CreateLink /etc/fonts/conf.d/49-sansserif.conf /usr/share/fontconfig/conf.default/49-sansserif.conf
CreateLink /etc/fonts/conf.d/50-user.conf /usr/share/fontconfig/conf.default/50-user.conf
CreateLink /etc/fonts/conf.d/51-local.conf /usr/share/fontconfig/conf.default/51-local.conf
CreateLink /etc/fonts/conf.d/60-generic.conf /usr/share/fontconfig/conf.default/60-generic.conf
CreateLink /etc/fonts/conf.d/60-latin.conf /usr/share/fontconfig/conf.default/60-latin.conf
CreateLink /etc/fonts/conf.d/65-fonts-persian.conf /usr/share/fontconfig/conf.default/65-fonts-persian.conf
CreateLink /etc/fonts/conf.d/65-nonlatin.conf /usr/share/fontconfig/conf.default/65-nonlatin.conf
CreateLink /etc/fonts/conf.d/66-noto-mono.conf /usr/share/fontconfig/conf.default/66-noto-mono.conf
CreateLink /etc/fonts/conf.d/66-noto-sans.conf /usr/share/fontconfig/conf.default/66-noto-sans.conf
CreateLink /etc/fonts/conf.d/66-noto-serif.conf /usr/share/fontconfig/conf.default/66-noto-serif.conf
CreateLink /etc/fonts/conf.d/69-unifont.conf /usr/share/fontconfig/conf.default/69-unifont.conf
CreateLink /etc/fonts/conf.d/80-delicious.conf /usr/share/fontconfig/conf.default/80-delicious.conf
CreateLink /etc/fonts/conf.d/90-synthetic.conf /usr/share/fontconfig/conf.default/90-synthetic.conf
CopyFile /etc/group
CopyFile /etc/group-
CopyFile /etc/gshadow
CopyFile /etc/gshadow- 600
CopyFile /etc/ld.so.cache
CopyFile /etc/lightdm/lightdm.conf
CopyFile /etc/locale.gen
CopyFile /etc/machine-id 444
CopyFile /etc/mkinitcpio.conf
CopyFile /etc/mkinitcpio.d/linux.preset
CreateLink /etc/os-release ../usr/lib/os-release
CopyFile /etc/pacman.d/mirrorlist
CopyFile /etc/shells
CopyFile /etc/subgid
CreateFile /etc/subgid- > /dev/null
CopyFile /etc/subuid
CreateFile /etc/subuid- > /dev/null
CopyFile /etc/sudoers.d/00_brody 440
CreateLink /etc/systemd/system/dbus-org.freedesktop.nm-dispatcher.service /usr/lib/systemd/system/NetworkManager-dispatcher.service
CreateLink /etc/systemd/system/dbus-org.freedesktop.timesync1.service /usr/lib/systemd/system/systemd-timesyncd.service
CreateLink /etc/systemd/system/display-manager.service /usr/lib/systemd/system/lightdm.service
CreateLink /etc/systemd/system/getty.target.wants/getty@tty1.service /usr/lib/systemd/system/getty@.service
CreateLink /etc/systemd/system/multi-user.target.wants/NetworkManager.service /usr/lib/systemd/system/NetworkManager.service
CreateLink /etc/systemd/system/multi-user.target.wants/remote-fs.target /usr/lib/systemd/system/remote-fs.target
CreateLink /etc/systemd/system/network-online.target.wants/NetworkManager-wait-online.service /usr/lib/systemd/system/NetworkManager-wait-online.service
CreateLink /etc/systemd/system/sockets.target.wants/systemd-userdbd.socket /usr/lib/systemd/system/systemd-userdbd.socket
CreateLink /etc/systemd/system/sysinit.target.wants/systemd-timesyncd.service /usr/lib/systemd/system/systemd-timesyncd.service
CreateLink /etc/systemd/user/default.target.wants/xdg-user-dirs-update.service /usr/lib/systemd/user/xdg-user-dirs-update.service
CreateLink /etc/systemd/user/pipewire-session-manager.service /usr/lib/systemd/user/wireplumber.service
CreateLink /etc/systemd/user/pipewire.service.wants/wireplumber.service /usr/lib/systemd/user/wireplumber.service
CreateLink /etc/systemd/user/sockets.target.wants/p11-kit-server.socket /usr/lib/systemd/user/p11-kit-server.socket
CreateLink /etc/systemd/user/sockets.target.wants/pipewire-pulse.socket /usr/lib/systemd/user/pipewire-pulse.socket
CreateLink /etc/systemd/user/sockets.target.wants/pipewire.socket /usr/lib/systemd/user/pipewire.socket
CopyFile /etc/systemd/zram-generator.conf
CopyFile /etc/vconsole.conf


# Thu Jun 12 01:42:20 PM CDT 2025 - New file properties


SetFileProperty /boot/intel-ucode.img mode 755
SetFileProperty /usr/bin/groupmems group groups
SetFileProperty /usr/bin/groupmems mode 2750
SetFileProperty /var/log/journal group systemd-journal


# Fri Jun 20 05:14:00 PM CDT 2025 - Unknown packages


AddPackage appmenu-gtk-module # Application Menu GTK+ Module
AddPackage bc # An arbitrary precision calculator language
AddPackage bitwarden # A secure and free password manager for all of your devices
AddPackage brightnessctl # Lightweight brightness control tool
AddPackage clang # C language family frontend for LLVM
AddPackage dart-sass # Sass makes CSS fun again
AddPackage decoder # QR code scanner and generator
AddPackage gnome-text-editor # A simple text editor for the GNOME desktop
AddPackage gnome-themes-extra # Extra Themes for GNOME Applications
AddPackage gtk4-layer-shell # Library to create panels and other desktop components for Wayland
AddPackage hyprpicker # A wlroots-compatible Wayland color picker that does not suck
AddPackage imagemagick # An image viewing/manipulation program
AddPackage interception-tools # A minimal composable infrastructure on top of libudev and libevdev
AddPackage just # A handy way to save and run project-specific commands
AddPackage kitty # A modern, hackable, featureful, OpenGL-based terminal emulator
AddPackage mold # A Modern Linker
AddPackage ntfs-3g # NTFS filesystem driver and utilities
AddPackage polkit-kde-agent # Daemon providing a polkit authentication UI for KDE
AddPackage swww # A Solution to your Wayland Wallpaper Woes
AddPackage tesseract # An OCR program
AddPackage tesseract-data-eng # Tesseract OCR data (eng)
AddPackage tuned # Daemon that performs monitoring and adaptive configuration of devices in the system
AddPackage unzip # For extracting and viewing files in .zip archives
AddPackage uv # An extremely fast Python package installer and resolver written in Rust
AddPackage wayland-protocols # Specifications of extended Wayland protocols
AddPackage webkit2gtk # Web content engine for GTK
AddPackage webkit2gtk-4.1 # Web content engine for GTK
AddPackage wev # A tool for debugging wayland events on a Wayland window, analagous to the X11 tool xev
AddPackage wl-clipboard # Command-line copy/paste utilities for Wayland
AddPackage xdotool # Command-line X11 automation tool


# Fri Jun 20 05:14:01 PM CDT 2025 - Missing packages


RemovePackage lightdm-slick-greeter


# Fri Jun 20 05:14:01 PM CDT 2025 - Unknown foreign packages


AddPackage --foreign adwaita-qt5-git # A style to bend Qt5 applications to look like they belong into GNOME Shell, git version
AddPackage --foreign adwaita-qt6-git # A style to bend Qt6 applications to look like they belong into GNOME Shell, git version
AddPackage --foreign brillo # Control the brightness of backlight and keyboard LED devices
AddPackage --foreign cobang # A QR code scanner desktop app for Linux
AddPackage --foreign consolas-font # Consolas font
AddPackage --foreign vesktop # A standalone Electron-based Discord app with Vencord & improved Linux support


# Fri Jun 20 05:14:01 PM CDT 2025 - Extra files


RemoveFile /etc/machine-id
RemoveFile /etc/lightdm/lightdm.conf
RemoveFile /etc/lightdm


# Fri Jun 20 05:14:01 PM CDT 2025 - New / changed files


CopyFile /etc/interception/remap.yaml
CopyFile /etc/iproute2/rt_tables '' '' piavpn
CreateFile /etc/modprobe.d/tuned.conf > /dev/null
CreateLink /etc/systemd/system/multi-user.target.wants/piavpn.service /etc/systemd/system/piavpn.service
CreateLink /etc/systemd/system/multi-user.target.wants/tuned.service /usr/lib/systemd/system/tuned.service
CreateLink /etc/systemd/system/multi-user.target.wants/udevmon.service /usr/lib/systemd/system/udevmon.service
CopyFile /etc/systemd/system/piavpn.service
CreateLink /etc/systemd/user/sockets.target.wants/gnome-keyring-daemon.socket /usr/lib/systemd/user/gnome-keyring-daemon.socket

# Fri Jun 20 05:14:06 PM CDT 2025 - Extra file properties


SetFileProperty /etc/machine-id mode ''


# Fri Jun 20 05:23:57 PM CDT 2025 - New / changed files


CopyFile /etc/tuned/active_profile
CreateFile /etc/tuned/post_loaded_profile > /dev/null
CopyFile /etc/tuned/profile_mode


# Fri Jun 20 05:23:57 PM CDT 2025 - New file properties


SetFileProperty /etc/iproute2 group piavpn
