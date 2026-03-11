import argparse
from pathlib import Path

from .manager import DotfileManager

# TODO: Better system for initializing the config

def main():
    parser = argparse.ArgumentParser(description="Dotfile Manager with templating and hot-reload.")
    parser.add_argument("command", choices=["apply", "watch", "save-packages", "load-packages"], help="Action to perform")
    parser.add_argument("--dir", default="~/dotfiles", type=Path, help="Directory containing dotfiles (default: ~/dotfiles)")
    parser.add_argument("--profile", default="default", type=str, help="Profile to use for rendering (default: 'default')")
    
    args = parser.parse_args()
    dots_dir = args.dir.expanduser().resolve()
    profile = args.profile

    manager = DotfileManager(dots_dir, profile)
    if args.command == "apply":
        manager.render_templates()
    elif args.command == "save-packages":
        manager.save_packages()
    elif args.command == "load-packages":
        manager.load_packages()
    elif args.command == "watch":
        with manager.log.info("Rendering templates once before watching"):
            manager.render_templates()
        manager.watch()

if __name__ == "__main__":
    main()