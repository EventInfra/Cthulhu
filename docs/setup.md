# Cthulhu setup guide

## prerequisites

- Working mqtt server with anonymous users.
- A serial console server that supports rawTCP sockets or a host with tty serial
- A staging network where a webserver is running to provide the relevant staging
files if its used for staging. see the contrib dir for details

## Design

Each angel deamon handles one serial port, and each have each own config, for see below

```
log_dir = "/var/log/cthulhu/"
active_states = [
    "wipe",
    "provision",
]


[RawTCP]
endpoint = "10.200.0.10:6001"

[JobConfig]
provision_ping_target = "10.100.0.1"
juniper_provision_script_url = "http://10.100.0.1:4242/juniper/provision.sh"
arista_provision_script_url = "http://10.100.0.1:4242/arista/provision.sh"

[Heaven]
id = "PM01"
host = "127.0.0.1"
port = 1883
```
Each angel deamon has a uniq id, and the host and port is the mqtt server, this is mostly
for status monitoring from the web interface

### Heaven

Heaven is the webinterface and status dashboard, see `heaven.toml` for an example config

### cthulhu-netbox

cthulhu-netbox gives the option to report the status of a provisioning or wipe to netbox based
on the serial number of the device as a journal entry.
```
[NetBox]
url = "https://netbox.example.org/"
token = "<token>"

target_status = "staged"

[Heaven]
id = "N1"
host = "172.17.0.1"
port = 1883
```
