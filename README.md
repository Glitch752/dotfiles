# dotfiles

My personal Arch Linux configuration, setup using [aconfmgr](https://github.com/CyberShadow/aconfmgr) and a custom lightweight templating engine written in Python (not because I could make something better than the alternatives, but because I wanted to).

I also got a little distracted and wrote a low-level kernel input intercepting tool to make niri work with the LWin+LAlt+LMB gesture I'm used to. It's _insanely_ hacky, but it works surprisingly well.

## TODO
- [ ] Add more documentation
- [ ] Organize aconfmgr files

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