alias ll = ls -l
alias la = ls -la

# Yay aliases because I'm an inexperienced Arch user
alias inst = yay -Sy --noconfirm
alias update = yay -Syu --noconfirm
alias search = yay -Ss
alias remove = yay -Rns
alias clean = yay $"-Rns (pacman -Qdtq)"

alias view = qimgv
alias show = qimgv

alias open = xdg-open

# sure. just.. autocorrect for any profanity
alias shit = fuck

alias neofetch = fastfetch

{% if device == "laptop" %}
def batt [] { upower -i /org/freedesktop/UPower/devices/battery_BAT1 | grep "percentage:" | awk '{print $2}' }
{% endif %}