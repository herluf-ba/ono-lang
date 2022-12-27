use crate::ast::*;
use crate::error::{Error, ErrorKind};
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

// A recursive descent parser that produces an AST
impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
        }
    }

    pub fn reset(&mut self) {
        self.tokens.clear();
        self.current = 0;
    }

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Vec<Stmt>, Error> {
        self.tokens = tokens;

        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.statement() {
                Ok(statement) => {
                    statements.push(statement);
                }
                Err(error) => {
                    self.reset();
                    return Err(error);
                }
            };
        }

        self.reset();
        Ok(statements)
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

    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.is_token_of_kind(&[TokenKind::PRINT]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        if self.consume(&TokenKind::SEMICOLON).is_none() {
            println!("{:?}", self.peek());

            return Err(Error::from_token(
                self.peek(),
                ErrorKind::SyntaxError,
                "Expected ';' before here",
            ));
        }

        Ok(Stmt::Expression(expr))
    }

    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        if self.consume(&TokenKind::SEMICOLON).is_none() {
            return Err(Error::from_token(
                self.peek(),
                ErrorKind::SyntaxError,
                "Expected ';' before here",
            ));
        }

        Ok(Stmt::Print(expr))
    }
    // expression -> equality
    fn expression(&mut self) -> Result<Expr, Error> {
        self.equality()
    }

    // equality -> comparison ( ("!=" | "==") comparison )* ;
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

    // comparison -> term ( (">" | ">=" | "<" | "<=") term )* ;
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

    // term -> factor ( ("-" | "+") factor )* ;
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

    // factor -> unary ( ("/" | "*") unary )* ;
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

    // unary -> ("!" | "-") unary | primary ;
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.is_token_of_kind(&[TokenKind::BANG, TokenKind::MINUS]) {
            return Ok(Expr::Unary {
                operator: self.previous().clone(),
                expr: Box::new(self.unary()?),
            });
        }
        self.primary()
    }

    // primary -> NUMBER | STRING | "true" | "false" | "null" | "(" expression ")" ;
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
                Err(Error::from_token(
                    &opening_token,
                    ErrorKind::SyntaxError,
                    "Expected ')' closing this",
                ))
            } else {
                Ok(Expr::Group {
                    expr: Box::new(expr),
                })
            };
        }

        Err(Error::from_token(
            self.previous(),
            ErrorKind::SyntaxError,
            "Expected expression",
        ))
    }
}
