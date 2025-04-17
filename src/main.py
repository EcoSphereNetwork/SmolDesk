"""Main entry point for the application."""

import logging
import os
from typing import Optional

import click
from dotenv import load_dotenv
from rich.console import Console
from rich.logging import RichHandler

# Load environment variables
load_dotenv()

# Configure logging
logging.basicConfig(
    level=os.getenv("LOG_LEVEL", "INFO"),
    format="%(message)s",
    datefmt="[%X]",
    handlers=[RichHandler(rich_tracebacks=True)],
)

log = logging.getLogger("rich")
console = Console()


@click.group()
@click.option("--debug/--no-debug", default=False, help="Enable debug mode")
def cli(debug: bool) -> None:
    """CLI entry point for the application."""
    if debug:
        log.setLevel(logging.DEBUG)
        log.debug("Debug mode enabled")


@cli.command()
@click.option("--name", "-n", help="Name to greet", default="World")
def hello(name: str) -> None:
    """Greet the user."""
    console.print(f"[green]Hello, {name}![/green]")


@cli.command()
@click.argument("x", type=float)
@click.argument("y", type=float)
def add(x: float, y: float) -> None:
    """Add two numbers."""
    result = x + y
    console.print(f"[blue]{x} + {y} = {result}[/blue]")


def main() -> None:
    """Main entry point."""
    try:
        cli()
    except Exception as e:
        log.exception(f"An error occurred: {e}")
        raise


if __name__ == "__main__":
    main()