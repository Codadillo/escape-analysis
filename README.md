## Usage
To compile a program, run `cargo run path_to_program`.  This will compile `path_to_program`, print out its SSA, and place its resultant C into a newly created `build/` directory. `build/` will contain a Makefile that can be used to build the C into an object file. Example programs can be found in the `inputs/` directory.
