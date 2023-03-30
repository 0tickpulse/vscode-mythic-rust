use crate::errors::error_registry::{
    Error, SyntaxError, TargeterAlreadyDefinedError, TriggerAlreadyDefinedError,
};

use super::{
    expressions::{
        Chance, GenericNameAndMlc, GenericString, HealthModifier, HealthModifierValue,
        HealthModifierValueOrRange, InlineCondition, Mlc, MlcContainer, MlcValue,
        MlcValueContainer, MlcValueIdentifier, Placeholder, SkillLine, Targeter, Trigger, InlineSkill, InlineSkillSkillContainer, ExprTrait,
    },
    lexer::{MythicToken, TokenType},
};

pub struct Parser {
    current: usize,
    tokens: Vec<MythicToken>,
    result: Vec<MythicToken>,
    source: String,
}

impl Parser {
    pub fn new(result: Vec<MythicToken>, source: String) -> Self {
        Self {
            current: 0,
            tokens: result,
            result: Vec::new(),
            source,
        }
    }
    pub fn parse(&mut self) -> Result<SkillLine, Error> {
        self.skill_line(Vec::new())
    }
    fn skill_line(&mut self, exit_types: Vec<TokenType>) -> Result<SkillLine, Error> {
        let mechanic = self.generic_name_and_mlc()?;
        let mut targeter: Option<Box<Targeter>> = None;
        let mut trigger: Option<Box<Trigger>> = None;
        let mut conditions: Vec<InlineCondition> = Vec::new();
        let mut chance: Option<Box<Chance>> = None;
        let mut health_modifier: Option<Box<HealthModifier>> = None;

        while !self.is_at_end() {
            self.consume_whitespace();
            if self.is_at_end() {
                break;
            }
            if self.match_all(vec![TokenType::At]) {
                if targeter.is_some() {
                    return Err(
                        TargeterAlreadyDefinedError::new(targeter.unwrap().get_range()).to_error(),
                    );
                }
                targeter = Some(Box::new(self.targeter()?));
            } else if self.match_all(vec![TokenType::Tilde]) {
                if trigger.is_some() {
                    return Err(
                        TriggerAlreadyDefinedError::new(targeter.unwrap().get_range()).to_error(),
                    );
                }
                trigger = Some(Box::new(self.trigger()?));
            } else if self.match_all(vec![TokenType::Question]) {
                conditions.push(self.inline_condition()?);
            } else if self.match_all(vec![TokenType::Percent]) {
                chance = Some(Box::new(Chance::new(self.previous().to_owned())));
            } else if self.check_any(vec![
                TokenType::LessThan,
                TokenType::GreaterThan,
                TokenType::Equal,
            ]) {
                health_modifier = Some(Box::new(self.health_modifier()?));
            } else if self.check_any(exit_types) {
                break;
            } else {
                return Err(SyntaxError::new(
                    self.peek().get_range(),
                    String::from("Expected a valid skill line modifier"),
                )
                .to_error());
            }
        }

        Ok(SkillLine::new(
            Box::new(mechanic),
            targeter,
            trigger,
            conditions,
            chance,
            health_modifier,
        ))
    }
    fn generic_name_and_mlc(&mut self) -> Result<GenericNameAndMlc, Error> {
        let name = self.generic_string(
            vec![TokenType::LeftBrace, TokenType::Space],
            Some(String::from("Expected mechanic name!")),
        )?;
        if name.tokens.len() == 0 {
            return Err(SyntaxError::new(
                self.peek().get_range(),
                String::from("Expected mechanic name!"),
            )
            .to_error());
        }
        if self.check(TokenType::LeftBrace) {
            let mlc = self.mlc()?;
            Ok(GenericNameAndMlc::new(name, Some(Box::new(mlc))))
        } else {
            Ok(GenericNameAndMlc::new(name, None))
        }
    }
    fn targeter(&mut self) -> Result<Targeter, Error> {
        let at = self.previous().to_owned();
        let name = self.consume(
            TokenType::Identifier,
            Some(String::from("Expected targeter name!")),
        )?;
        if self.check(TokenType::LeftBrace) {
            let mlc = self.mlc()?;
            Ok(Targeter::new(at, name, Some(Box::new(mlc))))
        } else {
            Ok(Targeter::new(at, name, None))
        }
    }
    fn trigger(&mut self) -> Result<Trigger, Error> {
        let caret = self.previous().to_owned();
        let name = self.generic_string(
            vec![TokenType::LeftBrace, TokenType::Space],
            Some(String::from("Expected a trigger name!")),
        )?;
        let arg: Option<GenericString>;
        let colon: Option<MythicToken>;
        if self.match_all(vec![TokenType::Colon]) {
            colon = Some(self.previous().to_owned());
            arg = Some(self.generic_string(
                vec![TokenType::LeftBrace, TokenType::Space],
                Some(String::from("Expected a trigger argument after ':'!")),
            )?);
        } else {
            colon = None;
            arg = None;
        }
        Ok(Trigger::new(caret, name, colon, arg.map(|x| Box::new(x))))
    }
    fn inline_condition(&mut self) -> Result<InlineCondition, Error> {
        let question = self.previous().to_owned();
        let mut not = false;
        let mut trigger = false;
        let exclam: Option<MythicToken> = None;
        let mut tilde: Option<MythicToken> = None;
        for _ in 0..2 {
            if self.check_any(vec![TokenType::Exclamation, TokenType::Tilde]) {
                if self.match_all(vec![TokenType::Exclamation]) {
                    if exclam.is_some() {
                        return Err(SyntaxError::new(
                            self.peek().get_range(),
                            String::from("Duplicate inline condition trigger symbol!"),
                        )
                        .to_error());
                    }
                    not = true;
                }
                if self.match_all(vec![TokenType::Tilde]) {
                    if tilde.is_some() {
                        return Err(SyntaxError::new(
                            self.peek().get_range(),
                            String::from("Duplicate inline condition trigger symbol!"),
                        )
                        .to_error());
                    }
                    tilde = Some(self.previous().to_owned());
                    trigger = true;
                }
            }
        }
        let name = self.consume(
            TokenType::Identifier,
            Some(String::from("Expected inline condition name!")),
        )?;
        if self.check(TokenType::LeftBrace) {
            let mlc = self.mlc()?;
            Ok(InlineCondition::new(question, exclam, tilde, name, Some(Box::new(mlc))))
        } else {
            Ok(InlineCondition::new(question, exclam, tilde, name, None))
        }
    }
    fn health_modifier(&mut self) -> Result<HealthModifier, Error> {
        let operator = self.consume_any(
            vec![
                TokenType::Equal,
                TokenType::GreaterThan,
                TokenType::LessThan,
            ],
            Some(String::from("Expected health modifier operator!")),
        )?;
        let min = self.consume(
            TokenType::Number,
            Some(String::from("Expected health modifier value!")),
        )?;
        let mut min_value = HealthModifierValue::Absolute(min.clone());
        if self.matches(TokenType::Percent) {
            min_value = HealthModifierValue::Percentage(min, self.previous().to_owned())
        }
        if operator.type_ == TokenType::Equal && self.match_all(vec![TokenType::Dash]) {
            let max = self.consume(
                TokenType::Number,
                Some(String::from("Expected second health modifier value!")),
            )?;
            let mut max_value = HealthModifierValue::Absolute(max.clone());
            if self.check(TokenType::Percent) {
                max_value = HealthModifierValue::Percentage(max, self.previous().to_owned())
            }
            Ok(HealthModifier::new(
                operator,
                HealthModifierValueOrRange::Range(min_value, max_value),
            ))
        } else {
            Ok(HealthModifier::new(
                operator,
                HealthModifierValueOrRange::Value(min_value),
            ))
        }
    }

    fn generic_string(
        &mut self,
        end: Vec<TokenType>,
        error: Option<String>,
    ) -> Result<GenericString, Error> {
        let start = self.current;
        while !self.check_any(end.clone()) && !self.is_at_end() {
            if self.check(TokenType::LeftBrace) {
                while !self.check(TokenType::RightBrace) {
                    self.advance();
                }
            }
            if self.check(TokenType::LeftSquareBracket) {
                while !self.check(TokenType::RightSquareBracket) {
                    self.advance();
                }
            }
            self.advance();
        }
        let string = self.tokens[start..self.current].to_vec();
        if string.len() == 0 && error.is_some() {
            return Err(SyntaxError::new(self.peek().get_range(), error.unwrap()).to_error());
        }
        Ok(GenericString::new(string))
    }
    fn mlc(&mut self) -> Result<MlcContainer, Error> {
        let left_brace = self.consume(
            TokenType::LeftBrace,
            Some(String::from("Expected '{' before mlc!")),
        )?;
        let mut mlcs: Vec<Mlc> = vec![];
        loop {
            let mut semicolon: Option<MythicToken> = None;
            let previous_lexeme = (&self.previous()).lexeme.clone().unwrap_or(String::from(""));
            if previous_lexeme == ";" {
                semicolon = Some(self.previous().to_owned());
            }
            if self.check(TokenType::RightBrace) {
                break;
            }
            self.consume_whitespace();
            let key = self.consume(
                TokenType::Identifier,
                Some(String::from("Expected mlc key!")),
            )?;
            // self.completion_generic(vec![TokenType::Equal]);
            let equals = self.consume(
                TokenType::Equal,
                Some(String::from("Expected '=' after mlc key!")),
            )?;
            let value = self.mlc_value()?;
            mlcs.push(Mlc::new(
                key,
                equals,
                MlcValueContainer::MlcValue(value),
                semicolon,
            ));
            // self.completion_generic(vec![TokenType::Semicolon, TokenType::RightBrace]);
            self.consume_whitespace();
            // self.completion_generic(vec![TokenType::Semicolon, TokenType::RightBrace]);
            self.consume_whitespace();
            if !self.match_all(vec![TokenType::Semicolon]) {
                break;
            }
        }
        let right_brace = self.consume(
            TokenType::RightBrace,
            Some(String::from("Expected '}' after mlc!")),
        )?;
        Ok(MlcContainer::new(left_brace, mlcs, right_brace))
    }
    fn mlc_value(&mut self) -> Result<MlcValue, Error> {
        let mut parts: Vec<MlcValueIdentifier> = vec![];
        let mut start = self.current;
        while !self.check_any(vec![TokenType::Semicolon, TokenType::RightBrace])
            && !self.is_at_end()
        {
            if self.match_all(vec![TokenType::LessThan]) {
                parts.push(MlcValueIdentifier::Identifiers(
                    self.tokens[start..self.current - 1].to_vec(),
                ));
                parts.push(MlcValueIdentifier::Placeholder(self.placeholder()?));
                start = self.current;
            } else if self.match_all(vec![TokenType::LeftBrace]) {
                while !self.match_all(vec![TokenType::RightBrace]) {
                    self.advance();
                }
            } else if self.match_all(vec![TokenType::LeftSquareBracket]) {
                while !self.match_all(vec![TokenType::RightSquareBracket]) {
                    self.advance();
                }
            } else {
                self.advance();
            }
        }
        parts.push(MlcValueIdentifier::Identifiers(
            self.tokens[start..self.current].to_vec(),
        ));
        Ok(MlcValue::new(parts))
    }
    fn placeholder(&mut self) -> Result<Placeholder, Error> {
        let left_square_bracket = self.consume(
            TokenType::LeftSquareBracket,
            Some(String::from("Expected '[' before placeholder!")),
        )?;
        let mut parts: Vec<GenericNameAndMlc> = vec![];
        let mut dots: Vec<MythicToken> = vec![];
        let part = self.generic_name_and_mlc()?;
        parts.push(part);
        // self.completion_generic(vec![TokenType::Dot, TokenType::GreaterThan]);
        while self.match_all(vec![TokenType::Dot]) && !self.is_at_end() {
            dots.push(self.previous().to_owned());
            let part = self.generic_name_and_mlc()?;
            parts.push(part);
            // self.completion_generic(vec![TokenType::Dot, TokenType::GreaterThan]);
        }
        let right_square_bracket = self.consume(
            TokenType::GreaterThan,
            Some(String::from("Expected '>' after placeholder!")),
        )?;
        Ok(Placeholder::new(
            left_square_bracket,
            parts,
            dots,
            right_square_bracket,
        ))
    }
    // typescript:
    // #inlineSkill() {
    //     const leftSquareBracket = this.#previous();
    //     const dashesAndSkills: [MythicToken, SkillLineExpr][] = [];
    //     while (!this.#check("RightSquareBracket") && !this.#isAtEnd()) {
    //         this.#completionGeneric(["- ", "]"]);
    //         // optional whitespace
    //         this.#consumeWhitespace();
    //         this.#completionGeneric(["- ", "]"]);
    //         // dash
    //         const dash = this.#consume("Dash", "Expected '-' after '['!");

    //         // optional whitespace
    //         this.#consumeWhitespace();
    //         // skill
    //         const skill = this.#skillLine("RightSquareBracket", "Dash");

    //         // optional whitespace
    //         this.#consumeWhitespace();
    //         dashesAndSkills.push([dash, skill]);
    //     }
    //     const rightSquareBracket = this.#consume("RightSquareBracket", "Expected ']' after inline skill!");
    //     return new InlineSkillExpr(this, this.#currentPosition(), leftSquareBracket, dashesAndSkills, rightSquareBracket);
    // }
    fn inline_skill(&mut self) -> Result<InlineSkill, Error> {
        let left_square_bracket = &self.previous().clone();
        let mut dashes_and_skills: Vec<InlineSkillSkillContainer> = vec![];
        while !self.check(TokenType::RightSquareBracket) && !self.is_at_end() {
            // self.completion_generic(vec![TokenType::Dash, TokenType::RightSquareBracket]);
            // optional whitespace
            let _ = &self.consume_whitespace();
            // self.completion_generic(vec![TokenType::Dash, TokenType::RightSquareBracket]);
            // dash
            let dash = self.consume(
                TokenType::Dash,
                Some(String::from("Expected '-' after '['!")),
            )?;
            // optional whitespace
            self.consume_whitespace();
            // skill
            let skill = self.skill_line(
                vec![TokenType::RightSquareBracket, TokenType::Dash],
            )?;
            // optional whitespace
            self.consume_whitespace();
            dashes_and_skills.push(InlineSkillSkillContainer::new(
                dash,
                skill,
            ));
        }
        let right_square_bracket = self.consume(
            TokenType::RightSquareBracket,
            Some(String::from("Expected ']' after inline skill!")),
        )?;
        Ok(InlineSkill::new(
            left_square_bracket.to_owned(),
            dashes_and_skills,
        ))
    }
    fn consume_whitespace(&mut self) {
        while !self.matches(TokenType::Space) {}
    }
    fn matches(&mut self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.peek().type_ != type_ {
            return false;
        }
        self.advance();
        true
    }
    fn match_all(&mut self, types: Vec<TokenType>) -> bool {
        for type_ in types {
            if !self.check(type_) {
                return false;
            }
            self.advance();
        }
        true
    }
    fn check(&self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().type_ == type_
    }
    fn check_any(&self, types: Vec<TokenType>) -> bool {
        for type_ in types {
            if self.check(type_) {
                return true;
            }
        }
        false
    }
    fn consume_any(
        &mut self,
        types: Vec<TokenType>,
        error: Option<String>,
    ) -> Result<MythicToken, Error> {
        for type_ in types {
            if self.check(type_) {
                return Ok(self.advance().to_owned());
            }
        }
        if error.is_some() {
            return Err(SyntaxError::new(self.peek().get_range(), error.unwrap()).to_error());
        }
        Err(SyntaxError::new(self.peek().get_range(), String::from("Unexpected token!")).to_error())
    }
    fn consume(&mut self, type_: TokenType, error: Option<String>) -> Result<MythicToken, Error> {
        if self.check(type_) {
            return Ok(self.advance().to_owned());
        }
        if error.is_some() {
            return Err(SyntaxError::new(self.peek().get_range(), error.unwrap()).to_error());
        }
        Err(SyntaxError::new(self.peek().get_range(), String::from("Unexpected token!")).to_error())
    }
    fn advance(&mut self) -> &MythicToken {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    fn is_at_end(&self) -> bool {
        self.peek().type_ == TokenType::Eof
    }
    fn peek(&self) -> &MythicToken {
        &self.tokens[self.current]
    }
    fn previous(&self) -> &MythicToken {
        &self.tokens[self.current - 1]
    }
}
