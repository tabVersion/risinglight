use super::*;
use crate::binder::{BoundBinaryOp, BoundExpr, BoundExpr::*};
use crate::parser::BinaryOperator::*;

/// Constant moving rule moves constants in the filtering conditions from one side to the other
/// side.
///
/// NOTICE: we don't process division as it is complicated.
/// x / 2 == 2 means x = 4 or x = 5 !!!
/// For example,
/// `select a from t where 100 + a > 300;`
/// The rule will convert it into
/// `select a from t where a > 200;`
pub struct ConstantMovingRule;

impl PlanRewriter for ConstantMovingRule {
    fn rewrite_expr(&mut self, expr: BoundExpr) -> BoundExpr {
        match &expr {
            BinaryOp(op) => match (&op.op, &*op.left_expr, &*op.right_expr) {
                (Eq | NotEq | Gt | Lt | GtEq | LtEq, BinaryOp(bin_op), Constant(rval)) => {
                    match (&bin_op.op, &*bin_op.left_expr, &*bin_op.right_expr) {
                        (Plus, other, Constant(lval)) | (Plus, Constant(lval), other) => {
                            BinaryOp(BoundBinaryOp {
                                op: op.op.clone(),
                                left_expr: other.clone().into(),
                                right_expr: Constant(rval - lval).into(),
                                return_type: op.return_type.clone(),
                            })
                        }
                        (Minus, other, Constant(lval)) => BinaryOp(BoundBinaryOp {
                            op: op.op.clone(),
                            left_expr: other.clone().into(),
                            right_expr: Constant(rval + lval).into(),
                            return_type: op.return_type.clone(),
                        }),
                        (Minus, Constant(lval), other) => BinaryOp(BoundBinaryOp {
                            op: op.op.clone(),
                            left_expr: Constant(lval - rval).into(),
                            right_expr: other.clone().into(),
                            return_type: op.return_type.clone(),
                        }),
                        (Multiply, other, Constant(lval)) | (Multiply, Constant(lval), other)
                            if lval.is_positive() && rval.is_divisible_by(lval) =>
                        {
                            BinaryOp(BoundBinaryOp {
                                // TODO: flip op when lval is negative
                                op: op.op.clone(),
                                left_expr: other.clone().into(),
                                right_expr: Constant(rval / lval).into(),
                                return_type: op.return_type.clone(),
                            })
                        }
                        _ => expr,
                    }
                }
                _ => expr,
            },
            _ => expr,
        }
    }
}
