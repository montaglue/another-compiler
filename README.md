# Another compiler
Just slowly developed pet project for 

## Structure
I use pest for parsing and inkwell for work with llvm.

Grammar can vary over time, but main theme would be the same.

All internal representation like control flow graph or additional ast's will appear in 
src/internal_representations folder.

## Goals 
- [x] basic math operations
- [ ] standard input and output
- [ ] compilation modules
- [ ] primitive types
- [ ] inner abstract syntax tree


## Resources
- https://mapping-high-level-constructs-to-llvm-ir.readthedocs.io/en/latest/a-quick-primer/index.html
- https://mukulrathi.com/create-your-own-programming-language/intro-to-compiler/
- https://buildmedia.readthedocs.org/media/pdf/mapping-high-level-constructs-to-llvm-ir/latest/mapping-high-level-constructs-to-llvm-ir.pdf