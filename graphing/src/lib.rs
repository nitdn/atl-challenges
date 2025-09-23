use std::sync::OnceLock;

use pest::iterators::Pairs;

use pest::pratt_parser::PrattParser;

pub mod ui_state;

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f32),
    UnaryMinus(Box<Expr>),
    VarX,
    BinOp {
        lhs: Box<Self>,
        op: Op,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    FloorDiv,
    Pow,
    Log,
}

impl Expr {
    // This improved version handles constant folding more thoroughly.
    pub fn fold_constants(&self) -> Expr {
        match self {
            Expr::Number(_) => self.clone(),
            Expr::VarX => self.clone(),

            Expr::UnaryMinus(expr) => {
                let folded_expr = expr.fold_constants();
                if let Expr::Number(val) = folded_expr {
                    Expr::Number(-val)
                } else {
                    Expr::UnaryMinus(Box::new(folded_expr))
                }
            }

            Expr::BinOp { lhs, op, rhs } => {
                // First, recursively fold the left and right sides.
                let folded_lhs = lhs.fold_constants();
                let folded_rhs = rhs.fold_constants();

                // Now, check the result of the folding.
                match (&folded_lhs, &folded_rhs) {
                    (Expr::Number(lval), Expr::Number(rval)) => {
                        // Case 1: Both sides are numbers. Perform the calculation.
                        let result = match op {
                            Op::Add => lval + rval,
                            Op::Subtract => lval - rval,
                            Op::Multiply => lval * rval,
                            Op::Divide => lval / rval,
                            Op::Modulo => lval % rval,
                            Op::FloorDiv => (lval / rval).floor(),
                            Op::Pow => lval.powf(*rval),
                            Op::Log => lval.log(*rval),
                        };
                        Expr::Number(result)
                    }
                    (Expr::Number(lval), _) => {
                        // Case 2: Left side is a number. For commutative ops, we can rearrange.
                        match op {
                            Op::Add | Op::Multiply => Expr::BinOp {
                                lhs: Box::new(folded_rhs),
                                op: op.clone(),
                                rhs: Box::new(Expr::Number(*lval)),
                            },
                            _ => Expr::BinOp {
                                lhs: Box::new(folded_lhs),
                                op: op.clone(),
                                rhs: Box::new(folded_rhs),
                            },
                        }
                    }
                    (_, Expr::Number(rval)) => {
                        // Case 3: Right side is a number. No rearrangement needed.
                        Expr::BinOp {
                            lhs: Box::new(folded_lhs),
                            op: op.clone(),
                            rhs: Box::new(Expr::Number(*rval)),
                        }
                    }
                    _ => {
                        // Case 4: Neither side is a number. Reconstruct the BinOp.
                        Expr::BinOp {
                            lhs: Box::new(folded_lhs),
                            op: op.clone(),
                            rhs: Box::new(folded_rhs),
                        }
                    }
                }
            }
        }
    }
}

#[derive(pest_derive::Parser)]
#[grammar = "calculator.pest"]
pub struct ExprParser;

static PRATT_PARSER: OnceLock<PrattParser<Rule>> = OnceLock::new();

pub fn init_pratt() -> PrattParser<Rule> {
    use Rule::*;
    use pest::pratt_parser::{Assoc::*, Op};

    // Precedence is defined lowest to highest
    PrattParser::new()
        // Addition and subtract have equal precedence
        .op(Op::infix(add, Left) | Op::infix(subtract, Left))
        .op(Op::infix(multiply, Left)
            | Op::infix(divide, Left)
            | Op::infix(modulo, Left)
            | Op::infix(floor_division, Left))
        .op(Op::prefix(unary_minus))
        .op(Op::infix(pow, Left))
        .op(Op::infix(log, Left))
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .get_or_init(init_pratt)
        .map_primary(|primary| match primary.as_rule() {
            Rule::number => Expr::Number(primary.as_str().parse::<f32>().unwrap()),
            Rule::var_x => Expr::VarX,
            Rule::expr => parse_expr(primary.into_inner()),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => Op::Add,
                Rule::subtract => Op::Subtract,
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                Rule::modulo => Op::Modulo,
                Rule::floor_division => Op::FloorDiv,
                Rule::pow => Op::Pow,
                Rule::log => Op::Log,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::unary_minus => Expr::UnaryMinus(Box::new(rhs)),
            _ => unreachable!(),
        })
        .parse(pairs)
}

pub fn inorder_eval(expr: &Expr, var_x: f32) -> f32 {
    match expr {
        Expr::Number(i) => *i,
        Expr::VarX => var_x,
        Expr::BinOp { lhs, op, rhs } => {
            let lhs = inorder_eval(lhs, var_x);
            let rhs = inorder_eval(rhs, var_x);
            match op {
                Op::Add => lhs + rhs,
                Op::Subtract => lhs - rhs,
                Op::Multiply => lhs * rhs,
                Op::Divide => lhs / rhs,
                Op::Modulo => lhs % rhs,
                Op::FloorDiv => lhs.div_euclid(rhs),
                Op::Pow => lhs.powf(rhs),
                Op::Log => lhs.log(rhs),
            }
        }
        Expr::UnaryMinus(expr) => -inorder_eval(expr, var_x),
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use super::*;

    #[test]
    fn eval_parse_tree() {
        let input = "2 + 3.5 * ( 2 + 3)";
        let pairs = ExprParser::parse(Rule::equation, input)
            .unwrap()
            .next()
            .unwrap()
            .into_inner();
        let expr = &parse_expr(pairs);
        assert_eq!(inorder_eval(expr, 1.0), 19.5)
    }

    #[test]
    fn all_necessary_operators() {
        let ops = ["+", "-", "*", "/", "%", "//", "^", "log"];
        for op in ops {
            let input = format!("2 {op} 3");
            let pairs = ExprParser::parse(Rule::equation, &input)
                .unwrap()
                .next()
                .unwrap()
                .into_inner();
            let expr = &parse_expr(pairs);
            inorder_eval(expr, 0.0);
        }
    }

    #[test]
    fn test_unary_minus() {
        let input = "-3";
        let pairs = ExprParser::parse(Rule::equation, input)
            .unwrap()
            .next()
            .unwrap()
            .into_inner();
        let expr = &parse_expr(pairs);
        assert_eq!(inorder_eval(expr, 0.0), -3.0)
    }

    #[test]
    fn eval_with_x() {
        let x = 2.0;

        let input = "2 + 3 * ( 2 - 3) * x";
        let pairs = ExprParser::parse(Rule::equation, input)
            .unwrap()
            .next()
            .unwrap()
            .into_inner();
        let mut expr = parse_expr(pairs);
        expr.simplify();
        assert_eq!(inorder_eval(&expr, x), -4.0)
    }
}
