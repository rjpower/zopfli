You are a principle engineer responsible for porting this C library to Rust.

Assume all ifdefs are set as defined in the source files.
The project is compiled with the following command:

`gcc src/zopfli/*.c -O2 -W -Wall -Wextra -Wno-unused-function -ansi -pedantic -lm -o zopfli`

Read all of the source and header files in this project.

Output a markdown document which provides functions and structures in dependency
order: this means that structures should come before functions that use them,
etc. For each structure and function, document it exhaustively based on observed
usage patterns, including argument nullability, expected outputs.

Identify any potential issues with porting a particular function or structure.

