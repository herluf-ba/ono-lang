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

    pub fn parse(&mut self, tokens: Vec<Token>) -> Result<Expr, Error> {
        self.tokens = tokens;
        let result = self.expression();
        self.tokens.clear();
        result
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

    // expression -> equality
    fn expression(&mut self) -> Result<Expr, Error> {
        self.equality()
    }

    // equality -> comparison ( ("!=" | "==") comparison )* ;
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;
        while self.is_token_of_kind(&[TokenKind::BANGEQUAL, TokenKind::EQUALEQUAL]) {
            let right = self.comparison()?;
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(right),
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
            let right = self.term()?;
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(right),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // term -> factor ( ("-" | "+") factor )* ;
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;
        while self.is_token_of_kind(&[TokenKind::MINUS, TokenKind::PLUS]) {
            let right = self.factor()?;
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(right),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // factor -> unary ( ("/" | "*") unary )* ;
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.is_token_of_kind(&[TokenKind::SLASH, TokenKind::STAR]) {
            let right = self.unary()?;
            expr = Expr::Binary {
                operator: self.previous().clone(),
                right: Box::new(right),
                left: Box::new(expr),
            }
        }
        Ok(expr)
    }

    // unary -> ("!" | "-") unary | primary ;
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.is_token_of_kind(&[TokenKind::BANG, TokenKind::MINUS]) {
            let inner = self.unary()?;
            return Ok(Expr::Unary {
                operator: self.previous().clone(),
                expr: Box::new(inner),
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
