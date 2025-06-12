
import json
import os
from .config import DotsConfig, ResolvedConfig
from pathlib import Path
from jinja2 import Environment, FileSystemLoader
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

class DotfileManager:
    dots_dir: Path
    config: ResolvedConfig
    
    def __init__(self, dots_dir: Path, profile: str):
        self.dots_dir = dots_dir
        
        if not self.dots_dir.exists():
            # If the dots directory does not exist, create it with the default content in ./default_dots
            self.create_default_dots()
        
        self.config = DotsConfig(self.dots_dir / "config.toml").resolve_profile(profile)
        
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
    
    def create_default_dots(self):
        default_dots = Path(__file__).parent / "default_dots"
        if default_dots.exists():
            print(f"Creating default dots directory at {self.dots_dir}")
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
        for to_render in self.config.render:
            output_path = to_render.destination
            
            rendered = self.env.get_template(to_render.source.as_posix()).render(self.config.variables)

            # Write to temp file and rename atomically
            tmp_path = output_path.with_suffix(".tmp")
            with open(tmp_path, "w") as f:
                f.write(rendered)
            os.replace(tmp_path, output_path)
            print(f"  Rendered and updated: {output_path}")

    def watch(self):
        class ReloadHandler(FileSystemEventHandler):
            def __init__(self, manager):
                self.manager = manager

            def on_modified(self, event):
                print("Change detected. Re-rendering templates...")
                self.manager.variables = self.manager.load_variables()
                self.manager.render_templates()

        observer = Observer()
        observer.schedule(ReloadHandler(self), path=str(self.dots_dir), recursive=True)
        observer.start()
        
        print("Watching for changes. Press Ctrl+C to exit.")
        try:
            while True:
                pass
        except KeyboardInterrupt:
            observer.stop()
        observer.join()