import os
import subprocess
import toml
import time

from .logging import ANSI_GREEN, ANSI_RED, ANSI_RESET, PrettyLogger
from .config import DotsConfig, ResolvedConfig
from pathlib import Path
from jinja2 import Environment, FileSystemLoader
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

import tempfile
import shutil
import pwd

def expand_user_to(user: str, path: str) -> str:
    import pwd
    if path.startswith("~"):
        path = f"{pwd.getpwnam(user).pw_dir}{path[1:]}"
    return path

# Write content to a temporary file, then move it atomically to the destination.
def write_atomic(filepath: Path, content: str):
    dirpath = filepath.parent
    dirpath.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile('w', dir=dirpath, delete=False) as tf:
        tf.write(content)
        tempname = tf.name

    os.replace(tempname, filepath)

def walk_files(dir: Path) -> list[str]:
    res = []
    for (dirpath, _, filenames) in os.walk(dir):
        for filename in filenames:
            res.append(os.path.join(dirpath, filename))
    return res

def copy_atomic(src: Path, dst: Path):
    dst.parent.mkdir(parents=True, exist_ok=True)
    
    # If the source is a directory, copy each file individually (and don't care about being atomic because whatever)
    if src.is_dir():
        shutil.copytree(src, dst, dirs_exist_ok=True)
        return
    
    with tempfile.NamedTemporaryFile('wb', dir=dst.parent, delete=False) as tf:
        with open(src, 'rb') as sf:
            shutil.copyfileobj(sf, tf)
        tempname = tf.name

    os.replace(tempname, dst)

class DotfileManager:
    dots_dir: Path
    config: ResolvedConfig
    profile: str
    log: PrettyLogger
    
    def __init__(self, dots_dir: Path, profile: str):
        self.dots_dir = dots_dir
        self.log = PrettyLogger()
        
        if not self.dots_dir.exists():
            # If the dots directory does not exist, create it with the default content in ./default_dots
            self.create_default_dots()
        
        self.profile = profile
        try:
            self.reload_config()
        except RuntimeError as e:
            self.log.error(f"Failed to load configuration: {e}")
            os._exit(1)
        
        self.env = Environment(
            loader=FileSystemLoader(self.dots_dir),
            autoescape=True,
            
            block_start_string="{%",
            block_end_string="%}",
            variable_start_string="{{",
            variable_end_string="}}",
            comment_start_string="{#",
            comment_end_string="#}",
            keep_trailing_newline=True
        )
        
    def reload_config(self):
        try:
            self.config = DotsConfig(self.dots_dir / "config.toml").resolve_profile(self.profile)
        except Exception as e:
            self.log.error(f"Error loading configuration: {e}")
            raise RuntimeError(f"Failed to load configuration for profile '{self.profile}'") from e
    
    def create_default_dots(self):
        default_dots = Path(__file__).parent / "default_dots"
        if default_dots.exists():
            self.log.info(f"Creating default dots directory at {self.dots_dir}")
            self.dots_dir.mkdir(parents=True, exist_ok=True)
            # recursively copy the contents of default_dots to dots_dir
            self.copy_folder(default_dots, self.dots_dir)
            
    def copy_folder(self, src, dst):
        if not dst.exists():
            dst.mkdir(parents=True, exist_ok=True)
        for item in src.iterdir():
            if item.is_dir():
                self.copy_folder(item, dst / item.name)
            else:
                (dst / item.name).write_bytes(item.read_bytes())

    def render_templates(self):
        user = os.getlogin()
                
        with self.log.info("Rendering templates"):
            for to_render in self.config.render:
                username = user if to_render.as_user == None else to_render.as_user
                
                output_path = Path(expand_user_to(username, str(to_render.destination)))
                if not output_path.is_absolute():
                    raise ValueError(f"Render destination path '{output_path}' must be absolute.")
                
                with self.log.info(f"Rendering {to_render.source}"):
                    pw = pwd.getpwnam(username)
                    os.seteuid(os.getuid()) # Allow setegid
                    os.setegid(pw.pw_gid)
                    os.seteuid(pw.pw_uid)
                        
                    if to_render.action == "copy":
                        source = self.dots_dir / to_render.source
                        if not source.exists():
                            self.log.error(f"Source file does not exist: {source}")
                            continue
                        copy_atomic(source, output_path)
                        self.log.info(f"Copied: {output_path}")
                    elif to_render.action == "link":
                        source = self.dots_dir / to_render.source
                        if not source.exists():
                            self.log.error(f"Source file does not exist: {source}")
                            continue
                        try:
                            if output_path.exists():
                                output_path.unlink()
                            os.symlink(source, output_path)
                            self.log.info(f"Linked: {output_path} -> {source}")
                        except OSError:
                            self.log.error(f"Link already exists: {output_path}. Skipping.")
                            continue
                    else:
                        try:
                            rendered = self.env.get_template(to_render.source.as_posix()).render(self.config.variables)
                        except Exception as e:
                            self.log.error(f"Error rendering template {to_render.source}: {e} ({e.__cause__})")
                            continue

                        # Write to temp file and rename atomically
                        write_atomic(output_path, rendered)
                    
                    if to_render.permissions:
                        try:
                            os.chmod(output_path, int(to_render.permissions, 8))
                            self.log.info(f"Set permissions for {output_path} to {to_render.permissions}")
                        except Exception as e:
                            self.log.error(f"Error setting permissions for {output_path}: {e} ({e.__cause__})")        
    

    def get_packages(self):
        # system packages (pacman)
        try:
            pacman_pkgs = subprocess.check_output([
                "pacman", "-Qqen"
            ], text=True).splitlines()
        except Exception as e:
            self.log.error(f"Failed to get system packages: {e}")
            pacman_pkgs = []

        # AUR packages (foreign)
        try:
            aur_pkgs = subprocess.check_output([
                "pacman", "-Qqem"
            ], text=True).splitlines()
        except Exception as e:
            self.log.error(f"Failed to get AUR packages: {e}")
            aur_pkgs = []

        return {
            "system": sorted(pacman_pkgs),
            "aur": sorted(aur_pkgs)
        }
    
    def get_saved_packages(self):
        pkgfile = self.dots_dir / "packages.toml"
        if not pkgfile.exists():
            return None

        try:
            with open(pkgfile, "r") as f:
                return toml.load(f)
        except Exception as e:
            self.log.error(f"Failed to load packages.toml: {e}")
            return None

    def save_packages(self):
        old_packages = self.get_saved_packages()
        packages = self.get_packages()
        
        # Show diffs
        if old_packages:
            old_system = set(old_packages.get("system", []))
            old_aur = set(old_packages.get("aur", []))
            new_system = set(packages.get("system", []))
            new_aur = set(packages.get("aur", []))

            added_system = new_system - old_system
            removed_system = old_system - new_system
            added_aur = new_aur - old_aur
            removed_aur = old_aur - new_aur

            if added_system or removed_system:
                with self.log.info("System package changes:"):
                    for pkg in added_system:
                        self.log.info(f"{ANSI_GREEN}+ {pkg}{ANSI_RESET}")
                    for pkg in removed_system:
                        self.log.info(f"{ANSI_RED}- {pkg}{ANSI_RESET}")
            if added_aur or removed_aur:
                with self.log.info("AUR package changes:"):
                    for pkg in added_aur:
                        self.log.info(f"{ANSI_GREEN}+ {pkg}{ANSI_RESET}")
                    for pkg in removed_aur:
                        self.log.info(f"{ANSI_RED}- {pkg}{ANSI_RESET}")

            if not (added_system or removed_system or added_aur or removed_aur):
                self.log.info("No package changes detected.")

        pkgfile = self.dots_dir / "packages.toml"
        try:
            with open(pkgfile, "w") as f:
                toml.dump(packages, f)
            # Allow all users to change this file
            os.chmod(pkgfile, 0o666)
            self.log.info(f"Saved packages to {pkgfile}")
        except Exception as e:
            self.log.error(f"Failed to save packages.toml: {e}")

    def load_packages(self):
        packages = self.get_saved_packages()
        if packages == None:
            self.log.error(f"No packages.toml found")
            return

        system_pkgs = packages.get("system", [])
        aur_pkgs = packages.get("aur", [])

        installed_pkgs = self.get_packages()
        
        # install system packages that aren't in the config
        for pkg in system_pkgs:
            if pkg not in installed_pkgs["system"]:
                self.log.info(f"Installing system package: {pkg}")
                try:
                    subprocess.run([
                        "sudo", "pacman", "-S", "--needed", "--noconfirm", pkg
                    ], check=True)
                except Exception as e:
                    self.log.error(f"Failed to install system package {pkg}: {e}")
        
        # install AUR packages that aren't in the config
        for pkg in aur_pkgs:
            if pkg not in installed_pkgs["aur"]:
                self.log.info(f"Installing AUR package: {pkg}")
                try:
                    subprocess.run([
                        "yay", "-S", "--needed", "--noconfirm", pkg
                    ], check=True)
                except Exception as e:
                    self.log.error(f"Failed to install AUR package {pkg}: {e}")
        
        # List (but don't remove) installed packages that aren't in config
        for pkg in installed_pkgs["system"]:
            if pkg not in system_pkgs:
                self.log.info(f"Installed system package not in config: {pkg}")
        for pkg in installed_pkgs["aur"]:
            if pkg not in aur_pkgs:
                self.log.info(f"Installed AUR package not in config: {pkg}")

    def watch(self):
        class ReloadHandler(FileSystemEventHandler):
            manager: "DotfileManager"
            
            def __init__(self, manager):
                self.manager = manager

            def on_modified(self, event):
                # Avoid double handling of events by avoiding directory reloads
                if event.is_directory:
                    return
                
                with self.manager.log.info("Change detected. Re-rendering templates..."):
                    try:
                        self.manager.reload_config()
                    except RuntimeError as e:
                        self.manager.log.error(f"Failed to reload configuration: {e}")
                        return
                    self.manager.render_templates()

        observer = Observer()
        observer.schedule(ReloadHandler(self), path=str(self.dots_dir), recursive=True)
        observer.start()
        
        self.log.info("Watching for changes. Press Ctrl+C to exit.")
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            observer.stop()
        observer.join()