# Rov

This is the binary that is supposed to continually run on the Raspberry embedded in the ROV.

## Setting up Raspbian

1. Install Raspberry using the Raspberry installer. Enable SSH with a password. Enter the details for the user. Set host name.
2. `ssh` the Raspberry.
3. `sudo apt update`
4. `sudo apt full-upgrade -y`
5. `sudo apt install git`
6. Generate deploy keys.
7. `git clone https://github.com/rov-24-25/rov.git`
8. `sudo ethtool --set-eee eth0 eee off` To prevent the Rasp from dropping the Ethernet connection after some inactivity.

## Setting up SSH on the Raspberry

1. Generate a private/public key pair: `ssh-keygen`
2. Add the public key as the last line of `~/.ssh/authorized_hosts` of the Raspberry
3. Connect to the Raspberry using `-i {path_to_private_key}`

## Building

1. Adding the target `rustup target add …`
2. Adding the chmod