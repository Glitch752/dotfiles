Hi, future me!

Files should be under /EFI and further organized by venddor or OS because that's what the UEFI spec mandates.

The standard Linux method is installing kernel files directly to /boot, and grub-mkconfig probes for them there automatically. Grub settings can be edited under /etc/grub.d/. See https://wiki.archlinux.org/title/GRUB

