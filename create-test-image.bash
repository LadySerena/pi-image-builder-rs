#!/usr/bin/env bash

set -x

fallocate meow.img -l 2Gb
sudo losetup -Pf meow.img
device=$(losetup -J |jq --raw-output '.loopdevices | to_entries[] | select(.value["back-file"]|contains("img"))|.value.name')


sudo parted --script "$device" mklabel msdos
sudo parted --script "$device" mkpart primary fat32 0% 100M
sudo parted --script "$device" mkpart primary ext4 100M 100%
sudo parted --script "$device" set 2 lvm on
sudo pvcreate "${device}p2"