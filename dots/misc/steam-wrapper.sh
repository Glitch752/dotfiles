#!/usr/bin/env bash
# Launch Steam with `-system-composer` to fix issues with it showing a solid black screen.
# See https://github.com/ValveSoftware/steam-for-linux/issues/10543 and 
# https://github.com/Supreeeme/xwayland-satellite/issues/150
exec /usr/bin/steam -system-composer "$@"