"""Unit tests for the main module."""

from click.testing import CliRunner

from src.main import cli


def test_hello_command() -> None:
    """Test the hello command."""
    runner = CliRunner()
    result = runner.invoke(cli, ["hello"])
    assert result.exit_code == 0
    assert "Hello, World!" in result.output


def test_hello_command_with_name() -> None:
    """Test the hello command with a custom name."""
    runner = CliRunner()
    result = runner.invoke(cli, ["hello", "--name", "Test"])
    assert result.exit_code == 0
    assert "Hello, Test!" in result.output


def test_add_command() -> None:
    """Test the add command."""
    runner = CliRunner()
    result = runner.invoke(cli, ["add", "2", "3"])
    assert result.exit_code == 0
    assert "2.0 + 3.0 = 5.0" in result.output


def test_debug_mode() -> None:
    """Test debug mode."""
    runner = CliRunner()
    with runner.isolation():
        result = runner.invoke(cli, ["--debug", "hello"])
        assert result.exit_code == 0
        assert "Hello, World!" in result.output