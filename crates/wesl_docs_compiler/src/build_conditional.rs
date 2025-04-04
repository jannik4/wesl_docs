use crate::map;
use wesl::syntax;
use wesl_docs::*;

pub struct ConditionalScope {
    prev: Vec<Conditional>,
}

impl ConditionalScope {
    pub fn new() -> Self {
        ConditionalScope { prev: Vec::new() }
    }
}

pub fn build_conditional(
    scope: &mut ConditionalScope,
    attributes: &[syntax::Attribute],
) -> Option<Conditional> {
    for attr in attributes {
        match attr {
            syntax::Attribute::If(spanned) => {
                let this = conditional_from_expr(spanned.node())?;
                scope.prev.clear();
                scope.prev.push(this.clone());
                return Some(this);
            }
            syntax::Attribute::Elif(spanned) => {
                let this = conditional_from_expr(spanned.node())?;
                let combined = scope.prev.iter().fold(this.clone(), |acc, c| {
                    Conditional::And(
                        Box::new(acc),
                        Box::new(Conditional::Not(Box::new(c.clone()))),
                    )
                });
                scope.prev.push(this);
                return Some(combined);
            }
            syntax::Attribute::Else => {
                return scope
                    .prev
                    .drain(..)
                    .map(|c| Conditional::Not(Box::new(c)))
                    .reduce(|a, b| Conditional::And(Box::new(a), Box::new(b)));
            }
            _ => (),
        }
    }

    scope.prev.clear();
    None
}

fn conditional_from_expr(expr: &syntax::Expression) -> Option<Conditional> {
    match expr {
        syntax::Expression::Literal(lit) => match lit {
            syntax::LiteralExpression::Bool(true) => Some(Conditional::True),
            syntax::LiteralExpression::Bool(false) => Some(Conditional::False),
            _ => {
                println!(
                    "Warning: Unsupported literal type for conditional: {:?}",
                    lit
                );
                None
            }
        },
        syntax::Expression::Parenthesized(paren) => conditional_from_expr(paren.expression.node()),
        syntax::Expression::Unary(unary) => match unary.operator {
            syntax::UnaryOperator::LogicalNegation => Some(Conditional::Not(Box::new(
                conditional_from_expr(unary.operand.node())?,
            ))),
            _ => {
                println!(
                    "Warning: Unsupported unary operator for conditional: {:?}",
                    unary.operator
                );
                None
            }
        },
        syntax::Expression::Binary(binary) => match binary.operator {
            syntax::BinaryOperator::ShortCircuitOr => Some(Conditional::Or(
                Box::new(conditional_from_expr(binary.left.node())?),
                Box::new(conditional_from_expr(binary.right.node())?),
            )),
            syntax::BinaryOperator::ShortCircuitAnd => Some(Conditional::And(
                Box::new(conditional_from_expr(binary.left.node())?),
                Box::new(conditional_from_expr(binary.right.node())?),
            )),
            _ => {
                println!(
                    "Warning: Unsupported binary operator for conditional: {:?}",
                    binary.operator
                );
                None
            }
        },
        syntax::Expression::TypeOrIdentifier(type_or_ident) => {
            if type_or_ident.template_args.is_some() {
                println!(
                    "Warning: Template arguments are not supported in conditionals: {:?}",
                    type_or_ident
                );
                None
            } else {
                Some(Conditional::Feature(map(&type_or_ident.ident)))
            }
        }
        _ => {
            println!(
                "Warning: Unsupported expression type for conditional: {:?}",
                expr
            );
            None
        }
    }
}
