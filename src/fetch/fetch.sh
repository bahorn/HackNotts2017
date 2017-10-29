NORMAL_TARGET=blah.nyc3.digitaloceanspaces.com
LEGIT_TARGET=bahorn.nyc3.digitaloceanspaces.com
UUID=$1
DATA=`curl --header "Host: $LEGIT_TARGET" https://$NORMAL_TARGET/dat/$UUID.blob 2>/dev/null`
# combine with tor here.
echo Bridge $DATA
