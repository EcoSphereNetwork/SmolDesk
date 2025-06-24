FROM mcr.microsoft.com/vscode/devcontainers/python:3.11

# Prevent apt from showing prompts
ENV DEBIAN_FRONTEND=noninteractive

# Install Node.js and npm
RUN curl -fsSL https://deb.nodesource.com/setup_16.x | bash - \
    && apt-get update \
    && apt-get install -y nodejs \
    && npm install -g npm@latest

# Install Python dependencies
RUN pip install --no-cache-dir poetry pre-commit

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    make \
    curl \
    wget \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Setup working directory
WORKDIR /workspace

# Install global dev tools
RUN npm install -g \
    typescript \
    @docusaurus/core \
    @docusaurus/preset-classic

# Create non-root user
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && apt-get update \
    && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME

# Set the default user
USER $USERNAME