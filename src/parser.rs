use pest::iterators::Pair;
use pest_derive::Parser;

use crate::internal_representations::gast::{Expr, Function, Name, Program, Stmt};

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct Parser;

fn parse_expr3(code: Pair<Rule>) -> Option<Expr> {
    let mut iter = code.into_inner();
    let inner = iter.next()?;

    match inner.as_rule() {
        Rule::number => Some(Expr::IntLiteral(inner.as_str().parse().unwrap())),
        Rule::string => Some(Expr::StringLiteral(inner.as_str().to_string())),
        Rule::ident => Some(Expr::Name(Name::new(inner.as_str().to_string()))),
        Rule::expr => parse_expr(inner),
        _ => unreachable!(),
    }
}

fn parse_expr2(code: Pair<Rule>) -> Option<Expr> {
    let mut iter = code.into_inner();
    let inner = iter.next()?;

    match inner.as_rule() {
        Rule::expr3 => parse_expr3(inner),
        Rule::call_expr => {
            let mut iter = inner.into_inner();
            let name = Name::new(iter.next()?.as_str().to_string());
            let mut args = Vec::new();

            for pair in iter {
                args.push(parse_expr(pair)?);
            }

            Some(Expr::Call(name, args))
        }
        _ => unreachable!(),
    }
}

fn parse_expr1(code: Pair<Rule>) -> Option<Expr> {
    let mut iter = code.into_inner();
    let expr2 = parse_expr2(iter.next()?)?;
    let op = iter.next();

    Some(match op.map(|op| op.as_str()) {
        Some("*") => Expr::mul(expr2, parse_expr1(iter.next()?)?),
        Some("/") => Expr::div(expr2, parse_expr1(iter.next()?)?),
        None => expr2,
        _ => unreachable!(),
    })
}

fn parse_expr(code: Pair<Rule>) -> Option<Expr> {
    let mut iter = code.into_inner();
    let expr1 = parse_expr1(iter.next()?)?;
    let op = iter.next();

    Some(match op.map(|op| op.as_str()) {
        Some("+") => Expr::add(expr1, parse_expr(iter.next()?)?),
        Some("-") => Expr::sub(expr1, parse_expr(iter.next()?)?),
        None => expr1,
        _ => unreachable!(),
    })
}

fn parse_assign(code: Pair<Rule>) -> Option<Stmt> {
    let mut iter = code.into_inner();
    let ident = Name::new(iter.next()?.as_str().to_string());
    let expr = parse_expr(iter.next()?)?;
    Some(Stmt::Assign(ident, Box::new(expr)))
}

fn parse_let(code: Pair<Rule>) -> Option<Stmt> {
    let mut iter = code.into_inner();
    let ident = Name::new(iter.next()?.as_str().to_string());

    let expr = iter.next()?;

    Some(Stmt::Let(ident, parse_expr(expr)?))
}

fn parse_return(code: Pair<Rule>) -> Option<Stmt> {
    Some(Stmt::Return(parse_expr(code.into_inner().next()?)?))
}

fn parse_if(code: Pair<Rule>) -> Option<Stmt> {
    let mut iter = code.into_inner();
    let condition = parse_expr(iter.next()?)?;
    let body = parse_block(iter.next()?)?;
    let else_body = parse_block(iter.next()?)?;

    Some(Stmt::If(condition, body, else_body))
}

fn parse_for(code: Pair<Rule>) -> Option<Stmt> {
    let mut iter = code.into_inner();
    let stmt = parse_let(iter.next()?)?;
    let condition = parse_expr(iter.next()?)?;
    let step = parse_expr(iter.next()?)?;
    let body = parse_block(iter.next()?)?;

    Some(Stmt::For(Box::new(stmt), condition, step, body))
}

fn parse_statement(code: Pair<Rule>) -> Option<Stmt> {
    let expr = code.into_inner().next()?;
    match expr.as_rule() {
        Rule::expr => Some(Stmt::Expr(parse_expr(expr)?)),
        Rule::let_expr => parse_let(expr),
        Rule::return_expr => parse_return(expr),
        Rule::if_expr => parse_if(expr),
        Rule::for_expr => parse_for(expr),
        Rule::assign_expr => parse_assign(expr),
        _ => unreachable!("{:?}", expr),
    }
}

fn parse_block(code: Pair<Rule>) -> Option<Vec<Stmt>> {
    let mut stmts = Vec::new();

    for pair in code.into_inner() {
        let stmt = parse_statement(pair);
        stmts.push(stmt?);
    }

    Some(stmts)
}

fn parse_function(code: Pair<Rule>) -> Option<Function> {
    let mut iter = code.into_inner();

    let name = Name::new(iter.next()?.as_str().to_string());

    let mut args = Vec::new();

    let mut body = None;

    for pair in iter {
        if pair.as_rule() == Rule::block {
            body = Some(pair);
            break;
        }
        args.push(Name::new(pair.as_str().to_string()));
    }

    let body = parse_block(body.unwrap())?;

    Some(Function { name, args, body })
}

pub fn parse_program(code: Pair<Rule>) -> Option<Program> {
    let mut functions = Vec::new();

    for pair in code.into_inner() {
        if pair.as_rule() == Rule::func {
            functions.push(parse_function(pair)?);
        }
    }

    Some(Program { functions })
}
