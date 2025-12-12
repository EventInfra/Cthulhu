#!/bin/bash
set -exuo pipefail
cd "$(dirname "$0")"
PACKAGES="cthulhu-angel cthulhu-heaven cthulhu-provision cthulhu-netbox octhulhu-agent"

for pkg in $PACKAGES; do
	cargo deb -p $pkg
done
