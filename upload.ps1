$config = Get-Content -Path .env.json -Raw | ConvertFrom-Json

scp -r -i $config.sshPrivateKeyPath ./target/aarch64-unknown-linux-gnu/debug/finale pi@rov.local:/home/pi/
ssh -i $config.sshPrivateKeyPath pi@rov.local "chmod +x /home/pi/finale"