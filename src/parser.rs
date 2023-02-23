use crate::error::{Error, SyntaxError};
use crate::types::{Expr, Stmt, Token, TokenKind};

/// ONO GRAMMAR ///
/// program     -> statement* EOF;
/// statement   -> exprStmt ;
/// exprStmt    -> expression ';' ;

/// expression  -> logic_or
/// logic_or    -> logic_and ( "or" logic_and )* ;
/// logic_and   -> equality ( "and" equality )* ;
/// equality    -> comparison ( ("!=" | "==") comparison )* ;
/// comparison  -> term ( (">" | ">=" | "<" | "<=") term )* ;
/// term        -> factor ( ("-" | "+") factor )* ;
/// factor      -> unary ( ("/" | "*") unary )* ;
/// unary       -> ("!" | "-") unary | primary ;
/// primary     -> NUMBER | STRING | "true" | "false" | "null" | tuple ;
/// tuple       -> "(" expression ( "," expression )* ")" ;

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

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Vec<Stmt>, Vec<Error>> {
        self.tokens = tokens;
        let mut errors = Vec::new();
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.statement() {
                Ok(statement) => {
                    statements.push(statement);
                }
                Err(error) => {
                    errors.push(error);
                    self.synchronize();
                }
            };
        }

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(statements)
        }
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        self.expression_statement()
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        if self.consume(&TokenKind::SEMICOLON).is_none() {
            return Err(Error::syntax_error(
                SyntaxError::S005(TokenKind::SEMICOLON),
                self.previous().clone(),
            ));
        }

        Ok(Stmt::Expression { expr })
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
            TokenKind::NUMBER(1.0),
            TokenKind::STRING("".to_string()),
        ]) {
            return Ok(Expr::Literal {
                value: self.previous().clone(),
            });
        }

        if self.is_token_of_kind(&[TokenKind::LEFTPAREN]) {
            return self.tuple();
        }

        Err(Error::syntax_error(
            SyntaxError::S004,
            self.previous().clone(),
        ))
    }

    fn tuple(&mut self) -> Result<Expr, Error> {
        if self.consume(&TokenKind::RIGHTPAREN).is_some() {
            return Ok(Expr::Tuple { inners: Vec::new() });
        }

        let opening_token = self.previous().clone();
        let mut inners = vec![self.expression()?];
        while let Some(_) = self.consume(&TokenKind::COMMA) {
            inners.push(self.expression()?);
        }

        if self.consume(&TokenKind::RIGHTPAREN).is_none() {
            return Err(Error::syntax_error(SyntaxError::S003, opening_token));
        }

        if inners.len() == 1 {
            Ok(Expr::Group {
                expr: Box::new(inners.pop().unwrap()),
            })
        } else {
            Ok(Expr::Tuple { inners })
        }
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current.max(1) - 1).unwrap()
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

    fn synchronize(&mut self) {
        // Skip ahead until the start of the next statement is found
        while !self.is_at_end() {
            match self.peek().kind {
                TokenKind::SEMICOLON => {
                    self.advance();
                    return;
                }
                // TokenKind::LET => {
                //     return;
                // }
                _ => self.advance(),
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use Expr::*;
    use TokenKind::*;

    #[test]
    fn tuple_unit() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(LEFTPAREN, 0, 0, "("),
            Token::new(RIGHTPAREN, 0, 1, ")"),
            Token::new(SEMICOLON, 0, 2, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = vec![Stmt::Expression {
            expr: Tuple { inners: vec![] },
        }];

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn logical_or() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(TRUE, 0, 0, "true"),
            Token::new(OR, 0, 5, "or"),
            Token::new(FALSE, 0, 8, "false"),
            Token::new(SEMICOLON, 0, 9, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = vec![Stmt::Expression {
            expr: Logical {
                operator: tokens.get(1).unwrap().clone(),
                left: Box::new(Literal {
                    value: tokens.get(0).unwrap().clone(),
                }),
                right: Box::new(Literal {
                    value: tokens.get(2).unwrap().clone(),
                }),
            },
        }];

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn logical_and() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(TRUE, 0, 0, "true"),
            Token::new(AND, 0, 5, "and"),
            Token::new(FALSE, 0, 9, "false"),
            Token::new(SEMICOLON, 0, 10, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = vec![Stmt::Expression {
            expr: Logical {
                operator: tokens.get(1).unwrap().clone(),
                left: Box::new(Literal {
                    value: tokens.get(0).unwrap().clone(),
                }),
                right: Box::new(Literal {
                    value: tokens.get(2).unwrap().clone(),
                }),
            },
        }];

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn comparison() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(NUMBER(1.0), 0, 0, "1"),
            Token::new(LESSEQUAL, 0, 2, "<="),
            Token::new(NUMBER(2.0), 0, 5, "2"),
            Token::new(SEMICOLON, 0, 6, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = vec![Stmt::Expression {
            expr: Binary {
                operator: tokens.get(1).unwrap().clone(),
                left: Box::new(Literal {
                    value: tokens.get(0).unwrap().clone(),
                }),
                right: Box::new(Literal {
                    value: tokens.get(2).unwrap().clone(),
                }),
            },
        }];

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn term() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(NUMBER(1.0), 0, 0, "1"),
            Token::new(PLUS, 0, 2, "+"),
            Token::new(NUMBER(2.0), 0, 4, "2"),
            Token::new(SEMICOLON, 0, 5, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = vec![Stmt::Expression {
            expr: Binary {
                operator: tokens.get(1).unwrap().clone(),
                left: Box::new(Literal {
                    value: tokens.get(0).unwrap().clone(),
                }),
                right: Box::new(Literal {
                    value: tokens.get(2).unwrap().clone(),
                }),
            },
        }];

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn factor() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(NUMBER(1.0), 0, 0, "1"),
            Token::new(STAR, 0, 2, "*"),
            Token::new(NUMBER(2.0), 0, 4, "2"),
            Token::new(SEMICOLON, 0, 5, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = vec![Stmt::Expression {
            expr: Binary {
                operator: tokens.get(1).unwrap().clone(),
                left: Box::new(Literal {
                    value: tokens.get(0).unwrap().clone(),
                }),
                right: Box::new(Literal {
                    value: tokens.get(2).unwrap().clone(),
                }),
            },
        }];

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn unary() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(MINUS, 0, 0, "-"),
            Token::new(NUMBER(1.0), 0, 2, "1"),
            Token::new(SEMICOLON, 0, 3, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = vec![Stmt::Expression {
            expr: Unary {
                operator: tokens.get(0).unwrap().clone(),
                expr: Box::new(Literal {
                    value: tokens.get(1).unwrap().clone(),
                }),
            },
        }];

        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn primary() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(NUMBER(1.0), 0, 0, "1"),
            Token::new(SEMICOLON, 0, 1, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = vec![Stmt::Expression {
            expr: Literal {
                value: tokens.get(0).unwrap().clone(),
            },
        }];
        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn primary_paren() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(LEFTPAREN, 0, 0, "("),
            Token::new(NUMBER(1.0), 0, 1, "1"),
            Token::new(RIGHTPAREN, 0, 2, ")"),
            Token::new(SEMICOLON, 0, 3, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone())?;
        let target = vec![Stmt::Expression {
            expr: Group {
                expr: Box::new(Literal {
                    value: tokens.get(1).unwrap().clone(),
                }),
            },
        }];
        assert_eq!(result, target);
        Ok(())
    }

    #[test]
    fn error_on_unclosed_paren() -> Result<(), Vec<Error>> {
        let tokens = vec![
            Token::new(LEFTPAREN, 0, 0, "("),
            Token::new(NUMBER(1.0), 0, 1, "1"),
            Token::new(SEMICOLON, 0, 3, ";"),
            Token::new(EOF, 1, 0, "\n"),
        ];

        let result = Parser::new().parse(tokens.clone());
        let target = Err(vec![Error::syntax_error(
            SyntaxError::S003,
            tokens.get(0).unwrap().clone(),
        )]);
        assert_eq!(result, target);
        Ok(())
    }
}
