# Embedded Rust
Getting rust to work an the Blue Pill stm32 board

# Links
- https://rust-embedded.github.io/book/intro/index.html
- https://wiki.stm32duino.com/index.php?title=Blue_Pill
- https://os.phil-opp.com/freestanding-rust-binary/

# Steps Done
# Intsall boatloader with Stlink (probablyincomplete)
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
```
sudo udevadm control --reload-rules
```
## Install boatloader with USB to Serial adapter
Download boatloader tools https://github.com/rogerclarkmelbourne/Arduino_STM32/archive/master.zip  
Download the bootloader https://github.com/rogerclarkmelbourne/STM32duino-bootloader/blob/master/binaries/generic_boot20_pb12.bin?raw=true  
Flash the board with stm32flash:
- need USB to Serial Adapter
- connect GND and 3.3V Pins
- connect rx and tx to A9(rx1) and A10(tx1) in the "correct" way
  - that normaly means rx to tx and vice verca
  - but somtimes not -- depends on the adapter
- connect Boot0 Jumper to 0 (and Boot1 Jumper to 1)

```
Arduino_STM32-master/tools/linux64/stm32flash/stm32flash -w ./generic_boot20_pb12.bin -v -g 0x0 /dev/ttyUSB0
```

## setup rust
Install toolchain
```
rustup target add thumbv7m-none-eabi
```
Install cargo-binutils
```
cargo install cargo-binutils
rustup component add llvm-tools-preview
```
Setup Cargo 
```
cargo init --bin --edition 2018
```

get the cortex m quickstart files like described in the embedded rust book  
https://rust-embedded.github.io/book/start/qemu.html  
  
Build binary
```
cargo build --target thumbv7m-none-eabi
```

Test build
```
cargo readobj --bin embeddedRust --target thumbv7m-none-eabi -- -file-headers
```

Size of linker sections
```
cargo size --bin embeddedRust --target thumbv7m-none-eabi --release -- -A
```
