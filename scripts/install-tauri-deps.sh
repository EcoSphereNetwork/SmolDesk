#!/usr/bin/env bash
set -e
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.0-dev \
  libjavascriptcoregtk-4.0-dev \
  libsoup2.4-dev \
  libglib2.0-dev

