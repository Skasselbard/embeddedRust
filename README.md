![Rust on ARM](https://github.com/Skasselbard/embeddedRust/workflows/Rust%20on%20ARM/badge.svg)
# Embedded Rust
Getting rust to work an the Blue Pill stm32 board

# Links
- [The embedded rust book](https://rust-embedded.github.io/book/intro/index.html)
- [blue pill wiki page](https://wiki.stm32duino.com/index.php?title=Blue_Pill)
- [rust core basics](https://os.phil-opp.com/freestanding-rust-binary/)
- [embedded rust project and link collection](https://github.com/rust-embedded/awesome-embedded-rust)
- [gdb commands](https://darkdust.net/files/GDB%20Cheat%20Sheet.pdf)
- [ARM instruction set](http://www.peter-cockerell.net/aalp/html/ch-3.html)
- [rust blue pill setup instructions and helpful pages](https://github.com/lupyuen/stm32-blue-pill-rust)
- [(want to read)](http://blog.japaric.io/brave-new-io/)
- [openocd docu](http://openocd.org/doc-release/pdf/openocd.pdf)
- https://medium.com/@ly.lee/hosting-embedded-rust-apps-on-apache-mynewt-with-stm32-blue-pill-c86b119fe5f
- [rust os how to](https://os.phil-opp.com/)

# crates
- blue pill peripherals crate https://crates.io/crates/stm32f103xx
- stm32f1 crate https://github.com/stm32-rs/stm32-rs
- blue pill hal crate https://crates.io/crates/stm32f1xx-hal

# Board specs
- [data sheet](https://www.st.com/resource/en/datasheet/stm32f103c8.pdf)
- [reference manual](https://www.st.com/content/ccc/resource/technical/document/reference_manual/59/b9/ba/7f/11/af/43/d5/CD00171190.pdf/files/CD00171190.pdf/jcr:content/translations/en.CD00171190.pdf)
- [flash programming manual](https://www.st.com/content/ccc/resource/technical/document/programming_manual/10/98/e8/d4/2b/51/4b/f5/CD00283419.pdf/files/CD00283419.pdf/jcr:content/translations/en.CD00283419.pdf)
- [bootloader notes](https://www.st.com/content/ccc/resource/technical/document/application_note/b9/9b/16/3a/12/1e/40/0c/CD00167594.pdf/files/CD00167594.pdf/jcr:content/translations/en.CD00167594.pdf)
- start of flash 0x0800_0000
- start of sram 0x2000_0000

# Steps Done
# Install bootloader with Stlink (probably incomplete)
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

## Debug with ST-Link
- connect both Boot Jumper to 0

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

get the cortex m quick-start files like described in the embedded rust book  
https://github.com/rust-embedded/cortex-m-quickstart/blob/master/
  
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
