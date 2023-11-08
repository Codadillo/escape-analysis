pub mod ast;
pub mod cfg;
pub mod annotate;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub parser);
