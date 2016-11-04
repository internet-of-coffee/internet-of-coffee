#!/bin/sh

SYSROOT=

export PKG_CONFIG_DIR=
export PKG_CONFIG_LIBDIR=${SYSROOT}/usr/lib/pkgconfig:${SYSROOT}/usr/share/pkgconfig:/usr/local/opt/openssl/lib/pkgconfig
export PKG_CONFIG_SYSROOT_DIR=${SYSROOT}
export PKG_CONFIG_ALLOW_CROSS=1
export CPPFLAGS=-I/usr/local/opt/openssl/include
export CFLAGS=-I/usr/local/opt/openssl/include
export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"
cargo build
