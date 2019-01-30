# Embedded Rust
Getting rust to work an the Blue Pill stm32 board

# Links
- https://rust-embedded.github.io/book/intro/index.html
- https://wiki.stm32duino.com/index.php?title=Blue_Pill

# Steps Done
```
  sudo apt install \
  gdb-multiarch \
  openocd \
  qemu-system-arm
```

Create file ``/etc/udev/rules.d/70-st-link.rules`` for permissions with this content:
```
# STM32F3DISCOVERY rev A/B - ST-LINK/V2\
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess"\
\
# STM32F3DISCOVERY rev C+ - ST-LINK/V2-1\
ATTRS{idVendor}=="0483", ATTRS{idProduct}=="374b", TAG+="uaccess"\
```

Update:
```plat
sudo udevadm control --reload-rules
```

Install rust toolchain
```
rustup target add thumbv7m-none-eabi
```

Download boatloader files https://github.com/rogerclarkmelbourne/Arduino_STM32/archive/master.zip