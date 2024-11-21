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