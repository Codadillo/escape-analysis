## Usage
To compile a program, run `cargo run path_to_program`.  This will compile `path_to_program`, print out its SSA, render its dependency graphs to dotfiles in `renders/`, and place its resultant C into `build/`. `build/` will contain a Makefile that can be used to build the C into an object file. Example programs can be found in the `inputs/` directory. 

You will also probably have to run `git clone https://github.com/lalrpop/lalrpop` first. Or, you could try changing `Cargo.toml` to properly depend on the most recent release of `larlpop` and `lalrpop-util`, but that didn't work for me.
