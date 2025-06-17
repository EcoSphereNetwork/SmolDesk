#!/bin/bash
# Simple helper to install SmolDesk build dependencies on Debian/Ubuntu systems
set -e

sudo apt update
sudo apt install -y build-essential libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev librsvg2-dev pkg-config

npm install
