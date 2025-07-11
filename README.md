# dotfiles



https://github.com/user-attachments/assets/ffd2b299-e94b-4478-bac2-42e37e773990



My personal Arch Linux configuration, setup using [aconfmgr](https://github.com/CyberShadow/aconfmgr) and a custom lightweight templating engine written in Python (not because I could make something better than the alternatives, but because I wanted to).

I also got a little distracted and wrote a low-level kernel input intercepting tool to make niri work with the LWin+LAlt+LMB gesture I'm used to. It's _insanely_ hacky, but it works surprisingly well.

## Features
- Custom Niri configuration
- Nvidia support
- Relatively consistent system-wide theming
- Wallpaper manager script using [swww](https://github.com/LGFae/swww) that changes and animates wallpaper transitions
- Custom layer-shell "bar" (though it goes around the whole screen) with launcher, notifications, and more
- Consistent system-wide themeing with custom themes for GTK 3, GTK 4, VSCode, Discord, Firefox, and more

## Images

![Screenshot](./images/screenshot1.png)
![Screenshot](./images/screenshot2.png)

## TODO
- [ ] Add more documentation
- [ ] Organize aconfmgr files
- [ ] Set up dotmgr to build and install custom theme instead of using shell script
- [ ] Support multiple monitors
- [ ] Launcher
  - [ ] Proper module icons
- [ ] More bar widgets
  - [ ] Audio input/output
  - [ ] System resource utilization
  - [ ] Mpris (audio players)
  - [ ] Wireless connections
  - [ ] System tray
- [ ] More popup menus
  - [ ] Power options
  - [ ] Power profiles under battery
  - [ ] Calendar
  - [ ] Clipboard history
- [ ] On-screen display
  - [ ] Volume
  - [ ] Brightness
- [ ] Custom lockscreen?
- [ ] Somehow implement system-wide theme customization?

### TODO for dotmgr
- [ ] Allow regex for file matching and destination
- [ ] Only re-render changed files
- [ ] Add post-render and post-first-install hook commands
- [ ] Track generated files and delete if no longer managed
- [ ] Better persistent per-machine configuration
- [ ] Change to something other than TOML since I'm unsatisfied with it (KDL?)
- [ ] Implement a more generic solution for the aconfmgr generated ignore list
  - [ ] ...or replace aconfmgr with dotmgr by adding a few more features for package management
- [ ] Allow using variables in paths?
