use crate::error::{Error, SyntaxError};
use crate::types::{Expr, Token, TokenKind};

/// ONO GRAMMAR ///
/// expression  -> logic_or
/// logic_or    -> logic_and ( "or" logic_and )* ;
/// logic_and   -> equality ( "and" equality )* ;
/// equality    -> comparison ( ("!=" | "==") comparison )* ;
/// comparison  -> term ( (">" | ">=" | "<" | "<=") term )* ;
/// term        -> factor ( ("-" | "+") factor )* ;
/// factor      -> unary ( ("/" | "*") unary )* ;
/// unary       -> ("!" | "-") unary | primary ;
/// primary     -> NUMBER | STRING | "true" | "false" | "null" | "(" expression ")" ;

/// Parses a Vec<Token> into an expression
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
        }
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Expr, Error> {
        self.tokens = tokens;
        self.expression()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn advance(&mut self) -> &Token {
        self.current += 1;
        self.previous()
    }

    fn consume(&mut self, kind: &TokenKind) -> Option<&Token> {
        if self.check(kind) {
            return Some(self.advance());
        }
        None
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EOF
    }

    fn check(&mut self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().kind.is_same(kind)
    }

    fn is_token_of_kind(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.logic_or()
    }

    fn logic_or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.logic_and()?;

        while self.is_token_of_kind(&[TokenKind::OR]) {
            let operator = self.previous();
            expr = Expr::Logical {
                operator: operator.clone(),
                left: Box::new(expr),
                right: Box::new(self.logic_and()?),
            }
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;

        while self.is_token_of_kind(&[TokenKind::OR]) {
            let operator = self.previous();
            expr = Expr::Logical {
                operator: operator.clone(),
                left: Box::new(expr),
                right: Box::new(self.equality()?),
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;
        while self.is_token_of_kind(&[TokenKind::BANGEQUAL, TokenKind::EQUALEQUAL]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.comparison()?),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;
        while self.is_token_of_kind(&[
            TokenKind::LESS,
            TokenKind::LESSEQUAL,
            TokenKind::GREATER,
            TokenKind::GREATEREQUAL,
        ]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.term()?),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;
        while self.is_token_of_kind(&[TokenKind::MINUS, TokenKind::PLUS]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.factor()?),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.is_token_of_kind(&[TokenKind::SLASH, TokenKind::STAR]) {
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(self.unary()?),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if self.is_token_of_kind(&[TokenKind::BANG, TokenKind::MINUS]) {
            return Ok(Expr::Unary {
                operator: self.previous().clone(),
                expr: Box::new(self.unary()?),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        if self.is_token_of_kind(&[
            TokenKind::FALSE,
            TokenKind::TRUE,
            TokenKind::NULL,
            TokenKind::NUMBER(1.0),
            TokenKind::STRING("".to_string()),
        ]) {
            return Ok(Expr::Literal {
                value: self.previous().clone(),
            });
        }

        if self.is_token_of_kind(&[TokenKind::LEFTPAREN]) {
            let opening_token = self.previous().clone();
            let expr = self.expression()?;

            return if self.consume(&TokenKind::RIGHTPAREN).is_none() {
                Err(Error::syntax_error(SyntaxError::S003, opening_token))
            } else {
                Ok(Expr::Group {
                    expr: Box::new(expr),
                })
            };
        }

        Err(Error::syntax_error(SyntaxError::S004, self.peek().clone()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::Position;
    use Expr::*;
    use TokenKind::*;

    // TODO: Test logical production
    // TODO: Test unary production
    // TODO: Test errors that might be produced

    #[test]
    fn parses_logical_expression() -> Result<(), Error> {
        let result = Parser::new().parse(vec![
            Token {
                kind: TRUE,
                lexeme: "true".to_string(),
                position: Position { line: 0, column: 4 },
            },
            Token {
                kind: AND,
                lexeme: "and".to_string(),
                position: Position { line: 0, column: 8 },
            },
            Token {
                kind: FALSE,
                lexeme: "false".to_string(),
                position: Position {
                    line: 0,
                    column: 14,
                },
            },
            Token {
                kind: EOF,
                lexeme: "\n".to_string(),
                position: Position { line: 1, column: 0 },
            },
        ])?;

        let target = Binary {
            operator: Token {
                kind: PLUS,
                lexeme: "+".to_string(),
                position: Position { line: 0, column: 3 },
            },
            left: Box::new(Literal {
                value: Token {
                    kind: NUMBER(1.0),
                    lexeme: "1".to_string(),
                    position: Position { line: 0, column: 1 },
                },
            }),
            right: Box::new(Literal {
                value: Token {
                    kind: NUMBER(2.0),
                    lexeme: "2".to_string(),
                    position: Position { line: 0, column: 5 },
                },
            }),
        };

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn parses_binary_expression() -> Result<(), Error> {
        let result = Parser::new().parse(vec![
            Token {
                kind: NUMBER(1.0),
                lexeme: "1".to_string(),
                position: Position { line: 0, column: 1 },
            },
            Token {
                kind: PLUS,
                lexeme: "+".to_string(),
                position: Position { line: 0, column: 3 },
            },
            Token {
                kind: NUMBER(2.0),
                lexeme: "2".to_string(),
                position: Position { line: 0, column: 5 },
            },
            Token {
                kind: EOF,
                lexeme: "\n".to_string(),
                position: Position { line: 1, column: 0 },
            },
        ])?;

        let target = Binary {
            operator: Token {
                kind: PLUS,
                lexeme: "+".to_string(),
                position: Position { line: 0, column: 3 },
            },
            left: Box::new(Literal {
                value: Token {
                    kind: NUMBER(1.0),
                    lexeme: "1".to_string(),
                    position: Position { line: 0, column: 1 },
                },
            }),
            right: Box::new(Literal {
                value: Token {
                    kind: NUMBER(2.0),
                    lexeme: "2".to_string(),
                    position: Position { line: 0, column: 5 },
                },
            }),
        };

        assert_eq!(result, target);
        Ok(())
    }
}
