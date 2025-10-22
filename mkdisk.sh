#!/bin/bash

cargo build --release || exit 1

nasm -o boot.img -f bin boot.s || exit 1
tail -c +513 template.img >> boot.img

sudo mount boot.img /mnt
sudo objcopy -O binary target/x86-unknown-none/release/bootloader /mnt/stage2
sudo umount /mnt

[ "$1" == "run" ] && qemu-system-i386 -fda boot.img
