#!/bin/sh

getent group nanocl > /dev/null 2&>1

if [ $? -ne 0 ]; then
  addgroup -S nanocl -g $NANOCL_GID
fi

getent passwd nanocl > /dev/null 2&>1

if [ $? -ne 0 ]; then
  adduser -S nanocl -G nanocl -u $NANOCL_UID
  chown nanocl:nanocl -R /run/nanocl
  chmod -R 770 /run/nanocl
fi

sh -c "sleep 5 && chmod -R 770 /run/nanocl" &

exec runuser -u nanocl -g nanocl -- /usr/local/bin/nanocld $@
