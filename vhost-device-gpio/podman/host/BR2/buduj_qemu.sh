#!/bin/bash
wget https://buildroot.org/downloads/buildroot-2023.11.1.tar.xz
tar -xJf buildroot-2023.11.1.tar.xz
cd buildroot-2023.11.1
cp ../br_config ./.config
make
