# pi-image-builder-rs

Rust re-implementation of [my other pi-image-builder](https://github.com/LadySerena/pi-image-builder). The main
motivation for this was in my Go project to reduce the amount of `exec.cmd()` calls since those were a massive pain to
debug when things went wrong (looking at you lvm2 nonsense). So ideally there will be 0 shelling out in this and using
either nicely made crates or calling the C libraries for things I'm interacting with.

## building

`cargo build`

note this does require the nightly toolchain

## testing

lol

## error handling

#TODO

## Required C Libraries

- loopdev requires `libclang.so`
    - `sudo pacman -Sy clang`
- lvm-rs requires `libbd_lvm.so`
    - `sudo pacman -Sy libblockdev`