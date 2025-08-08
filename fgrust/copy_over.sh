#!/bin/sh

if [ "$1" = "--release" ]
then
    cp target/release/librgdext_client.so ../../fgclient/bin/release/librgdext_client.so
    cp target/release/librgdext_serverutil.so ../../fgauth/bin/release/librgdext_serverutil.so
    cp target/release/librgdext_serverutil.so ../../fggateway/bin/release/librgdext_serverutil.so
    cp target/release/librgdext_serverutil.so ../../fgdatabase/bin/release/librgdext_serverutil.so
else
    cp target/debug/librgdext_client.so ../../fgclient/bin/debug/librgdext_client.so
    cp target/debug/librgdext_serverutil.so ../../fgauth/bin/debug/librgdext_serverutil.so
    cp target/debug/librgdext_serverutil.so ../../fggateway/bin/debug/librgdext_serverutil.so
    cp target/debug/librgdext_serverutil.so ../../fgdatabase/bin/debug/librgdext_serverutil.so
fi


