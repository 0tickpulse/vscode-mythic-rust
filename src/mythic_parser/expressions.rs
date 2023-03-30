use tower_lsp::lsp_types::{Position, Range};

use crate::utilities::positions_and_ranges::CustomRange;

use super::lexer::MythicToken;

pub trait ExprTrait {
    fn get_range(&self) -> CustomRange;
}

#[derive(Debug)]
pub struct SkillLine {
    mechanic: Box<GenericNameAndMlc>,
    targeter: Option<Box<Targeter>>,
    trigger: Option<Box<Trigger>>,
    conditions: Vec<InlineCondition>,
    chance: Option<Box<Chance>>,
    health_modifier: Option<Box<HealthModifier>>,
}
impl SkillLine {
    pub fn new(
        mechanic: Box<GenericNameAndMlc>,
        targeter: Option<Box<Targeter>>,
        trigger: Option<Box<Trigger>>,
        conditions: Vec<InlineCondition>,
        chance: Option<Box<Chance>>,
        health_modifier: Option<Box<HealthModifier>>,
    ) -> Self {
        Self {
            mechanic,
            targeter,
            trigger,
            conditions,
            chance,
            health_modifier,
        }
    }
}

#[derive(Debug)]
pub struct GenericString {
    pub tokens: Vec<MythicToken>,
}

impl GenericString {
    pub fn new(tokens: Vec<MythicToken>) -> Self {
        Self { tokens }
    }
}

#[derive(Debug)]
/// Generic container for a name and an optional MLC.
/// Useful for item configuartions, placeholder bits, skill mechanics, and more.
pub struct GenericNameAndMlc {
    name: GenericString,
    mlc: Option<Box<MlcContainer>>,
}

impl GenericNameAndMlc {
    pub fn new(name: GenericString, mlc: Option<Box<MlcContainer>>) -> Self {
        Self { name, mlc }
    }
}

#[derive(Debug)]
pub struct Targeter {
    at: MythicToken,
    name: MythicToken,
    mlc: Option<Box<MlcContainer>>,
}

impl ExprTrait for Targeter {
    fn get_range(&self) -> CustomRange {
        CustomRange::new(
            self.at.get_range().start,
            self.mlc.as_ref().unwrap().get_range().end,
        )
    }
}

impl Targeter {
    pub fn new(at: MythicToken, name: MythicToken, mlc: Option<Box<MlcContainer>>) -> Self {
        Self { at, name, mlc }
    }
}

#[derive(Debug)]
pub struct Trigger {
    caret: MythicToken,
    name: GenericString,
    colon: Option<MythicToken>,
    arg: Option<Box<GenericString>>,
}

impl Trigger {
    pub fn new(
        caret: MythicToken,
        name: GenericString,
        colon: Option<MythicToken>,
        arg: Option<Box<GenericString>>,
    ) -> Self {
        Self {
            caret,
            name,
            colon,
            arg,
        }
    }
}

#[derive(Debug)]
pub struct InlineCondition {
    question_mark: MythicToken,
    exclamation_mark: Option<MythicToken>,
    tilde: Option<MythicToken>,
    name: MythicToken,
    mlc: Option<Box<MlcContainer>>,
}

impl InlineCondition {
    pub fn new(
        question_mark: MythicToken,
        exclamation_mark: Option<MythicToken>,
        tilde: Option<MythicToken>,
        name: MythicToken,
        mlc: Option<Box<MlcContainer>>,
    ) -> Self {
        Self {
            question_mark,
            exclamation_mark,
            tilde,
            name,
            mlc,
        }
    }
}

#[derive(Debug)]
pub struct Chance {
    token: MythicToken,
}

impl Chance {
    pub fn new(token: MythicToken) -> Self {
        Self { token }
    }
}

#[derive(Debug)]
pub struct HealthModifier {
    operator: MythicToken,
    value: HealthModifierValueOrRange,
}

impl HealthModifier {
    pub fn new(operator: MythicToken, value: HealthModifierValueOrRange) -> Self {
        Self { operator, value }
    }
}

#[derive(Debug)]
pub enum HealthModifierValueOrRange {
    Value(HealthModifierValue),
    Range(HealthModifierValue, HealthModifierValue),
}

#[derive(Debug)]
pub enum HealthModifierValue {
    Absolute(MythicToken),
    /// First is the MythicToken, second is the percentage.
    Percentage(MythicToken, MythicToken),
}

#[derive(Debug)]
pub struct MlcContainer {
    left_brace: MythicToken,
    mlcs: Vec<Mlc>,
    right_brace: MythicToken,
}

impl ExprTrait for MlcContainer {
    fn get_range(&self) -> CustomRange {
        CustomRange::new(
            self.left_brace.get_range().start,
            self.right_brace.get_range().end,
        )
    }
}

impl MlcContainer {
    pub fn new(left_brace: MythicToken, mlcs: Vec<Mlc>, right_brace: MythicToken) -> Self {
        Self {
            left_brace,
            mlcs,
            right_brace,
        }
    }
}

#[derive(Debug)]
pub struct Mlc {
    key: MythicToken,
    equals: MythicToken,
    value: MlcValueContainer,
    semicolon: Option<MythicToken>,
}

impl Mlc {
    pub fn new(
        key: MythicToken,
        equals: MythicToken,
        value: MlcValueContainer,
        semicolon: Option<MythicToken>,
    ) -> Self {
        Self {
            key,
            equals,
            value,
            semicolon,
        }
    }
}

#[derive(Debug)]
pub enum MlcValueContainer {
    MlcValue(MlcValue),
    InlineSkill(InlineSkill),
}

#[derive(Debug)]
pub struct MlcValue {
    identifiers: Vec<MlcValueIdentifier>,
}

impl MlcValue {
    pub fn new(identifiers: Vec<MlcValueIdentifier>) -> Self {
        Self { identifiers }
    }
}

#[derive(Debug)]
pub enum MlcValueIdentifier {
    Identifiers(Vec<MythicToken>),
    Placeholder(Placeholder),
}

#[derive(Debug)]
pub struct Placeholder {
    left_angle_bracket: MythicToken,
    identifiers: Vec<GenericNameAndMlc>,
    dots: Vec<MythicToken>,
    right_angle_bracket: MythicToken,
}

impl Placeholder {
    pub fn new(
        left_angle_bracket: MythicToken,
        identifiers: Vec<GenericNameAndMlc>,
        dots: Vec<MythicToken>,
        right_angle_bracket: MythicToken,
    ) -> Self {
        Self {
            left_angle_bracket,
            identifiers,
            dots,
            right_angle_bracket,
        }
    }
}

#[derive(Debug)]
pub struct InlineSkill {
    left_square_bracket: MythicToken,
    skills: Vec<InlineSkillSkillContainer>,
}

impl InlineSkill {
    pub fn new(left_square_bracket: MythicToken, skills: Vec<InlineSkillSkillContainer>) -> Self {
        Self {
            left_square_bracket,
            skills,
        }
    }
}

#[derive(Debug)]
pub struct InlineSkillSkillContainer {
    dash: MythicToken,
    skill: SkillLine,
}

impl InlineSkillSkillContainer {
    pub fn new(dash: MythicToken, skill: SkillLine) -> Self {
        Self { dash, skill }
    }
}
