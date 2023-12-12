pub mod ast;
pub mod backend;
pub mod cfg;
pub mod types;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub parser);
