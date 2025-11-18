use crate::{
    generator::{generator::Generator, instruction::Instruction},
    lexer::tokens::{Ident, Literal},
    parser::{
        expr::{expr::CallExpr, infix::InfixOp, prefix::PrefixOp, Expr},
        node::Node,
    },
};

use Expr::*;

pub struct ExprGenerator;

impl ExprGenerator {
    fn generate_ident(node: &Node<Ident>, generator: &mut Generator) {
        if generator.is_global(node.id()) {
            let global = generator.get_global_mapping(node.id());
            generator.push_instruction(Instruction::LOAD_GLOBAL(global));
        } else {
            let local = generator.get_local_mapping(node.id());
            generator.push_instruction(Instruction::LOAD_LOCAL(local));
        }
    }

    fn generate_infix(lhs: &Expr, op: &InfixOp, rhs: &Expr, generator: &mut Generator) {
        Self::generate(lhs, generator);
        Self::generate(rhs, generator);

        generator.gen(op);
    }

    fn generate_prefix(op: &PrefixOp, rhs: &Expr, generator: &mut Generator) {
        Self::generate(rhs, generator);
        generator.gen(op);
    }

    fn generate_call(call: &Node<CallExpr>, generator: &mut Generator) {
        for ele in &call.args {
            Self::generate(ele, generator);
        }

        generator.push_instruction(Instruction::CALL(generator.get_call_mapping(call.id())));
    }

    fn generate_borrow(expr: &Expr, generator: &mut Generator) {
        let id = expr.lvalue().unwrap().id();
        let local = generator.get_local_mapping(id);
        generator.push_instruction(Instruction::PUSH_ADDR_LOCAL(local));
    }

    fn generate_lit(lit: &Node<Literal>, generator: &mut Generator) {
        let id = generator.get_const_mapping(lit.id());
        generator.push_instruction(Instruction::LOAD_CONST(id))
    }

    fn generate_box(expr: &Node<Expr>, generator: &mut Generator) {
        let id = generator.get_expr_type(expr.id());
        generator.gen_expr(expr);
        generator.push_instruction(Instruction::BOX_ALLOC(id));
    }

    pub fn generate(expr: &Expr, generator: &mut Generator) {
        match expr {
            Ident(i) => Self::generate_ident(i, generator),
            Infix(lhs, op, rhs) => Self::generate_infix(lhs, op, rhs, generator),
            Prefix(op, rhs) => Self::generate_prefix(op, rhs, generator),
            Call(call) => Self::generate_call(call, generator),
            Borrow(expr, _) => Self::generate_borrow(expr, generator),
            Literal(l) => Self::generate_lit(l, generator),
            Box(expr) => Self::generate_box(expr, generator),
        }
    }
}
