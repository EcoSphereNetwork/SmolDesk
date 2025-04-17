.PHONY: install test lint format clean docs serve-docs

install:
	poetry install

test:
	poetry run pytest

lint:
	poetry run ruff check .
	poetry run black --check .
	poetry run mypy .

format:
	poetry run ruff check --fix .
	poetry run black .

clean:
	find . -type d -name "__pycache__" -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete
	find . -type f -name "*.pyo" -delete
	find . -type f -name "*.pyd" -delete
	find . -type f -name ".coverage" -delete
	find . -type d -name "*.egg-info" -exec rm -rf {} +
	find . -type d -name "*.egg" -exec rm -rf {} +
	find . -type d -name ".pytest_cache" -exec rm -rf {} +
	find . -type d -name ".mypy_cache" -exec rm -rf {} +
	find . -type d -name ".ruff_cache" -exec rm -rf {} +
	find . -type d -name "build" -exec rm -rf {} +
	find . -type d -name "dist" -exec rm -rf {} +

docs:
	poetry run mkdocs build

serve-docs:
	poetry run mkdocs serve