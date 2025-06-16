# Set personal aliases, overriding those provided by Oh My Zsh libs,
# plugins, and themes. Aliases can be placed here, though Oh My Zsh
# users are encouraged to define aliases within a top-level file in
# the $ZSH_CUSTOM folder, with .zsh extension. Examples:
# - $ZSH_CUSTOM/aliases.zsh
# - $ZSH_CUSTOM/macos.zsh
# For a full list of active aliases, run `alias`.

alias ll="ls -l"
alias la="ls -la"

# Yay aliases because I'm an inexperienced Arch user
alias inst="yay -S --noconfirm"
alias update="yay -Syu --noconfirm"
alias search="yay -Ss"
alias remove="yay -Rns"
alias clean="yay -Rns $(pacman -Qdtq)"

alias shit="fuck"

alias neofetch="fastfetch"

alias saveconf="aconfmgr save -c ~/dotfiles/aconfmgr"
alias applyconf="aconfmgr apply -c ~/dotfiles/aconfmgr"

{% if device == "laptop" %}
alias batt="upower -i /org/freedesktop/UPower/devices/battery_BAT1 | grep \"percentage:\" | awk '{print \$2}'"
{% endif %}