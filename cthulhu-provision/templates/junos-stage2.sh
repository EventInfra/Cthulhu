#!/bin/sh
FILE_LIST="autoreload.cron autoreload.sh"

CONFIG_FILE=/tmp/provision.config
load_file=/tmp/provision.load
check_file=/tmp/provision.check

pfail() {
        echo "[PROVISION-S2] Failure!"
        exit 1
}

echo '[PROVISION-S2] Provisioning start!'

find_version() {
  echo "show system snapshot media internal | find $1 | display xml" | cli | grep -v "version>ex-" | grep "package-version" | awk -F'>' '{print $2}' | awk -F'<' '{print $1}' | awk 'NR==1{print}'
}

echo "[PROVISION-S2] Gathering snapshot versions..."
PRIMARY_VERSION=`find_version primary`
echo "[PROVISION-S2] Primary OS: $PRIMARY_VERSION"
BACKUP_VERSION=`find_version backup`
echo "[PROVISION-S2] Backup OS: $BACKUP_VERSION"

if [ "x$PRIMARY_VERSION" != "x$BACKUP_VERSION" ]; then
  echo "[PROVISION-S2] Mismatched OS version on snapshots, correcting..."
  echo "request system snapshot media internal slice alternate" | cli
fi

echo "[PROVISION-S2] Disabling VC ports..."
echo "show virtual-chassis vc-port" | cli | grep -i Configured | (while read line ; do
    pic=`echo "$line" | awk '{ print $1 }' | awk -F/ '{ print $1 }'`
    port=`echo "$line" | awk '{ print $1 }' | awk -F/ '{ print $2 }'`
    echo "request virtual-chassis vc-port delete pic-slot $pic port $port" | cli
done)
echo "[PROVISION-S2] Attempting to configure VCP ports to act as network ports..."
echo "request virtual-chassis mode network-port" | cli
echo '[PROVISION-S2] VCP ports complete!'

for f in $FILE_LIST ; do
        echo "[PROVISION-S2] Retrieving $f..."
        fetch -o /var/tmp/$f "{{base_url}}/provision/juniper/assets/$f" || pfail
done
chmod +x /var/tmp/autoreload.sh

SERIALNO=`sysctl hw.chassis.serialid | awk '{ print $2 }'`
echo "[PROVISION-S2] Detected serial number: $SERIALNO"
echo "[PROVISION-S2] Fetching config file..."
fetch -o $CONFIG_FILE "{{base_url}}/configuration/juniper/$SERIALNO" || pfail
chmod 644 $CONFIG_FILE

echo "[PROVISION-S2] Checking config file..."
# check if the config has errors
{ echo "configure"
  echo "load override $CONFIG_FILE | save $load_file"
  echo "commit check | save $check_file"
  echo "rollback 0"
  echo "exit"
} | /usr/sbin/cli

if grep -q error $load_file; then
        echo "[PROVISION-S2] Failure in load file!"
        cat $load_file
        pfail
fi

if grep -q error $check_file; then
        echo "[PROVISION-S2] Failure in check file!"
        cat $check_file
        pfail
fi

echo "[PROVISION-S2] Applying config file..."

{ echo "configure"
  echo "load override $CONFIG_FILE | match IDONOTWANTYOUROUTPUT"
  echo "commit and-quit"
} | /usr/sbin/cli

echo "[PROVISION-S2] Saving rescue config..."
echo "request system configuration rescue save" | /usr/sbin/cli

echo "[PROVISION-S2] Enabling crontab..."
crontab /var/tmp/autoreload.cron
crontab -l

echo '[PROVISION-S2] Provisioning complete!'
