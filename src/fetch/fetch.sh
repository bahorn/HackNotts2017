NORMAL_TARGET=blah.nyc3.digitaloceanspaces.com # This is what other people see
LEGIT_TARGET=bahorn.nyc3.digitaloceanspaces.com # only the host sees this.
UUID=$1
DATA=`curl --header "Host: $LEGIT_TARGET" https://$NORMAL_TARGET/dat/$UUID.blob 2>/dev/null`
# combine with tor here.
echo Bridge $DATA | cat torrc - >latest_torrc
