#!/usr/bin/env python3
"""
Standalone repository reorganization script.
This version can be run without Poetry and has minimal dependencies.
"""

import argparse
import json
import logging
import os
import re
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

try:
    import toml
except ImportError:
    print("Installing required packages...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "--user", "toml"])
    import toml

try:
    import rich
    from rich.console import Console
    from rich.logging import RichHandler
    from rich.progress import Progress, SpinnerColumn, TextColumn
    from rich.prompt import Confirm
except ImportError:
    print("Installing required packages...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "--user", "rich"])
    import rich
    from rich.console import Console
    from rich.logging import RichHandler
    from rich.progress import Progress, SpinnerColumn, TextColumn
    from rich.prompt import Confirm

try:
    import git
except ImportError:
    print("Installing required packages...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "--user", "gitpython"])
    import git

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(message)s",
    datefmt="[%X]",
    handlers=[RichHandler(rich_tracebacks=True)],
)

log = logging.getLogger("rich")
console = Console()

# Template structure
TEMPLATE_STRUCTURE = {
    "required_dirs": {
        "src": "Source code directory",
        "tests": "Test files",
        "docs": "Documentation",
        ".github/workflows": "GitHub Actions",
        "scripts": "Utility scripts",
    },
    "required_files": {
        "README.md": "Project documentation",
        "LICENSE": "Project license",
        ".gitignore": "Git ignore rules",
        "pyproject.toml": "Project configuration",
        "Makefile": "Build automation",
    },
    "optional_dirs": {
        "docs/api": "API documentation",
        "docs/guides": "User guides",
        "docs/development": "Development documentation",
        "tests/unit": "Unit tests",
        "tests/integration": "Integration tests",
        "tests/e2e": "End-to-end tests",
    },
}

class RepoReorganizer:
    """Simple repository reorganizer."""

    def __init__(self, repo_path: str, template_path: str, branch_name: Optional[str] = None):
        """Initialize reorganizer."""
        self.repo_path = Path(repo_path).resolve()
        self.template_path = Path(template_path).resolve()
        self.branch_name = branch_name or f"refactor/reorganize-{datetime.now():%Y%m%d-%H%M%S}"
        self.repo = git.Repo(repo_path)
        self.changes: List[Dict[str, str]] = []

    def analyze_structure(self) -> Tuple[Set[str], Set[str], Set[str]]:
        """Analyze repository structure."""
        with Progress(SpinnerColumn(), TextColumn("[progress.description]{task.description}")) as progress:
            task = progress.add_task("Analyzing repository structure...", total=None)

            # Get current structure
            current_dirs = {
                str(p.relative_to(self.repo_path))
                for p in self.repo_path.rglob("*")
                if p.is_dir() and ".git" not in str(p)
            }
            current_files = {
                str(p.relative_to(self.repo_path))
                for p in self.repo_path.rglob("*")
                if p.is_file() and ".git" not in str(p)
            }

            # Required structure
            required_dirs = set(TEMPLATE_STRUCTURE["required_dirs"].keys())
            required_files = set(TEMPLATE_STRUCTURE["required_files"].keys())

            # Find differences
            missing_dirs = required_dirs - current_dirs
            missing_files = required_files - current_files
            extra_files = current_files - required_files

            progress.update(task, completed=True)

        return missing_dirs, missing_files, extra_files

    def create_plan(self, missing_dirs: Set[str], missing_files: Set[str], extra_files: Set[str]) -> List[Dict[str, str]]:
        """Create reorganization plan."""
        plan = []

        # Plan directory creation
        for dir_path in missing_dirs:
            plan.append({
                "action": "create_dir",
                "path": dir_path,
                "description": TEMPLATE_STRUCTURE["required_dirs"][dir_path],
            })

        # Plan file copying
        for file_path in missing_files:
            plan.append({
                "action": "copy_file",
                "path": file_path,
                "source": str(self.template_path / file_path),
                "description": TEMPLATE_STRUCTURE["required_files"][file_path],
            })

        # Plan file moves
        for file_path in extra_files:
            # Determine appropriate destination
            if file_path.endswith(".py") and "test_" in file_path:
                dest = f"tests/unit/{Path(file_path).name}"
                plan.append({
                    "action": "move_file",
                    "path": file_path,
                    "destination": dest,
                    "description": "Move to tests directory",
                })
            elif file_path.endswith(".py"):
                dest = f"src/{Path(file_path).name}"
                plan.append({
                    "action": "move_file",
                    "path": file_path,
                    "destination": dest,
                    "description": "Move to source directory",
                })
            elif file_path.endswith((".md", ".rst")) and file_path.lower() != "readme.md":
                dest = f"docs/{Path(file_path).name}"
                plan.append({
                    "action": "move_file",
                    "path": file_path,
                    "destination": dest,
                    "description": "Move to documentation directory",
                })

        return plan

    def execute_plan(self, plan: List[Dict[str, str]], dry_run: bool = False) -> None:
        """Execute reorganization plan."""
        with Progress(SpinnerColumn(), TextColumn("[progress.description]{task.description}")) as progress:
            task = progress.add_task("Executing reorganization plan...", total=len(plan))

            for change in plan:
                action = change["action"]
                path = change["path"]

                try:
                    if dry_run:
                        log.info(f"Would {action}: {path}")
                        continue

                    if action == "create_dir":
                        os.makedirs(self.repo_path / path, exist_ok=True)
                    elif action == "copy_file":
                        shutil.copy2(change["source"], self.repo_path / path)
                    elif action == "move_file":
                        dest = self.repo_path / change["destination"]
                        os.makedirs(dest.parent, exist_ok=True)
                        shutil.move(self.repo_path / path, dest)

                    self.changes.append(change)
                except Exception as e:
                    log.error(f"Error executing {action} for {path}: {e}")

                progress.advance(task)

    def commit_changes(self) -> None:
        """Commit changes to a new branch."""
        if not self.changes:
            log.info("No changes to commit")
            return

        try:
            # Create and checkout new branch
            current = self.repo.active_branch
            new_branch = self.repo.create_head(self.branch_name)
            new_branch.checkout()

            # Stage all changes
            self.repo.git.add(A=True)

            # Create commit message
            message = "refactor: reorganize repository structure\n\n"
            message += "Reorganize repository to match template structure:\n\n"
            for change in self.changes:
                message += f"* {change['action']}: {change['path']}\n"

            # Commit changes
            self.repo.index.commit(message)

            log.info(f"Changes committed to branch: {self.branch_name}")
            log.info(f"Previous branch was: {current.name}")

        except Exception as e:
            log.error(f"Error committing changes: {e}")
            # Try to restore previous branch
            try:
                current.checkout()
            except:
                pass

    def generate_report(self) -> str:
        """Generate report of changes made."""
        report = "# Repository Reorganization Report\n\n"
        report += f"Date: {datetime.now():%Y-%m-%d %H:%M:%S}\n\n"

        if self.changes:
            report += "## Changes Made\n\n"
            for change in self.changes:
                report += f"* {change['action']}: {change['path']}\n"
                if "description" in change:
                    report += f"  - {change['description']}\n"
        else:
            report += "No changes were made.\n"

        return report


def main() -> None:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Reorganize a repository to match the template structure"
    )
    parser.add_argument("repo_path", help="Path to the repository to reorganize")
    parser.add_argument(
        "--template-path",
        default=os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
        help="Path to the template repository",
    )
    parser.add_argument(
        "--branch-name", help="Name of the branch to create", default=None
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without making changes",
    )
    parser.add_argument(
        "--no-input",
        action="store_true",
        help="Run without user interaction",
    )
    args = parser.parse_args()

    try:
        reorganizer = RepoReorganizer(args.repo_path, args.template_path, args.branch_name)

        # Analyze current structure
        console.print("\n[bold blue]Analyzing repository structure...[/bold blue]")
        missing_dirs, missing_files, extra_files = reorganizer.analyze_structure()

        # Show analysis results
        console.print("\n[bold green]Analysis Results:[/bold green]")
        if missing_dirs:
            console.print("\nMissing directories:")
            for dir_path in missing_dirs:
                console.print(f"  - {dir_path}")
        if missing_files:
            console.print("\nMissing files:")
            for file_path in missing_files:
                console.print(f"  - {file_path}")
        if extra_files:
            console.print("\nFiles to reorganize:")
            for file_path in extra_files:
                console.print(f"  - {file_path}")

        # Create and show plan
        plan = reorganizer.create_plan(missing_dirs, missing_files, extra_files)
        console.print("\n[bold green]Proposed Changes:[/bold green]")
        for change in plan:
            console.print(f"  - {change['action']}: {change['path']}")
            if "destination" in change:
                console.print(f"    → {change['destination']}")

        # Confirm execution
        if not args.dry_run and plan:
            if not args.no_input and not Confirm.ask("\nProceed with reorganization?"):
                console.print("Aborted.")
                return

        # Execute plan
        reorganizer.execute_plan(plan, args.dry_run)

        # Commit changes
        if not args.dry_run and reorganizer.changes:
            if not args.no_input and Confirm.ask("\nCommit changes to new branch?"):
                reorganizer.commit_changes()

        # Generate and save report
        report = reorganizer.generate_report()
        report_path = Path(args.repo_path) / "reorganization_report.md"
        with open(report_path, "w") as f:
            f.write(report)
        console.print(f"\n[bold green]Report saved to:[/bold green] {report_path}")

    except Exception as e:
        console.print(f"[bold red]Error:[/bold red] {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()