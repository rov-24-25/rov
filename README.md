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
8. Copy `init_rasp` folder to the Rasp itself and run `setup_rash.sh`, to prevent the Rasp from dropping the Ethernet connection after some inactivity.

## Setting up SSH on the Raspberry

This section explains how to use SSH keys to connect to the Raspberry and upload files to it without having to enter the password everytime.

### Generating a SSH key pair

First, you must generate a new SSH key pair using the command-line tool `ssh-keygen`.

From the command line, run `ssh-keygen`.

    PS C:\Users\johndoe> ssh-keygen
    Generating public/private ed25519 key pair.

Give the filename and file path you want when asked for it. **Remember you need to put the full path (and not just the filename).** I advise you to keep it in your home `.ssh` folder. I advise something like `C:\Users\johndoe\.ssh\rov_key`.

    Enter file in which to save the key (C:\Users\johndoe/.ssh/id_ed25519): C:\Users\johndoe\.ssh\rov_key

Then, press `Enter` twice **without typing anything** to specify no password.

You should now see:

    Enter passphrase (empty for no passphrase):
    Enter same passphrase again:
    Your identification has been saved in C:\Users\johndoe\.ssh\rov_key
    Your public key has been saved in C:\Users\johndoe\.ssh\rov_key.pub
    The key fingerprint is:
    SHA256:**** ***johndoe***@****
    The key's randomart image is:
    +--[ED25519 256]--+
    |      ...     .. |
    |      ...     .. |
    |      ...     .. |
    |      ...     .. |
    |      ...     .. |
    +----[SHA256]-----+

`ssh-keygen` just generated a new SSH key and it exited.

### Uploading the key to the Rasp

If we go to the folder we specified when generating our SSH key, we should see two files:

    PS C:\Users\johndoe> ls .\.ssh\

    Directory: C:\Users\johndoe\.ssh

    Mode                 LastWriteTime         Length Name
    ----                 -------------         ------ ----
    -a---           12/2/2024  1:20 PM            419 rov_key
    -a---           12/2/2024  1:20 PM            110 rov_key.pub

The `rov_key` file (or whatever you chose to name our SSH key) is your private key of your key pair, **it MUST REMAIN PRIVATE, which means it must not leave your computer.**

The `rov_key.pub` file is the public key of your key pair. It must be sent to the Rasp so that it can identify you.

To do so, open your newly-generated public key (`rov_key.pub`). It should contain only one line. Yours will be different, but mine is:

    ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICeUVX/mwM9Zf4WCUgyVNrNpjnMaVm5KVAqW77rh9KwI johndoe@****

You must now add this line to the `~/.ssh/authorized_keys` file **in the Rasp itself**.

To do so, first send the public key file to the Rasp, the location in the Rasp does not really matter.

    scp C:\Users\johndoe\.ssh\rov_key.pub pi@raspberrypi.local:/home/pi/rov_key.pub

(Modify this command so that the paths are correct on your computer.)

You will then need to type the SSH password.

Once this is done, you should see something like:

    rov_key.pub                                            100%  110     6.7KB/s   00:00

That means our public key is on the Rasp! But it is not at the correct location yet!

Let’s first connect to the Rasp using SSH and the password login.

    PS C:\Users\johndoe> ssh pi@raspberrypi.local

Then, we execute the command:

    pi@raspberrypi:~ $ cat rov_key.pub >> .ssh/authorized_keys

`cat rov_key.pub` returns the content of the file, and `>>` put thats content at the end of the file `authorized_keys` located in the `.ssh` folder.

Now, we can disconnect from the Rasp, then reconnect to it using SSH!

    PS C:\Users\johndoe> ssh -i .\.ssh\rov_key pi@raspberrypi.local
    
No more need to type the password everytime!

Not that the command to copy a file over SSH, `scp`, can also be run the same way:

    `scp -i .\.ssh\rov_key myfile.rs pi@raspberrypi.local:/home/pi/myfile.rs`

And you’re set! :) 

## Building

1. Adding the target `rustup target add …`
2. Adding the chmod