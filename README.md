![Under construction GIF 1](http://textfiles.com/underconstruction/HeHeartland3460buttonsConstructani.gif)
![Under construction GIF 2](http://textfiles.com/underconstruction/3762construction.gif)


# STRM1
Very very **very** work in progress homebrew RISCy microprocessor architecture and codegen tooling.
This is a monorepo containing all the individual parts of the project.

Progress is slow, but maybe one day you can compile code through a custom compiler and run it on an FPGA implementation of the architecture. Just maybe, maybe not, we'll see.

## libstormir
libstormir is the most prominent and worked on thing here, it is/will be a compiler framework converting it's own intermediate representation language to machine code for the custom architecture.
The idea is similar to LLVM, just far simpler and worse.

I am aiming for libstormir to be target-extensible, allowing support for other targets with their own backends compiling IR to native code. Right now, this idea is far out of scope.

## The STRM1 architecture
As of writing, the architecture has nothing note-worthy, and only contains a few basic instructions. I'll develop the architecture further once libstormir can work with even these few basic instructions.
