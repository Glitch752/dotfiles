# A simple logging implementation that makes pretty logs.
# Supports nested levels of indentation.

import logging
from contextlib import _GeneratorContextManager, contextmanager

ANSI_GRAY = "\033[90m"
ANSI_BLUE = "\033[94m"
ANSI_YELLOW = "\033[93m"
ANSI_RED = "\033[91m"
ANSI_MAGENTA = "\033[95m"
ANSI_RESET = "\033[0m"

class PrettyFormatter(logging.Formatter):
    """A custom formatter that formats log messages with colors and styles."""
    
    def get_indent(self) -> str:
        raise NotImplementedError("Must override get_indent method")
    
    def format(self, record):
        levelname = record.levelname
        indent = self.get_indent()
        message = record.getMessage()
        if levelname == "DEBUG":
            return f"{indent}{ANSI_GRAY}{message}{ANSI_RESET}"
        elif levelname == "INFO":
            return f"{indent}{message}{ANSI_RESET}"
        elif levelname == "WARNING":
            return f"{indent}{ANSI_YELLOW}{message}{ANSI_RESET}"
        elif levelname == "ERROR":
            return f"{indent}{ANSI_RED}{message}{ANSI_RESET}"
        elif levelname == "CRITICAL":
            return f"{indent}{ANSI_MAGENTA}{message}{ANSI_RESET}"
        else:
            return message

@contextmanager
def LoggingContext(logger: "PrettyLogger", indent_level: int):
    """A context manager for logging with indentation."""
    try:
        logger.indent_level += 1
        yield
    finally:
        logger.indent_level -= 1
        if logger.indent_level < 0:
            logger.indent_level = 0

class PrettyLogger:
    """A simple logger that uses PrettyFormatter for pretty logging."""
    
    logger: logging.Logger
    indent_level: int
    
    def __init__(self, name: str = "PrettyLogger", level: int = logging.DEBUG):
        self.logger = logging.getLogger(name)
        self.logger.setLevel(level)
        
        # Create console handler
        ch = logging.StreamHandler()
        ch.setLevel(level)
        
        # Set the custom formatter
        formatter = PrettyFormatter()
        formatter.get_indent = self.get_indent
        ch.setFormatter(formatter)
        
        # Add the handler to the logger
        self.logger.addHandler(ch)
        self.logger.propagate = False
        
        self.indent_level = 0
    
    def get_indent(self) -> str:
        return f"{ANSI_GRAY}| {ANSI_RESET}" * (self.indent_level)
    
    def debug(self, msg, *args, **kwargs) -> _GeneratorContextManager:
        self.logger.debug(msg, *args, **kwargs)
        return LoggingContext(self, self.indent_level)
    def info(self, msg, *args, **kwargs) -> _GeneratorContextManager:
        self.logger.info(msg, *args, **kwargs)
        return LoggingContext(self, self.indent_level)
    def warn(self, msg, *args, **kwargs) -> _GeneratorContextManager:
        self.logger.warning(msg, *args, **kwargs)
        return LoggingContext(self, self.indent_level)
    def error(self, msg, *args, **kwargs) -> _GeneratorContextManager:
        self.logger.error(msg, *args, **kwargs)
        return LoggingContext(self, self.indent_level)
    def critical(self, msg, *args, **kwargs) -> _GeneratorContextManager:
        self.logger.critical(msg, *args, **kwargs)
        return LoggingContext(self, self.indent_level)