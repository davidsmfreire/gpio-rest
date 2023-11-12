#!/bin/sh

sudo mv ops/gpio-rest.service /lib/systemd/system/
sudo systemctl enable --now gpio-rest
