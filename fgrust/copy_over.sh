#!/bin/sh

if [ "$1" = "--release" ]
then
    cp target/release/librgdext.so ../fgserver/bin/release/librgdext.so
    cp target/release/librgdext.so ../fgmapeditor/bin/release/librgdext.so
    cp target/release/librgdext_client.so ../fgclient/bin/release/librgdext_client.so
    cp target/x86_64-pc-windows-gnu/release/rgdext_client.dll ../fgclient/bin/release.dll
    cp target/release/librgdext_serverutil.so ../fgauth/bin/release/librgdext_serverutil.so
    cp target/release/librgdext_serverutil.so ../fggateway/bin/release/librgdext_serverutil.so
    cp target/release/librgdext_serverutil.so ../fgdatabase/bin/release/librgdext_serverutil.so
else
    cp target/debug/librgdext.so ../fgserver/bin/debug/librgdext.so
    cp target/debug/librgdext.so ../fgmapeditor/bin/debug/librgdext.so
    cp target/debug/librgdext_client.so ../fgclient/bin/debug/librgdext_client.so
    cp target/x86_64-pc-windows-gnu/release/rgdext_client.dll ../fgclient/bin/debug.dll
    cp target/debug/librgdext_serverutil.so ../fgauth/bin/debug/librgdext_serverutil.so
    cp target/debug/librgdext_serverutil.so ../fggateway/bin/debug/librgdext_serverutil.so
    cp target/debug/librgdext_serverutil.so ../fgdatabase/bin/debug/librgdext_serverutil.so
fi


