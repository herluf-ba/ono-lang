use crate::error::{Error, SyntaxError, language_error};
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

        while self.is_token_of_kind(&[TokenKind::AND]) {
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
        Err(Error::syntax_error(SyntaxError::S004, self.previous().clone()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::Position;
    use Expr::*;
    use TokenKind::*;

    #[test]
    fn logical_or() -> Result<(), Error> {
        let tokens = vec![
            Token::new(TRUE, Position::new(0, 4), "true"),
            Token::new(OR, Position::new(0, 8), "or"),
            Token::new(FALSE, Position::new(0, 13), "false"),
            Token::new(EOF, Position::new(1, 0), "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = Logical {
            operator: tokens.get(1).unwrap().clone(),
            left: Box::new(Literal {
                value: tokens.get(0).unwrap().clone(),
            }),
            right: Box::new(Literal {
                value: tokens.get(2).unwrap().clone(),
            }),
        };

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn logical_and() -> Result<(), Error> {
        let tokens = vec![
            Token::new(TRUE, Position::new(0, 4), "true"),
            Token::new(AND, Position::new(0, 8), "and"),
            Token::new(FALSE, Position::new(0, 14), "false"),
            Token::new(EOF, Position::new(1, 0), "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = Logical {
            operator: tokens.get(1).unwrap().clone(),
            left: Box::new(Literal {
                value: tokens.get(0).unwrap().clone(),
            }),
            right: Box::new(Literal {
                value: tokens.get(2).unwrap().clone(),
            }),
        };

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn comparison() -> Result<(), Error> {
        let tokens = vec![
            Token::new(NUMBER(1.0), Position::new(0, 1), "1"),
            Token::new(LESSEQUAL, Position::new(0, 3), "<="),
            Token::new(NUMBER(2.0), Position::new(0, 6), "2"),
            Token::new(EOF, Position::new(1, 0), "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = Binary {
            operator: tokens.get(1).unwrap().clone(),
            left: Box::new(Literal {
                value: tokens.get(0).unwrap().clone(),
            }),
            right: Box::new(Literal {
                value: tokens.get(2).unwrap().clone(),
            }),
        };

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn term() -> Result<(), Error> {
        let tokens = vec![
            Token::new(NUMBER(1.0), Position::new(0, 1), "1"),
            Token::new(PLUS, Position::new(0, 3), "+"),
            Token::new(NUMBER(2.0), Position::new(0, 5), "2"),
            Token::new(EOF, Position::new(1, 0), "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = Binary {
            operator: tokens.get(1).unwrap().clone(),
            left: Box::new(Literal {
                value: tokens.get(0).unwrap().clone(),
            }),
            right: Box::new(Literal {
                value: tokens.get(2).unwrap().clone(),
            }),
        };

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn factor() -> Result<(), Error> {
        let tokens = vec![
            Token::new(NUMBER(1.0), Position::new(0, 1), "1"),
            Token::new(STAR, Position::new(0, 3), "*"),
            Token::new(NUMBER(2.0), Position::new(0, 5), "2"),
            Token::new(EOF, Position::new(1, 0), "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = Binary {
            operator: tokens.get(1).unwrap().clone(),
            left: Box::new(Literal {
                value: tokens.get(0).unwrap().clone(),
            }),
            right: Box::new(Literal {
                value: tokens.get(2).unwrap().clone(),
            }),
        };

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn unary() -> Result<(), Error> {
        let tokens = vec![
            Token::new(MINUS, Position::new(0, 1), "-"),
            Token::new(NUMBER(1.0), Position::new(0, 2), "1"),
            Token::new(EOF, Position::new(1, 0), "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = Unary {
            operator: tokens.get(0).unwrap().clone(),
            expr: Box::new(Literal {
                value: tokens.get(1).unwrap().clone(),
            }),
        };

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn primary() -> Result<(), Error> {
        let tokens = vec![
            Token::new(NUMBER(1.0), Position::new(0, 2), "1"),
            Token::new(EOF, Position::new(1, 0), "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = Literal {
            value: tokens.get(0).unwrap().clone(),
        };
        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn primary_paren() -> Result<(), Error> {
        let tokens = vec![
            Token::new(LEFTPAREN, Position::new(0, 1), "("),
            Token::new(NUMBER(1.0), Position::new(0, 2), "1"),
            Token::new(RIGHTPAREN, Position::new(0, 1), ")"),
            Token::new(EOF, Position::new(1, 0), "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = Group {
            expr: Box::new(Literal {
                value: tokens.get(1).unwrap().clone(),
            }),
        };
        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn error_on_unclosed_paren() -> Result<(), Error> {
        let tokens = vec![
            Token::new(LEFTPAREN, Position::new(0, 1), "("),
            Token::new(NUMBER(1.0), Position::new(0, 2), "1"),
            Token::new(EOF, Position::new(1, 0), "\n"),
        ];

        let result = Parser::new().parse(tokens.clone());
        let target = Err(Error::syntax_error(
            SyntaxError::S003,
            tokens.get(0).unwrap().clone(),
        ));
        assert_eq!(result, target);
        Ok(())
    }
}
