#!/bin/bash
set -euo pipefail

sudo apt-get -qq update

sudo apt-get install -y \
  libasound-dev \
  libopus-dev
