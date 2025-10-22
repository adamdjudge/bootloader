#!/bin/bash

BUILD=release

if [ $BUILD == "release" ]; then
    cargo build --release || exit 1
elif [ $BUILD == "debug" ]; then
    cargo build || exit 1
fi

nasm -o boot.img -f bin boot.s || exit 1
tail -c +513 template.img >> boot.img

sudo mount boot.img /mnt
sudo objcopy -O binary target/x86-unknown-none/$BUILD/bootloader /mnt/stage2
ls -lh /mnt/stage2
sudo umount /mnt

[ "$1" == "run" ] && qemu-system-i386 -fda boot.img
