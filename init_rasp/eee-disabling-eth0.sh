#!/bin/sh
while true
do
	/sbin/ethtool --set-eee eth0 eee off > /dev/null 2>&1
	sleep 1
done