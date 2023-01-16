# Fullscale Sam Software
---
Software powering the SAM boards onboard Fullscale.

## Installation
---
The Fullscale SAM boards will primarily be running C/C++ code. As such, it is necessary to install the g++ compiler:

`sudo apt install g++`

Additionally, ensure that make is installed with:

`make -v`

This source code will be developed in and for a Linux system. Therefore, it is essential to either install a Linux distribution or develop in a WSL (Windows Subsystem for Linux).

## Running
---
Similar to MCFSV2, Fullscale's SAM board software will be compiled through a central Makefile in the bin directory. This means that all that needs to be done to compile is running `make` in the source directory.

## IDE Setup (VSCode)
---
The essentials for C++ development in VSCode can all be found in the [C/C++ Extension Pack](https://marketplace.visualstudio.com/items?itemName=ms-vscode.cpptools-extension-pack).

## Debugging
---
GDP can be used to debug C++ programs. Run the following snippet to be sure that it is installed on your system:

`sudo apt-get install gdb`

Once installed, it can be ran like such to debug a program:

`gdb executable_file`