# Cthulhu

Industrial Grade™️ switch wiping and provisioning for events.

## Why?
Ever had to wipe/factoryreset 200+ switches in the span of a few hours? Multiple times per year?

Trying to do this with just a laptop and a serial cable is really tedious and slow. Even with a team of people.

## What?
This software suite can be used in combination with a Serial Console Server to automate the process of wiping / factory resetting managed network switches.
All you have to do is plug serial cables into the switches and apply power, the software does the rest.
When the port goes green in the dashboard, the switch is ready to be packed away.

### Only wiping?
Cthulhu can also provision switches.
The basic flow is that Cthulhu will fetch an IP using DHCP (on the management network interface) and then fetch a specified shell script.
Examples of these are in the contrib folder.

