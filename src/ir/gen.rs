use super::{Codegen, IROp};

use crate::enviroment::Symbol;
use crate::parser::ast::{Expr, Ident, Node};
use crate::types::{AtomType, BasicType};

type IR = Vec<IROp>;
type IRRes = Result<IR, u8>;

pub trait IRGen {
    fn gen_prog(&mut self, exprs: Vec<Node>) -> IR;
    fn gen_func(&mut self, name: String, params: Vec<Ident>, ret: AtomType, body: Vec<Node>) -> IRRes;
    fn gen_extern(&mut self, name: String, params: Vec<Ident>, ret: AtomType) -> IRRes;

    fn gen_expr(&mut self, expr: Node) -> IRRes;

    fn gen_var_declare(&mut self, name: String, expr: Node) -> IRRes;
    fn gen_var_assign(&mut self, name: Node, expr: Node) -> IRRes;
    fn gen_binary_expr(&mut self, ty: AtomType, op: String, left: Node, right: Node) -> IRRes;
}

impl IRGen for Codegen {
    fn gen_prog(&mut self, exprs: Vec<Node>) -> IR {
        let mut gen = vec![];

        for expr in exprs {
            let compiled_expr = self.gen_expr(expr);
            if compiled_expr.is_ok() {
                gen.append(&mut compiled_expr.unwrap());
            }
        }
        gen
    }

    fn gen_func(
        &mut self,
        name: String,
        params: Vec<Ident>,
        ret: AtomType,
        body: Vec<Node>,
    ) -> IRRes {
        for param in &params {
            self.env.add(Symbol {
                name: param.val().clone(),
                ty: param.ty().clone(),
                refers_to_atom: false,
                value: None,
                expected: param.ty().clone(),
            });
        }
        let mut exprs = vec![];

        for expr in body {
            exprs.append(&mut self.gen_expr(expr)?);
        }

        Ok(vec![IROp::Def(ret, name, params, exprs)])
    }

    fn gen_expr(&mut self, expr: Node) -> IRRes {
        match expr.expr {
            Expr::Import { module, name, params } => {
                Ok(vec![IROp::Import(expr.ty, module, name, params)])
            }

            Expr::Func {
                ret,
                name,
                args,
                body,
            } => self.gen_func(name, args, ret, body),
            Expr::Extern { name, params } => {
                self.gen_extern(name.val().clone(), params, name.ty().clone())
            }

            Expr::Literal(lit) => Ok(vec![IROp::Const(lit)]),

            Expr::BinaryExpr { op, left, right } => {
                self.gen_binary_expr(expr.ty, op, *left, *right)
            }

            Expr::VarDeclare { name, val } => self.gen_var_declare(name.val().clone(), *val),
            Expr::VarAssign { name, val } => self.gen_var_assign(*name, *val),
            Expr::Ident(name) => Ok(vec![IROp::Load(expr.ty, name.val().clone())]),

            Expr::ListExpr(items) => {
                let mut bonded = vec![];
                for item in items {
                    bonded.push(self.gen_expr(item)?);
                }

                Ok(vec![IROp::List(expr.ty, bonded)])
            }

            Expr::MemberExpr { parent, child } => {
                let parent = self.gen_expr(*parent)?;
                let mut res = parent;
                res.push(IROp::LoadProp(expr.ty, child));

                Ok(res)
            }

            Expr::IndexExpr {
                parent: expr,
                index,
            } => {
                let expr = self.gen_expr(*expr)?;
                let idx = self.gen_expr(*index.clone())?;
                Ok([expr, idx, vec![IROp::LoadIdx(index.ty)]].concat())
            }

            Expr::FnCall { name, args } => {
                let mut res: Vec<IROp> = vec![];
                let count = args.len().clone() as u16;

                for arg in args {
                    res.append(&mut self.gen_expr(arg)?);
                }
                res.append(&mut self.gen_expr(*name)?);
                res.push(IROp::Call(expr.ty, count));

                Ok(res)
            }
            Expr::RetExpr(expr) => {
                let mut res = vec![];
                let mut compiled_expr = self.gen_expr(*expr.clone())?;

                res.append(&mut compiled_expr);
                res.push(IROp::Ret(expr.ty));
                Ok(res)
            }

            Expr::As(conv) => {
                let mut res = vec![];
                let mut inside = self.gen_expr(*conv.clone())?;

                res.append(&mut inside);
                res.push(IROp::Conv(expr.ty, (*conv).ty));
                Ok(res)
            }

            Expr::PosInfo(_, _, _) => Ok(vec![]),
            Expr::Discard(dis) => {
                let mut compiled = self.gen_expr(*dis.clone())?;
                if dis.ty != AtomType::Basic(BasicType::Void) {
                    compiled.append(&mut vec![IROp::Pop]);
                }
                Ok(compiled)
            }

            Expr::IfExpr {
                condition,
                body,
                alt,
            } => {
                let mut cond = self.gen_expr(*condition)?;

                let mut compiled_body = vec![];

                // TODO func which generates scope body
                self.env.child();
                for expr in body {
                    compiled_body.append(&mut self.gen_expr(expr)?);
                }
                for sym in self.env.symbols.values() {
                    compiled_body.push(IROp::Dealloc(sym.ty.clone(), sym.name.clone()));
                }
                self.env.parent();

                let alt = if alt.is_none() {
                    vec![]
                } else {
                    self.gen_expr(*alt.unwrap())?
                };

                let mut res = Vec::new();
                res.append(&mut cond);
                res.push(IROp::If(expr.ty, compiled_body, alt));
                Ok(res)
            }

            Expr::Block(block) => {
                let mut compiled_block = vec![];
                self.env.child();
                for expr in block {
                    compiled_block.append(&mut self.gen_expr(expr)?);
                }

                for sym in self.env.symbols.values() {
                    compiled_block.push(IROp::Dealloc(sym.ty.clone(), sym.name.clone()));
                }
                self.env.parent();
                Ok(compiled_block)
            }

            Expr::WhileExpr { condition, body } => {
                let mut cond = self.gen_expr(*condition)?;

                let mut compiled_body = vec![];
                self.env.child();
                for expr in body {
                    compiled_body.append(&mut self.gen_expr(expr)?);
                }

                for sym in self.env.symbols.values().clone() {
                    compiled_body.push(IROp::Dealloc(sym.ty.clone(), sym.name.clone()));
                }

                let mut res = Vec::new();
                res.append(&mut cond);
                res.push(IROp::While(compiled_body));

                self.env.parent();
                Ok(res)
            }
            _ => todo!("{:#?}", expr),
        }
    }

    fn gen_extern(&mut self, name: String, params: Vec<Ident>, ret: AtomType) -> IRRes {
        Ok(vec![IROp::Extern(ret, name, params)])
    }

    fn gen_var_declare(&mut self, name: String, expr: Node) -> IRRes {
        let mut res = vec![];
        let mut g = self.gen_expr(expr.clone())?;
        let ty = expr.ty;

        res.push(IROp::Alloc(ty.clone(), name.clone()));

        self.env.add(Symbol {
            name: name.clone(),
            ty: ty.clone(),
            refers_to_atom: false,
            value: None,
            expected: ty.clone(),
        });

        res.append(&mut g);
        res.push(IROp::Store(ty, name));

        Ok(res)
    }

    fn gen_var_assign(&mut self, name: Node, expr: Node) -> IRRes {
        let mut res = vec![];
        res.append(&mut self.gen_expr(name)?);
        let mut compiled_expr = self.gen_expr(expr.clone())?;
        let ty = expr.ty;

        res.append(&mut compiled_expr);
        res.push(IROp::Set(ty));
        Ok(res)
    }

    fn gen_binary_expr(&mut self, ty: AtomType, op: String, left: Node, right: Node) -> IRRes {
        let mut res: IR = vec![];
        let mut lhs = self.gen_expr(left.clone())?;
        let mut rhs = self.gen_expr(right)?;
        res.append(&mut rhs);
        res.append(&mut lhs);
        if op.as_str() == "<" || op.as_str() == "<=" {
            res.reverse();
        }
        res.append(&mut vec![match op.as_str() {
            "+" => IROp::Add(ty),
            "-" => IROp::Sub(ty),
            "*" => IROp::Mul(ty),
            "/" => IROp::Div(ty),
            "%" => IROp::Mod(ty),
            ">" | "<" => IROp::Comp,
            ">=" | "<=" => IROp::EComp,
            "==" => IROp::Eq,
            "&&" => IROp::And,
            "||" => IROp::Or,
            o => todo!("add op {}", o),
        }]);
        Ok(res)
    }
}
