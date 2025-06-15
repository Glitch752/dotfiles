from pathlib import Path
from typing import Optional
import jinja2
import toml
from dataclasses import dataclass

class RenderConfig:
    """
    Represents a render configuration for a dotfile.
    """
    source: Path
    destination: Path
    user: Optional[str]
    copy: bool
    
    def __init__(self, source: str | Path, destination: str | Path, user: Optional[str] = None, copy: bool = False):
        self.source = Path(source)
        dest = Path(destination).expanduser()
        if not dest.is_absolute():
            raise ValueError(f"Render destination path '{destination}' must be absolute.")
        self.destination = dest
        self.user = user
        self.copy = copy
    
    def source_relative(self, diff: Path) -> 'RenderConfig':
        self.source = diff / self.source
        return self

@dataclass
class Configuration:
    """
    Represents a configuration, which is a set of dotfiles and variables.
    """
    variables: dict[str, str]
    render: list[RenderConfig]
    subconfigs: list[str]

@dataclass
class Profile:
    """
    Represents a profile, which is a set of configurations.
    """
    name: str
    configurations: list[str]

@dataclass
class ResolvedConfig:
    """
    Represents a resolved configuration, containing the paths that need to be rendered and the variables to use.
    """
    render: list[RenderConfig]
    variables: dict[str, str]

class DotsConfig:
    """
    Stores configuration (from config.toml) for the dot manager and handles importing the tree of config files.
    """
    
    config_path: Path
    configurations: dict[str, Configuration]
    profiles: dict[str, Profile]
    
    def __init__(self, config_path: Path):
        """
        Initializes the DotsConfig with the path to the configuration file.
        
        :param config_path: Path to the configuration file (e.g., config.toml).
        """
        self.config_path = config_path
        self.configurations = {}
        self.profiles = {}
        self.load()
    
    def load(self):
        try:
            with open(self.config_path, 'r') as f:
                config_data = toml.load(f)
            
            # Load configurations
            for name, config in config_data.get('configurations', {}).items():
                self.configurations[name] = Configuration(
                    variables=config.get('variables', {}),
                    render=[RenderConfig(**r) for r in config.get('render', [])],
                    subconfigs=config.get('subconfigs', [])
                )
            
            # Load profiles
            for name, profile in config_data.get('profiles', {}).items():
                self.profiles[name] = Profile(
                    name=name,
                    configurations=profile.get('configurations', [])
                )
            
            # Load imports and merge configurations. This means imports can override existing configurations.
            for import_path in config_data.get('imports', []):
                import_config = DotsConfig(self.config_path.parent / import_path).with_paths_relative_to(self.config_path)
                self.configurations.update(import_config.configurations)
                self.profiles.update(import_config.profiles)
        except toml.TomlDecodeError as e:
            raise ValueError(f"Error decoding TOML configuration file: {self.config_path}\n{e}")
    
    def with_paths_relative_to(self, base_path: Path):
        """
        Returns a new DotsConfig with paths relative to the given base path.
        
        :param base_path: The base path to make paths relative to.
        :return: A new DotsConfig instance with updated paths.
        """
        new_config = DotsConfig(self.config_path)
        diff = self.config_path.parent.relative_to(base_path.parent)
        for name, config in self.configurations.items():
            new_config.configurations[name] = Configuration(
                variables={k: v for k, v in config.variables.items()},
                render=[r.source_relative(diff) for r in config.render],
                subconfigs=config.subconfigs
            )
        return new_config
    
    def resolve_profile(self, profile_name: str) -> ResolvedConfig:
        """
        Resolves a profile and returns the paths that need to be rendered and the variables to use.
        
        :param profile_name: The name of the profile to resolve.
        :return: A ResolvedConfig containing the render paths and variables.
        """
        if profile_name not in self.profiles:
            raise ValueError(f"Profile '{profile_name}' not found in configuration.")
        
        profile = self.profiles[profile_name]
        config = ResolvedConfig(render=[], variables={})
        for config_name in profile.configurations:
            if config_name not in self.configurations:
                raise ValueError(f"Configuration '{config_name}' not found in configurations.")
            subconfig = self.configurations[config_name]
            self.configure_resolved_config(subconfig, config)
        
        return config

    def configure_resolved_config(self, config: Configuration, resolved_config: ResolvedConfig):
        """
        Configures a resolved configuration with the given configuration.
        
        :param config: The Configuration to use.
        :param resolved_config: The ResolvedConfig to update.
        """
        resolved_config.render.extend(config.render)
        resolved_config.variables.update(config.variables)
        
        for subconfig_name in config.subconfigs:
            if subconfig_name not in self.configurations:
                raise ValueError(f"Subconfiguration '{subconfig_name}' not found in configurations.")
            subconfig = self.configurations[subconfig_name]
            self.configure_resolved_config(subconfig, resolved_config)