# Use multi-stage build for development and production
FROM python:3.11-slim as base

# Set environment variables
ENV PYTHONFAULTHANDLER=1 \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    POETRY_VERSION=1.5.1 \
    POETRY_NO_INTERACTION=1 \
    POETRY_VIRTUALENVS_CREATE=false \
    POETRY_CACHE_DIR='/var/cache/pypoetry' \
    PATH="$PATH:/root/.local/bin"

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Poetry
RUN curl -sSL https://install.python-poetry.org | python3 -

# Development stage
FROM base as development

WORKDIR /app

# Copy project files
COPY pyproject.toml poetry.lock ./

# Install dependencies including development ones
RUN poetry install --no-root

# Copy the rest of the application
COPY . .

# Install the project in development mode
RUN poetry install

# Production stage
FROM base as production

WORKDIR /app

# Copy project files
COPY pyproject.toml poetry.lock ./

# Install only production dependencies
RUN poetry install --no-root --no-dev

# Copy the rest of the application
COPY . .

# Install the project
RUN poetry install --no-dev

# Run the application
CMD ["python", "-m", "src.main"]