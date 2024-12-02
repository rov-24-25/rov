#!/bin/sh

sudo mv ./eee-disabling-eth0.service /etc/systemd/system/eee-disabling-eth0.service
sudo mv ./eee-disabling-eth0.sh /usr/local/sbin/eee-disabling-eth0.sh
sudo systemctl enable eee-disabling-eth0.service
sudo systemctl start eee-disabling-eth0.service