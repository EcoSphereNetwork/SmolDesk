#!/usr/bin/env python3
# ruff: noqa: E501
"""
Repository Reorganization Script

This script reorganizes an existing repository to match the template structure.
"""

import argparse
import fnmatch
import json
import logging
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

import requests
import toml
from pydantic import BaseModel, Field

import git
import rich
from rich.console import Console
from rich.logging import RichHandler
from rich.progress import Progress, SpinnerColumn, TextColumn
from rich.prompt import Confirm, Prompt

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(message)s",
    datefmt="[%X]",
    handlers=[RichHandler(rich_tracebacks=True)],
)

log = logging.getLogger("rich")
console = Console()

# Template structure definition
# Configuration classes
class CustomTemplate(BaseModel):
    """Custom template configuration."""
    name: str = Field(description="Template name")
    description: str = Field(description="Template description")
    path: str = Field(description="Path to template files")
    variables: Dict[str, str] = Field(default_factory=dict, description="Template variables")
    files: List[str] = Field(default_factory=list, description="Files to include")
    directories: List[str] = Field(default_factory=list, description="Directories to include")
    hooks: Dict[str, str] = Field(default_factory=dict, description="Template hooks")

class TemplateHook(BaseModel):
    """Template hook configuration."""
    name: str = Field(description="Hook name")
    script: str = Field(description="Hook script")
    events: List[str] = Field(description="Hook events")
    order: int = Field(default=0, description="Hook order")

class MigrationScript(BaseModel):
    """Migration script configuration."""
    name: str = Field(description="Script name")
    description: str = Field(description="Script description")
    source_version: str = Field(description="Source version")
    target_version: str = Field(description="Target version")
    script: str = Field(description="Migration script")
    reversible: bool = Field(default=False, description="Whether the script is reversible")

class ProjectAnalysis(BaseModel):
    """Project analysis results."""
    name: str = Field(description="Project name")
    description: str = Field(description="Project description")
    language: str = Field(description="Primary language")
    dependencies: Dict[str, List[str]] = Field(description="Project dependencies")
    structure: Dict[str, List[str]] = Field(description="Project structure")
    issues: List[Dict[str, str]] = Field(description="Issues found")
    recommendations: List[Dict[str, str]] = Field(description="Recommendations")

class SecurityCheck(BaseModel):
    """Security check configuration."""
    name: str = Field(description="Check name")
    description: str = Field(description="Check description")
    severity: str = Field(description="Check severity")
    pattern: str = Field(description="Check pattern")
    fix: Optional[str] = Field(description="Fix suggestion")

# Configuration classes
class FilePattern(BaseModel):
    """Configuration for a file pattern."""
    patterns: List[str] = Field(description="File patterns to match")
    exclude: List[str] = Field(default_factory=list, description="Patterns to exclude")
    dest: str = Field(description="Destination path template")

    def match(self, path: str) -> bool:
        """Check if a path matches this pattern."""
        path = str(path)
        # Check if path matches any pattern
        matches = False
        for pattern in self.patterns:
            if fnmatch.fnmatch(path, pattern):
                matches = True
                break
        
        # Check exclusions
        if matches and self.exclude:
            for exclude in self.exclude:
                if fnmatch.fnmatch(path, exclude):
                    matches = False
                    break
        
        return matches

class DirectoryConfig(BaseModel):
    """Configuration for a directory."""
    description: str = Field(description="Directory description")
    required: bool = Field(default=True, description="Whether the directory is required")

class FileConfig(BaseModel):
    """Configuration for a file."""
    description: str = Field(description="File description")
    required: bool = Field(default=True, description="Whether the file is required")
    template: bool = Field(default=True, description="Whether to copy from template")

class RepoConfig(BaseModel):
    """Repository configuration."""
    name: str = Field(description="Repository name")
    description: str = Field(description="Repository description")
    author: str = Field(description="Repository author")
    email: str = Field(description="Author email")
    python_version: str = Field(default="3.9", description="Python version")
    use_docker: bool = Field(default=True, description="Whether to use Docker")
    use_github_actions: bool = Field(default=True, description="Whether to use GitHub Actions")
    use_docs: bool = Field(default=True, description="Whether to use documentation")

@dataclass
class ReorganizeConfig:
    """Configuration for repository reorganization."""
    ignore_patterns: List[str] = field(default_factory=lambda: [
        ".git/*",
        "__pycache__/*",
        "*.pyc",
        ".pytest_cache/*",
        ".mypy_cache/*",
        ".ruff_cache/*",
        "*.egg-info/*",
        "build/*",
        "dist/*",
        ".env",
        ".venv/*",
        "node_modules/*",
    ])
    
    def should_ignore(self, path: str) -> bool:
        """Check if a path should be ignored."""
        path = str(path)
        return any(
            fnmatch.fnmatch(path, pattern)
            for pattern in self.ignore_patterns
        )

# File patterns to identify file types
FILE_PATTERNS = {
    "python_source": {
        "patterns": ["*.py"],
        "exclude": ["test_*.py", "conftest.py"],
        "dest": "src/{module_path}",
    },
    "python_test": {
        "patterns": ["test_*.py", "conftest.py"],
        "dest": "tests/{test_type}/{module_path}",
    },
    "documentation": {
        "patterns": ["*.md", "*.rst"],
        "exclude": ["README.md", "CHANGELOG.md", "LICENSE.md"],
        "dest": "docs/{doc_type}/{name}",
    },
    "github_workflow": {
        "patterns": [".github/workflows/*.yml", ".github/workflows/*.yaml"],
        "dest": ".github/workflows/{name}",
    },
    "docker": {
        "patterns": ["Dockerfile*", "docker-compose*.yml", "docker-compose*.yaml"],
        "dest": "{name}",
    },
    "config": {
        "patterns": ["*.toml", "*.ini", "*.cfg", "*.conf", ".env*"],
        "dest": "{name}",
    },
}

# Template structure definition
TEMPLATE_STRUCTURE = {
    "required_dirs": {
        ".github/workflows": "GitHub Actions workflow configurations",
        "docs": "Project documentation",
        "src": "Source code directory",
        "tests": "Test files",
        "scripts": "Utility and automation scripts",
    },
    "required_files": {
        "README.md": "Project documentation",
        "LICENSE": "Project license",
        ".gitignore": "Git ignore rules",
        "pyproject.toml": "Project configuration and dependencies",
        ".pre-commit-config.yaml": "Pre-commit hook configuration",
        "Dockerfile": "Docker configuration",
        "docker-compose.yml": "Docker Compose configuration",
        "Makefile": "Build automation",
        "mkdocs.yml": "Documentation configuration",
    },
    "optional_dirs": {
        ".devcontainer": "Development container configuration",
        "docs/api": "API documentation",
        "docs/guides": "User guides",
        "docs/development": "Development documentation",
        "tests/unit": "Unit tests",
        "tests/integration": "Integration tests",
        "tests/e2e": "End-to-end tests",
    },
}


class RepoReorganizer:
    """Handles repository reorganization."""

    def __init__(
        self,
        repo_path: str,
        template_path: str,
        branch_name: Optional[str] = None,
        config: Optional[ReorganizeConfig] = None,
    ):
        """Initialize the reorganizer.

        Args:
            repo_path: Path to the repository to reorganize
            template_path: Path to the template repository
            branch_name: Name of the branch to create (optional)
            config: Reorganization configuration (optional)
        """
        self.repo_path = Path(repo_path).resolve()
        self.template_path = Path(template_path).resolve()
        self.branch_name = branch_name or f"refactor/reorganize-{datetime.now():%Y%m%d-%H%M%S}"
        self.repo = git.Repo(self.repo_path)
        self.changes: List[Dict[str, str]] = []
        self.config = config or ReorganizeConfig()
        
        # Load repository configuration
        self.repo_config = self._load_repo_config()
        
        # Convert file patterns to models
        self.file_patterns = {
            name: FilePattern(**pattern)
            for name, pattern in FILE_PATTERNS.items()
        }
        
        # Convert directory configs to models
        self.dir_configs = {
            name: DirectoryConfig(description=config)
            for name, config in TEMPLATE_STRUCTURE["required_dirs"].items()
        }
        
        # Convert file configs to models
        self.file_configs = {
            name: FileConfig(description=config)
            for name, config in TEMPLATE_STRUCTURE["required_files"].items()
        }
    
    def _load_repo_config(self) -> RepoConfig:
        """Load repository configuration."""
        try:
            # Try to load from pyproject.toml
            if (self.repo_path / "pyproject.toml").exists():
                with open(self.repo_path / "pyproject.toml") as f:
                    data = toml.load(f)
                    if "tool" in data and "poetry" in data["tool"]:
                        poetry_data = data["tool"]["poetry"]
                        return RepoConfig(
                            name=poetry_data.get("name", self.repo_path.name),
                            description=poetry_data.get("description", ""),
                            author=poetry_data.get("authors", [""])[0].split("<")[0].strip(),
                            email=poetry_data.get("authors", ["<>"])[0].split("<")[1].rstrip(">"),
                            python_version=poetry_data.get("dependencies", {}).get("python", "^3.9").lstrip("^"),
                        )
        except Exception as e:
            log.warning(f"Error loading pyproject.toml: {e}")
        
        # Use repository information
        try:
            return RepoConfig(
                name=self.repo_path.name,
                description=self.repo.description or "",
                author=self.repo.config_reader().get_value("user", "name", ""),
                email=self.repo.config_reader().get_value("user", "email", ""),
                python_version="3.9",
            )
        except Exception as e:
            log.warning(f"Error loading git config: {e}")
        
        # Use default values
        return RepoConfig(
            name=self.repo_path.name,
            description="",
            author="",
            email="",
            python_version="3.9",
        )

    def analyze_structure(self) -> Tuple[Set[str], Set[str], Set[str]]:
        """Analyze the current repository structure.

        Returns:
            Tuple of (missing_dirs, missing_files, extra_files)
        """
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console,
        ) as progress:
            task = progress.add_task("Analyzing repository structure...", total=None)

            # Get current structure
            current_dirs = set()
            current_files = set()
            for path in self.repo_path.rglob("*"):
                if self.config.should_ignore(path):
                    continue
                
                try:
                    rel_path = str(path.relative_to(self.repo_path))
                    if path.is_dir():
                        current_dirs.add(rel_path)
                    elif path.is_file():
                        current_files.add(rel_path)
                except Exception as e:
                    log.warning(f"Error processing path {path}: {e}")

            # Required structure based on configuration
            required_dirs = {
                name
                for name, config in self.dir_configs.items()
                if config.required
            }
            
            required_files = {
                name
                for name, config in self.file_configs.items()
                if config.required
            }

            # Find differences
            missing_dirs = required_dirs - current_dirs
            missing_files = required_files - current_files
            extra_files = {
                f for f in current_files
                if not any(
                    pattern.match(f)
                    for pattern in self.file_patterns.values()
                )
            }

            progress.update(task, completed=True)

        return missing_dirs, missing_files, extra_files

    def create_plan(
        self, missing_dirs: Set[str], missing_files: Set[str], extra_files: Set[str]
    ) -> List[Dict[str, str]]:
        """Create a reorganization plan.

        Args:
            missing_dirs: Directories to create
            missing_files: Files to add
            extra_files: Files to potentially move

        Returns:
            List of planned changes
        """
        plan = []

        # Plan directory creation
        for dir_path in missing_dirs:
            if dir_path in self.dir_configs:
                config = self.dir_configs[dir_path]
                plan.append({
                    "action": "create_dir",
                    "path": dir_path,
                    "description": config.description,
                    "required": config.required,
                })

        # Plan file copying
        for file_path in missing_files:
            if file_path in self.file_configs:
                config = self.file_configs[file_path]
                if config.template:
                    plan.append({
                        "action": "copy_file",
                        "path": file_path,
                        "source": str(self.template_path / file_path),
                        "description": config.description,
                        "required": config.required,
                    })

        # Plan file moves
        for file_path in extra_files:
            # Skip ignored files
            if self.config.should_ignore(file_path):
                continue

            # Determine appropriate destination based on file type
            file_type, vars = self._detect_file_type(file_path)
            if file_type and file_type in self.file_patterns:
                pattern = self.file_patterns[file_type]
                try:
                    # Remove duplicate src directories
                    if "module_path" in vars:
                        parts = Path(vars["module_path"]).parts
                        filtered_parts = []
                        for part in parts:
                            if part != "src" or not filtered_parts or filtered_parts[-1] != "src":
                                filtered_parts.append(part)
                        vars["module_path"] = str(Path(*filtered_parts))

                    dest = pattern.dest.format(**vars)
                    plan.append({
                        "action": "move_file",
                        "path": file_path,
                        "destination": dest,
                        "description": f"Move to appropriate directory",
                        "file_type": file_type,
                    })
                except KeyError as e:
                    log.warning(f"Missing variable {e} for file {file_path}")

        # Add configuration updates
        plan.append({
            "action": "update_config",
            "path": "pyproject.toml",
            "description": "Update project configuration",
            "config": self.repo_config.model_dump(),
        })

        # Add documentation setup if enabled
        if self.repo_config.use_docs:
            plan.extend([
                {
                    "action": "create_dir",
                    "path": "docs/api",
                    "description": "API documentation directory",
                },
                {
                    "action": "create_dir",
                    "path": "docs/guides",
                    "description": "User guides directory",
                },
                {
                    "action": "create_dir",
                    "path": "docs/development",
                    "description": "Development documentation directory",
                },
            ])

        # Add GitHub Actions setup if enabled
        if self.repo_config.use_github_actions:
            plan.extend([
                {
                    "action": "create_dir",
                    "path": ".github/workflows",
                    "description": "GitHub Actions workflows directory",
                },
                {
                    "action": "setup_workflows",
                    "description": "Set up GitHub Actions workflows",
                },
            ])

        # Add Docker setup if enabled
        if self.repo_config.use_docker:
            plan.extend([
                {
                    "action": "copy_file",
                    "path": "Dockerfile",
                    "source": str(self.template_path / "Dockerfile"),
                    "description": "Docker configuration",
                },
                {
                    "action": "copy_file",
                    "path": "docker-compose.yml",
                    "source": str(self.template_path / "docker-compose.yml"),
                    "description": "Docker Compose configuration",
                },
            ])

        return plan

    def _detect_file_type(self, file_path: str) -> Tuple[Optional[str], Dict[str, str]]:
        """Detect file type and extract path variables.

        Args:
            file_path: Path to the file

        Returns:
            Tuple of (file_type, path_vars) or (None, {})
        """
        path = Path(file_path)
        
        for file_type, config in FILE_PATTERNS.items():
            # Check if file matches any pattern
            matches = False
            for pattern in config["patterns"]:
                if path.match(pattern):
                    matches = True
                    break
            
            # Check exclusions
            if matches and "exclude" in config:
                for exclude in config["exclude"]:
                    if path.match(exclude):
                        matches = False
                        break
            
            if matches:
                # Extract variables for destination path
                vars = {"name": path.name}
                
                if file_type == "python_source":
                    # Determine module path
                    module_path = path.parent.relative_to(path.parent.anchor) if path.parent != Path(".") else ""
                    vars["module_path"] = str(module_path / path.name)
                
                elif file_type == "python_test":
                    # Determine test type (unit, integration, e2e)
                    if "integration" in str(path):
                        vars["test_type"] = "integration"
                    elif "e2e" in str(path):
                        vars["test_type"] = "e2e"
                    else:
                        vars["test_type"] = "unit"
                    vars["module_path"] = path.name
                
                elif file_type == "documentation":
                    # Determine doc type
                    if "api" in str(path.parent):
                        vars["doc_type"] = "api"
                    elif "guide" in str(path.parent):
                        vars["doc_type"] = "guides"
                    else:
                        vars["doc_type"] = "development"
                
                return file_type, vars
        
        return None, {}

    def _suggest_destination(self, file_path: str) -> Optional[str]:
        """Suggest appropriate destination for a file.

        Args:
            file_path: Current file path

        Returns:
            Suggested destination path or None
        """
        file_type, vars = self._detect_file_type(file_path)
        if file_type:
            try:
                return FILE_PATTERNS[file_type]["dest"].format(**vars)
            except KeyError as e:
                log.warning(f"Missing variable {e} for file {file_path}")
                return None
        return None

    def _copy_template_file(self, file_path: str) -> None:
        """Copy a file from the template repository.

        Args:
            file_path: Path to the file relative to repository root
        """
        src = self.template_path / file_path
        dest = self.repo_path / file_path
        
        if not src.exists():
            log.warning(f"Template file not found: {file_path}")
            return
        
        os.makedirs(dest.parent, exist_ok=True)
        shutil.copy2(src, dest)

    def _analyze_python_files(self) -> Dict[str, List[str]]:
        """Analyze Python files for imports and dependencies.

        Returns:
            Dictionary of module dependencies
        """
        import ast
        
        dependencies = {}
        
        for py_file in self.repo_path.rglob("*.py"):
            if "test_" in py_file.name:
                continue
                
            rel_path = py_file.relative_to(self.repo_path)
            try:
                with open(py_file) as f:
                    tree = ast.parse(f.read())
                
                imports = []
                for node in ast.walk(tree):
                    if isinstance(node, ast.Import):
                        for name in node.names:
                            imports.append(name.name)
                    elif isinstance(node, ast.ImportFrom):
                        if node.module:
                            imports.append(node.module)
                
                dependencies[str(rel_path)] = imports
            except Exception as e:
                log.warning(f"Error analyzing {rel_path}: {e}")
        
        return dependencies

    def _update_pyproject_toml(self) -> None:
        """Update pyproject.toml with project information."""
        import toml
        
        pyproject_path = self.repo_path / "pyproject.toml"
        if not pyproject_path.exists():
            self._copy_template_file("pyproject.toml")
        
        try:
            with open(pyproject_path) as f:
                config = toml.load(f)
            
            # Update project name
            repo_name = self.repo_path.name.lower().replace("-", "_")
            if "tool" in config and "poetry" in config["tool"]:
                config["tool"]["poetry"]["name"] = repo_name
            
            # Update Python dependencies based on imports
            dependencies = self._analyze_python_files()
            if dependencies and "tool" in config and "poetry" in config["tool"]:
                # Add common dependencies
                deps = config["tool"]["poetry"].get("dependencies", {})
                deps.update({
                    "pydantic": "^2.5.3",
                    "python-dotenv": "^1.0.0",
                    "rich": "^13.7.0",
                })
                config["tool"]["poetry"]["dependencies"] = deps
            
            with open(pyproject_path, "w") as f:
                toml.dump(config, f)
        
        except Exception as e:
            log.error(f"Error updating pyproject.toml: {e}")

    def _update_readme(self) -> None:
        """Update README.md with template structure."""
        readme_path = self.repo_path / "README.md"
        if not readme_path.exists():
            self._copy_template_file("README.md")
            return
        
        try:
            with open(readme_path) as f:
                content = f.read()
            
            # Keep original title and description
            title_match = re.search(r"#\s+(.+)", content)
            desc_match = re.search(r"#.+\n+([^#\n]+)", content)
            
            with open(self.template_path / "README.md") as f:
                template = f.read()
            
            if title_match:
                template = re.sub(r"<h1>([^<]+)</h1>", f"<h1>{title_match.group(1)}</h1>", template)
            
            if desc_match:
                template = re.sub(
                    r"<p>([^<]+)</p>",
                    f"<p>{desc_match.group(1).strip()}</p>",
                    template,
                    count=1
                )
            
            with open(readme_path, "w") as f:
                f.write(template)
        
        except Exception as e:
            log.error(f"Error updating README.md: {e}")

    def _setup_git_hooks(self) -> None:
        """Set up git hooks using pre-commit."""
        try:
            if not (self.repo_path / ".pre-commit-config.yaml").exists():
                self._copy_template_file(".pre-commit-config.yaml")
            
            subprocess.run(
                ["poetry", "run", "pre-commit", "install"],
                cwd=self.repo_path,
                check=True,
                capture_output=True,
            )
        except Exception as e:
            log.error(f"Error setting up git hooks: {e}")

    def _setup_github_actions(self) -> None:
        """Set up GitHub Actions workflows."""
        workflows_dir = self.repo_path / ".github" / "workflows"
        if not workflows_dir.exists():
            os.makedirs(workflows_dir, exist_ok=True)
        
        # Copy all workflow files
        template_workflows = (self.template_path / ".github" / "workflows").glob("reusable-*.yml")
        for workflow in template_workflows:
            shutil.copy2(workflow, workflows_dir / workflow.name)

    def execute_plan(self, plan: List[Dict[str, str]], dry_run: bool = False) -> None:
        """Execute the reorganization plan.

        Args:
            plan: List of planned changes
            dry_run: If True, only show what would be done
        """
        with Progress(
            SpinnerColumn(),
            TextColumn("[progress.description]{task.description}"),
            console=console,
        ) as progress:
            task = progress.add_task("Executing reorganization plan...", total=len(plan))

            for change in plan:
                action = change["action"]
                path = change.get("path", "")
                description = change.get("description", "")

                try:
                    if dry_run:
                        log.info(f"Would {action}: {description}")
                        continue

                    progress.update(task, description=f"Executing: {description}")

                    if action == "create_dir":
                        os.makedirs(self.repo_path / path, exist_ok=True)
                    
                    elif action == "copy_file":
                        source = change.get("source")
                        if source:
                            shutil.copy2(source, self.repo_path / path)
                    
                    elif action == "move_file":
                        dest = change.get("destination")
                        if dest:
                            dest_path = self.repo_path / dest
                            os.makedirs(dest_path.parent, exist_ok=True)
                            shutil.move(self.repo_path / path, dest_path)
                    
                    elif action == "update_config":
                        config = change.get("config", {})
                        self._update_project_config(config)
                    
                    elif action == "setup_workflows":
                        self._setup_github_actions()
                    
                    self.changes.append(change)

                except Exception as e:
                    if change.get("required", False):
                        log.error(f"Error executing {action} for {path}: {e}")
                    else:
                        log.warning(f"Error executing {action} for {path}: {e}")

                progress.advance(task)

            if not dry_run:
                # Final steps
                final_steps = [
                    ("Setting up development environment", self._setup_development_environment),
                    ("Setting up documentation", self._setup_documentation),
                    ("Setting up git hooks", self._setup_git_hooks),
                    ("Performing final cleanup", self._cleanup),
                ]

                for description, func in final_steps:
                    try:
                        progress.update(task, description=description)
                        func()
                    except Exception as e:
                        log.warning(f"Error during {description}: {e}")
                    progress.advance(task)

    def _update_project_config(self, config: dict) -> None:
        """Update project configuration files."""
        # Update pyproject.toml
        try:
            pyproject_path = self.repo_path / "pyproject.toml"
            if pyproject_path.exists():
                with open(pyproject_path) as f:
                    data = toml.load(f)
            else:
                data = {}

            # Update poetry configuration
            if "tool" not in data:
                data["tool"] = {}
            if "poetry" not in data["tool"]:
                data["tool"]["poetry"] = {}

            poetry_config = data["tool"]["poetry"]
            poetry_config.update({
                "name": config["name"],
                "description": config["description"],
                "authors": [f"{config['author']} <{config['email']}>"],
                "python": f"^{config['python_version']}",
            })

            with open(pyproject_path, "w") as f:
                toml.dump(data, f)

        except Exception as e:
            log.error(f"Error updating pyproject.toml: {e}")

        # Update other configuration files as needed
        self._update_readme()
        self._update_documentation()

    def _setup_development_environment(self) -> None:
        """Set up development environment."""
        try:
            # Check if poetry is installed
            try:
                subprocess.run(
                    ["poetry", "--version"],
                    check=True,
                    capture_output=True,
                )
            except:
                # Install poetry
                subprocess.run(
                    ["pip", "install", "poetry"],
                    check=True,
                    capture_output=True,
                )

            # Install dependencies
            try:
                subprocess.run(
                    ["poetry", "install"],
                    cwd=self.repo_path,
                    check=True,
                    capture_output=True,
                )
            except subprocess.CalledProcessError as e:
                log.warning(f"Error installing dependencies: {e.output.decode()}")
                # Try to fix common issues
                if "pyproject.toml not found" in str(e.output):
                    self._copy_template_file("pyproject.toml")
                    subprocess.run(
                        ["poetry", "install"],
                        cwd=self.repo_path,
                        check=True,
                        capture_output=True,
                    )

            # Set up pre-commit
            if (self.repo_path / ".pre-commit-config.yaml").exists():
                try:
                    subprocess.run(
                        ["poetry", "run", "pre-commit", "install"],
                        cwd=self.repo_path,
                        check=True,
                        capture_output=True,
                    )
                except subprocess.CalledProcessError as e:
                    log.warning(f"Error setting up pre-commit: {e.output.decode()}")

        except Exception as e:
            log.error(f"Error setting up development environment: {e}")

    def _setup_documentation(self) -> None:
        """Set up project documentation."""
        if not self.repo_config.use_docs:
            return

        try:
            # Create documentation directories
            for dir_path in ["docs/api", "docs/guides", "docs/development"]:
                os.makedirs(self.repo_path / dir_path, exist_ok=True)

            # Copy documentation files
            doc_files = {
                "docs/index.md": "Documentation home page",
                "docs/guides/quickstart.md": "Quick start guide",
                "docs/development/guide.md": "Development guide",
                "docs/api/reference.md": "API reference",
            }

            for file_path, description in doc_files.items():
                source = self.template_path / file_path
                if source.exists():
                    shutil.copy2(source, self.repo_path / file_path)

            # Update mkdocs.yml
            if (self.template_path / "mkdocs.yml").exists():
                shutil.copy2(
                    self.template_path / "mkdocs.yml",
                    self.repo_path / "mkdocs.yml",
                )

        except Exception as e:
            log.error(f"Error setting up documentation: {e}")

    def _update_documentation(self) -> None:
        """Update documentation with project information."""
        try:
            # Update API documentation
            api_ref = self.repo_path / "docs/api/reference.md"
            if api_ref.exists():
                with open(api_ref) as f:
                    content = f.read()
                
                content = content.replace(
                    "ecosphere-project",
                    self.repo_config.name,
                )
                
                with open(api_ref, "w") as f:
                    f.write(content)

        except Exception as e:
            log.warning(f"Error updating documentation: {e}")

    def _cleanup(self) -> None:
        """Perform final cleanup."""
        try:
            # Remove empty directories
            for dirpath, dirnames, filenames in os.walk(self.repo_path, topdown=False):
                try:
                    if not dirnames and not filenames and dirpath != str(self.repo_path):
                        os.rmdir(dirpath)
                except:
                    pass

            # Remove cache directories
            for pattern in ["*.pyc", "__pycache__", "*.egg-info"]:
                for path in self.repo_path.rglob(pattern):
                    try:
                        if path.is_file():
                            path.unlink()
                        elif path.is_dir():
                            shutil.rmtree(path)
                    except:
                        pass

        except Exception as e:
            log.warning(f"Error during cleanup: {e}")

    def _analyze_project(self) -> ProjectAnalysis:
        """Analyze project structure and dependencies."""
        try:
            # Detect primary language
            language_stats = {}
            for path in self.repo_path.rglob("*"):
                if path.is_file() and not self.config.should_ignore(path):
                    ext = path.suffix.lower()
                    if ext:
                        language_stats[ext] = language_stats.get(ext, 0) + 1
            
            primary_language = max(language_stats.items(), key=lambda x: x[1])[0]
            if primary_language == ".py":
                primary_language = "Python"
            elif primary_language == ".js":
                primary_language = "JavaScript"
            elif primary_language == ".java":
                primary_language = "Java"
            else:
                primary_language = "Unknown"

            # Analyze dependencies
            dependencies = {}
            if primary_language == "Python":
                # Python dependencies
                dependencies = self._analyze_python_files()
            
            # Analyze structure
            structure = {}
            for path in self.repo_path.rglob("*"):
                if not self.config.should_ignore(path):
                    rel_path = str(path.relative_to(self.repo_path))
                    parent = str(path.parent.relative_to(self.repo_path))
                    if parent not in structure:
                        structure[parent] = []
                    if path.is_file():
                        structure[parent].append(path.name)

            # Find issues
            issues = []
            
            # Missing documentation
            if not (self.repo_path / "README.md").exists():
                issues.append({
                    "type": "documentation",
                    "severity": "high",
                    "message": "Missing README.md file",
                })
            
            # Missing license
            if not (self.repo_path / "LICENSE").exists():
                issues.append({
                    "type": "license",
                    "severity": "high",
                    "message": "Missing LICENSE file",
                })
            
            # Missing tests
            if not list(self.repo_path.rglob("test_*.py")):
                issues.append({
                    "type": "testing",
                    "severity": "medium",
                    "message": "No test files found",
                })

            # Generate recommendations
            recommendations = []
            
            # Documentation recommendations
            if not (self.repo_path / "docs").exists():
                recommendations.append({
                    "type": "documentation",
                    "message": "Add project documentation using MkDocs",
                    "action": "setup_docs",
                })
            
            # CI/CD recommendations
            if not (self.repo_path / ".github" / "workflows").exists():
                recommendations.append({
                    "type": "ci_cd",
                    "message": "Add GitHub Actions workflows for CI/CD",
                    "action": "setup_workflows",
                })
            
            # Docker recommendations
            if not (self.repo_path / "Dockerfile").exists():
                recommendations.append({
                    "type": "containerization",
                    "message": "Add Docker support for containerization",
                    "action": "setup_docker",
                })

            return ProjectAnalysis(
                name=self.repo_path.name,
                description=self.repo.description or "",
                language=primary_language,
                dependencies=dependencies,
                structure=structure,
                issues=issues,
                recommendations=recommendations,
            )

        except Exception as e:
            log.error(f"Error analyzing project: {e}")
            return ProjectAnalysis(
                name=self.repo_path.name,
                description="",
                language="Unknown",
                dependencies={},
                structure={},
                issues=[],
                recommendations=[],
            )

    def _run_security_checks(self) -> List[Dict[str, str]]:
        """Run security checks on the repository."""
        findings = []

        # Define security checks
        checks = [
            SecurityCheck(
                name="hardcoded_secrets",
                description="Check for hardcoded secrets",
                severity="high",
                pattern=r"(?i)(password|secret|token|key).*?['\"][^'\"]+['\"]",
                fix="Move secrets to environment variables or secure storage",
            ),
            SecurityCheck(
                name="sql_injection",
                description="Check for potential SQL injection",
                severity="high",
                pattern=r"(?i)execute\(['\"][^'\"]*%[^'\"]*['\"]",
                fix="Use parameterized queries",
            ),
            SecurityCheck(
                name="debug_code",
                description="Check for debug code",
                severity="medium",
                pattern=r"(?i)(print|console\.log|debug)\(",
                fix="Remove debug statements before deployment",
            ),
        ]

        try:
            # Run checks
            for check in checks:
                pattern = re.compile(check.pattern)
                
                for path in self.repo_path.rglob("*"):
                    if path.is_file() and not self.config.should_ignore(path):
                        try:
                            with open(path) as f:
                                content = f.read()
                                matches = pattern.finditer(content)
                                for match in matches:
                                    line_no = content.count("\n", 0, match.start()) + 1
                                    findings.append({
                                        "file": str(path.relative_to(self.repo_path)),
                                        "line": str(line_no),
                                        "type": check.name,
                                        "severity": check.severity,
                                        "description": check.description,
                                        "fix": check.fix or "",
                                    })
                        except Exception as e:
                            log.warning(f"Error checking file {path}: {e}")

        except Exception as e:
            log.error(f"Error running security checks: {e}")

        return findings

    def _apply_template(self, template: CustomTemplate) -> None:
        """Apply a custom template to the repository."""
        try:
            # Create directories
            for dir_path in template.directories:
                dir_path = dir_path.format(**template.variables)
                os.makedirs(self.repo_path / dir_path, exist_ok=True)

            # Copy files
            for file_path in template.files:
                src = Path(template.path) / file_path
                if src.exists():
                    dest = self.repo_path / file_path.format(**template.variables)
                    os.makedirs(dest.parent, exist_ok=True)
                    
                    # Read and replace variables
                    with open(src) as f:
                        content = f.read()
                    
                    for var, value in template.variables.items():
                        content = content.replace(f"${{{var}}}", value)
                    
                    with open(dest, "w") as f:
                        f.write(content)

            # Run hooks
            for event, script in template.hooks.items():
                try:
                    subprocess.run(
                        [script],
                        cwd=self.repo_path,
                        check=True,
                        capture_output=True,
                    )
                except Exception as e:
                    log.warning(f"Error running hook {event}: {e}")

        except Exception as e:
            log.error(f"Error applying template: {e}")

    def _run_migration(self, script: MigrationScript) -> bool:
        """Run a migration script."""
        try:
            # Create temporary script file
            script_path = self.repo_path / f".migration_{script.name}.py"
            with open(script_path, "w") as f:
                f.write(script.script)

            # Run migration
            result = subprocess.run(
                ["python", str(script_path)],
                cwd=self.repo_path,
                check=True,
                capture_output=True,
                text=True,
            )

            # Check output
            if "SUCCESS" in result.stdout:
                log.info(f"Migration {script.name} completed successfully")
                return True
            else:
                log.warning(f"Migration {script.name} completed with warnings: {result.stdout}")
                return False

        except Exception as e:
            log.error(f"Error running migration {script.name}: {e}")
            return False

        finally:
            # Cleanup
            try:
                script_path.unlink()
            except:
                pass

    def commit_changes(self) -> None:
        """Commit the changes to a new branch."""
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
            message += "Reorganize repository to match template structure.\n\n"
            
            # Group changes by type
            changes_by_type = {}
            for change in self.changes:
                change_type = change["action"]
                if change_type not in changes_by_type:
                    changes_by_type[change_type] = []
                changes_by_type[change_type].append(change["path"])

            # Add changes to commit message
            for change_type, paths in changes_by_type.items():
                message += f"\n{change_type.replace('_', ' ').title()}:\n"
                for path in sorted(paths):
                    message += f"* {path}\n"

            # Add template features
            message += "\nTemplate Features Added:\n"
            message += "* GitHub Actions workflows for CI/CD\n"
            message += "* Poetry for dependency management\n"
            message += "* Pre-commit hooks for code quality\n"
            message += "* Documentation structure\n"
            message += "* Testing framework\n"
            message += "* Docker support\n"

            # Commit changes
            self.repo.index.commit(message)

            # Try to push changes if remote exists
            try:
                if "origin" in self.repo.remotes:
                    self.repo.git.push("--set-upstream", "origin", self.branch_name)
                    log.info(f"Changes pushed to origin/{self.branch_name}")
            except Exception as e:
                log.warning(f"Could not push changes: {e}")

            log.info(f"Changes committed to branch: {self.branch_name}")
            log.info(f"Previous branch was: {current.name}")

            # Create pull request if GitHub token is available
            github_token = os.getenv("GITHUB_TOKEN")
            if github_token:
                try:
                    remote_url = self.repo.remotes.origin.url
                    if "github.com" in remote_url:
                        # Extract owner and repo from URL
                        parts = remote_url.split("github.com/")[-1].replace(".git", "").split("/")
                        if len(parts) == 2:
                            owner, repo = parts
                            
                            # Create pull request
                            url = f"https://api.github.com/repos/{owner}/{repo}/pulls"
                            headers = {
                                "Authorization": f"Bearer {github_token}",
                                "Accept": "application/vnd.github.v3+json",
                            }
                            data = {
                                "title": "Reorganize Repository Structure",
                                "body": message,
                                "head": self.branch_name,
                                "base": "main",
                                "draft": True,
                            }
                            
                            response = requests.post(url, headers=headers, json=data)
                            if response.status_code == 201:
                                pr_url = response.json()["html_url"]
                                log.info(f"Created draft pull request: {pr_url}")
                            else:
                                log.warning(f"Could not create pull request: {response.text}")
                except Exception as e:
                    log.warning(f"Could not create pull request: {e}")

        except Exception as e:
            log.error(f"Error committing changes: {e}")
            # Try to restore previous branch
            try:
                current.checkout()
            except:
                pass

    def generate_report(self) -> str:
        """Generate a report of changes made.

        Returns:
            Report as a string
        """
        report = "# Repository Reorganization Report\n\n"
        report += f"Date: {datetime.now():%Y-%m-%d %H:%M:%S}\n\n"

        # Repository Information
        report += "## Repository Information\n\n"
        report += f"* Repository: {self.repo_path.name}\n"
        report += f"* Branch: {self.branch_name}\n"
        try:
            if "origin" in self.repo.remotes:
                report += f"* Remote: {self.repo.remotes.origin.url}\n"
        except:
            pass

        # Structure Analysis
        report += "\n## Structure Analysis\n\n"
        missing_dirs, missing_files, extra_files = self.analyze_structure()
        
        if missing_dirs:
            report += "### Missing Directories\n\n"
            for dir_path in sorted(missing_dirs):
                report += f"* `{dir_path}`\n"
                if dir_path in TEMPLATE_STRUCTURE["required_dirs"]:
                    report += f"  - {TEMPLATE_STRUCTURE['required_dirs'][dir_path]}\n"

        if missing_files:
            report += "\n### Missing Files\n\n"
            for file_path in sorted(missing_files):
                report += f"* `{file_path}`\n"
                if file_path in TEMPLATE_STRUCTURE["required_files"]:
                    report += f"  - {TEMPLATE_STRUCTURE['required_files'][file_path]}\n"

        if extra_files:
            report += "\n### Files to Reorganize\n\n"
            for file_path in sorted(extra_files):
                report += f"* `{file_path}`\n"
                dest = self._suggest_destination(file_path)
                if dest:
                    report += f"  - → `{dest}`\n"

        # Changes Made
        if self.changes:
            report += "\n## Changes Made\n\n"
            
            # Group changes by type
            changes_by_type = {}
            for change in self.changes:
                change_type = change["action"]
                if change_type not in changes_by_type:
                    changes_by_type[change_type] = []
                changes_by_type[change_type].append(change)

            for change_type, changes in changes_by_type.items():
                report += f"### {change_type.replace('_', ' ').title()}\n\n"
                for change in sorted(changes, key=lambda x: x.get("path", "")):
                    path = change.get("path", "")
                    if path:
                        report += f"* `{path}`\n"
                    else:
                        report += f"* {change.get('description', 'No description')}\n"
                    if "description" in change:
                        report += f"  - {change['description']}\n"
                    if "destination" in change:
                        report += f"  - → `{change['destination']}`\n"
                report += "\n"

        # Template Features
        report += "## Template Features Added\n\n"
        report += "### Development Environment\n\n"
        report += "* Poetry for dependency management\n"
        report += "* Pre-commit hooks for code quality\n"
        report += "* Docker and devcontainer support\n"
        report += "* VS Code configuration\n\n"

        report += "### CI/CD\n\n"
        report += "* GitHub Actions workflows\n"
        report += "* Automated testing\n"
        report += "* Code quality checks\n"
        report += "* Security scanning\n\n"

        report += "### Documentation\n\n"
        report += "* MkDocs with Material theme\n"
        report += "* API documentation\n"
        report += "* Development guides\n"
        report += "* User guides\n\n"

        # Python Analysis
        try:
            dependencies = self._analyze_python_files()
            if dependencies:
                report += "## Python Analysis\n\n"
                report += "### Module Dependencies\n\n"
                for module, imports in dependencies.items():
                    report += f"#### `{module}`\n\n"
                    if imports:
                        report += "Imports:\n"
                        for imp in sorted(imports):
                            report += f"* `{imp}`\n"
                    else:
                        report += "No imports found.\n"
                    report += "\n"
        except:
            pass

        # Next Steps
        report += "## Next Steps\n\n"
        report += "1. Review the changes and test the repository\n"
        report += "2. Update documentation as needed\n"
        report += "3. Configure GitHub repository settings:\n"
        report += "   - Branch protection rules\n"
        report += "   - Environment secrets\n"
        report += "   - Repository variables\n"
        report += "4. Set up external services:\n"
        report += "   - Code coverage (Codecov)\n"
        report += "   - Security scanning (Snyk)\n"
        report += "   - Documentation hosting\n"

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
    parser.add_argument(
        "--config",
        help="Path to configuration file",
    )
    parser.add_argument(
        "--python-version",
        help="Python version to use",
        default="3.9",
    )
    parser.add_argument(
        "--no-docker",
        action="store_true",
        help="Disable Docker setup",
    )
    parser.add_argument(
        "--no-github-actions",
        action="store_true",
        help="Disable GitHub Actions setup",
    )
    parser.add_argument(
        "--no-docs",
        action="store_true",
        help="Disable documentation setup",
    )
    parser.add_argument(
        "--custom-template",
        help="Path to custom template configuration",
    )
    parser.add_argument(
        "--migration-script",
        help="Path to migration script",
    )
    parser.add_argument(
        "--security-check",
        action="store_true",
        help="Run security checks",
    )
    parser.add_argument(
        "--analyze",
        action="store_true",
        help="Analyze project structure",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable debug logging",
    )
    args = parser.parse_args()

    # Configure logging
    if args.debug:
        log.setLevel(logging.DEBUG)

    try:
        # Load configuration
        config = ReorganizeConfig()
        repo_config = None

        # Load custom template if provided
        custom_template = None
        if args.custom_template:
            try:
                with open(args.custom_template) as f:
                    template_data = toml.load(f)
                    custom_template = CustomTemplate(**template_data)
            except Exception as e:
                log.warning(f"Error loading custom template: {e}")

        # Load migration script if provided
        migration_script = None
        if args.migration_script:
            try:
                with open(args.migration_script) as f:
                    script_data = toml.load(f)
                    migration_script = MigrationScript(**script_data)
            except Exception as e:
                log.warning(f"Error loading migration script: {e}")

        # Load repository configuration
        if args.config:
            try:
                with open(args.config) as f:
                    data = toml.load(f)
                    repo_config = RepoConfig(**data)
            except Exception as e:
                log.warning(f"Error loading configuration file: {e}")

        if not repo_config:
            # Create repository configuration
            repo_path = Path(args.repo_path)
            repo = git.Repo(repo_path)
            
            try:
                author = repo.config_reader().get_value("user", "name", "")
                email = repo.config_reader().get_value("user", "email", "")
            except:
                author = ""
                email = ""

            repo_config = RepoConfig(
                name=repo_path.name,
                description=repo.description or "",
                author=author,
                email=email,
                python_version=args.python_version,
                use_docker=not args.no_docker,
                use_github_actions=not args.no_github_actions,
                use_docs=not args.no_docs,
            )

        # Create reorganizer
        reorganizer = RepoReorganizer(
            args.repo_path,
            args.template_path,
            args.branch_name,
            config=config,
        )
        reorganizer.repo_config = repo_config

        # Run project analysis if requested
        if args.analyze:
            console.print("\n[bold blue]Analyzing project...[/bold blue]")
            analysis = reorganizer._analyze_project()
            
            console.print(f"\n[bold green]Project Analysis:[/bold green]")
            console.print(f"Name: {analysis.name}")
            console.print(f"Description: {analysis.description}")
            console.print(f"Language: {analysis.language}")
            
            if analysis.dependencies:
                console.print("\nDependencies:")
                for module, imports in analysis.dependencies.items():
                    console.print(f"  {module}:")
                    for imp in imports:
                        console.print(f"    - {imp}")
            
            if analysis.issues:
                console.print("\nIssues Found:")
                for issue in analysis.issues:
                    severity = issue["severity"].upper()
                    color = "red" if severity == "HIGH" else "yellow"
                    console.print(f"  [{color}]{severity}[/{color}] {issue['message']}")
            
            if analysis.recommendations:
                console.print("\nRecommendations:")
                for rec in analysis.recommendations:
                    console.print(f"  - {rec['message']}")

        # Run security checks if requested
        if args.security_check:
            console.print("\n[bold blue]Running security checks...[/bold blue]")
            findings = reorganizer._run_security_checks()
            
            if findings:
                console.print("\n[bold red]Security Issues Found:[/bold red]")
                for finding in findings:
                    severity = finding["severity"].upper()
                    color = "red" if severity == "HIGH" else "yellow"
                    console.print(f"\n[{color}]{finding['type']} ({severity})[/{color}]")
                    console.print(f"File: {finding['file']}:{finding['line']}")
                    console.print(f"Description: {finding['description']}")
                    if finding['fix']:
                        console.print(f"Fix: {finding['fix']}")
            else:
                console.print("\n[bold green]No security issues found[/bold green]")

        # Apply custom template if provided
        if custom_template:
            console.print("\n[bold blue]Applying custom template...[/bold blue]")
            reorganizer._apply_template(custom_template)

        # Run migration script if provided
        if migration_script:
            console.print("\n[bold blue]Running migration script...[/bold blue]")
            success = reorganizer._run_migration(migration_script)
            if not success:
                if not args.no_input and not Confirm.ask("\nMigration had warnings. Continue?"):
                    console.print("Aborted.")
                    return

        # Analyze current structure
        console.print("\n[bold blue]Analyzing repository structure...[/bold blue]")
        missing_dirs, missing_files, extra_files = reorganizer.analyze_structure()

        # Show analysis results
        console.print("\n[bold green]Analysis Results:[/bold green]")
        if missing_dirs:
            console.print("\nMissing directories:")
            for dir_path in sorted(missing_dirs):
                desc = reorganizer.dir_configs[dir_path].description if dir_path in reorganizer.dir_configs else ""
                console.print(f"  - {dir_path}" + (f" ({desc})" if desc else ""))

        if missing_files:
            console.print("\nMissing files:")
            for file_path in sorted(missing_files):
                desc = reorganizer.file_configs[file_path].description if file_path in reorganizer.file_configs else ""
                console.print(f"  - {file_path}" + (f" ({desc})" if desc else ""))

        if extra_files:
            console.print("\nFiles to reorganize:")
            for file_path in sorted(extra_files):
                if not config.should_ignore(file_path):
                    dest = reorganizer._suggest_destination(file_path)
                    if dest:
                        console.print(f"  - {file_path} → {dest}")
                    else:
                        console.print(f"  - {file_path}")

        # Create and show plan
        plan = reorganizer.create_plan(missing_dirs, missing_files, extra_files)
        console.print("\n[bold green]Proposed Changes:[/bold green]")
        
        # Group changes by type
        changes_by_type = {}
        for change in plan:
            action = change["action"]
            if action not in changes_by_type:
                changes_by_type[action] = []
            changes_by_type[action].append(change)

        for action, changes in sorted(changes_by_type.items()):
            console.print(f"\n{action.replace('_', ' ').title()}:")
            for change in sorted(changes, key=lambda x: x.get("path", "")):
                path = change.get("path", "")
                desc = change.get("description", "")
                console.print(f"  - {path}" + (f" ({desc})" if desc else ""))
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
        if log.level <= logging.DEBUG:
            import traceback
            traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()