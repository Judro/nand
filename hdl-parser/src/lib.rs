use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, not},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};
use nom_locate::{locate, Located};

pub type ChipName<'s> = Located<&'s str, &'s str>;

pub type PinName<'s> = Located<&'s str, &'s str>;

pub struct Chip<'s> {
    pub name: ChipName<'s>,
}

#[derive(Debug, PartialEq)]
pub enum RhsConnector<'s> {
    Pin(&'s str),
    Potential(bool),
}

pub fn chip<'s>(_input: &'s str) -> IResult<&'s str, Chip<'s>> {
    todo!()
}

fn identifier(input: &str) -> IResult<&str, &str> {
    preceded(
        not(digit1),
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
    )(input)
}
pub fn chip_head(input: &str) -> IResult<&str, ChipName> {
    preceded(
        delimited(skip_white, tag("CHIP"), skip_white),
        locate(identifier),
    )(input)
}

pub fn chip_body(input: &str) -> IResult<&str, (Vec<PinName>, Vec<PinName>)> {
    delimited(
        preceded(skip_white, char('{')),
        tuple((pin_decl::<In>, pin_decl::<Out>)),
        preceded(skip_white, char('}')),
    )(input)
}

fn skip_white(input: &str) -> IResult<&str, &str> {
    multispace0(input)
}

fn pin_list(input: &str) -> IResult<&str, Vec<PinName>> {
    separated_list0(
        preceded(skip_white, char(',')),
        preceded(skip_white, locate(identifier)),
    )(input)
}

fn pin_decl<'s, P: Pin>(input: &'s str) -> IResult<&'s str, Vec<PinName>> {
    preceded(
        skip_white,
        delimited(tag(P::conn()), pin_list, preceded(skip_white, char(';'))),
    )(input)
}

fn bool(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false)))(input)
}

fn rhs_connector<'s>(input: &'s str) -> IResult<&'s str, RhsConnector<'s>> {
    alt((
        map(bool, RhsConnector::Potential),
        map(identifier, RhsConnector::Pin),
    ))(input)
}

fn connection(input: &str) -> IResult<&str, (PinName, RhsConnector)> {
    separated_pair(
        preceded(skip_white, locate(identifier)),
        preceded(skip_white, char('=')),
        preceded(skip_white, rhs_connector),
    )(input)
}

fn connection_list(input: &str) -> IResult<&str, Vec<(PinName, RhsConnector)>> {
    separated_list0(preceded(skip_white, char(',')), connection)(input)
}

struct In;
struct Out;

trait Pin {
    fn conn() -> &'static str;
}

impl Pin for In {
    fn conn() -> &'static str {
        "IN"
    }
}

impl Pin for Out {
    fn conn() -> &'static str {
        "OUT"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ident_0() {
        let (rem, res) = identifier("ALU_01!").unwrap();
        assert_eq!(rem, "!");
        assert_eq!(res, "ALU_01");
    }

    #[test]
    #[should_panic]
    fn ident_1() {
        let (_rem, _res) = identifier("99ALU_01!").unwrap();
    }

    #[test]
    fn ident_2() {
        let (rem, res) = identifier("_ALU_01 ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(res, "_ALU_01");
    }

    #[test]
    fn chip_head_0() {
        let (rem, res) = chip_head("\tCHIP ALU_01!").unwrap();
        assert_eq!(rem, "!");
        assert_eq!(*res, "ALU_01");
    }
    #[test]
    #[should_panic]
    fn chip_head_1() {
        let (_rem, _res) = chip_head("\tChip ALU_01!").unwrap();
    }

    #[test]
    fn chip_head_2() {
        let (rem, res) = chip_head("CHIPALU_01!").unwrap();
        assert_eq!(rem, "!");
        assert_eq!(*res, "ALU_01");
    }

    #[test]
    fn body_0() {
        let (rem, _res) = chip_body(" {\tIN;\nOUT; } ").unwrap();
        assert_eq!(rem, " ");
    }

    #[test]
    #[should_panic]
    fn body_1() {
        let (_rem, _res) = chip_body(" {  ").unwrap();
    }

    #[test]
    fn pin_list_0() {
        let (rem, res) = pin_list("a, b , c,d;").unwrap();
        assert_eq!(rem, ";");
        assert_eq!(*res[0], "a");
        assert_eq!(*res[1], "b");
        assert_eq!(*res[2], "c");
        assert_eq!(*res[3], "d");
    }

    #[test]
    fn pin_list_1() {
        let (rem, res) = pin_list("a, b  c,d;").unwrap();
        assert_eq!(*res[0], "a");
        assert_eq!(*res[1], "b");
        assert_eq!(rem, "  c,d;");
    }

    #[test]
    fn input_pin_0() {
        let (rem, res) = pin_decl::<In>("IN a, b , c,d; ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(*res[0], "a");
        assert_eq!(*res[1], "b");
        assert_eq!(*res[2], "c");
        assert_eq!(*res[3], "d");
    }

    #[test]
    #[should_panic]
    fn input_pin_1() {
        let (_rem, _res) = pin_decl::<In>("IN a, b , c,d").unwrap();
    }

    #[test]
    #[should_panic]
    fn input_pin_2() {
        let (_rem, _res) = pin_decl::<In>("OUT a, b , c,d").unwrap();
    }

    #[test]
    fn output_pin_0() {
        let (rem, res) = pin_decl::<Out>("OUT a, b , c,d; ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(*res[0], "a");
        assert_eq!(*res[1], "b");
        assert_eq!(*res[2], "c");
        assert_eq!(*res[3], "d");
    }

    #[test]
    fn rhs_connector_0() {
        let (rem, res) = rhs_connector("true ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(res, RhsConnector::Potential(true));
    }

    #[test]
    fn rhs_connector_1() {
        let (rem, res) = rhs_connector("false ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(res, RhsConnector::Potential(false));
    }

    #[test]
    fn rhs_connector_2() {
        let (rem, res) = rhs_connector("ab ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(res, RhsConnector::Pin("ab"));
    }

    #[test]
    fn connection_0() {
        let (rem, (a, b)) = connection(" a = b ").unwrap();
        assert_eq!(rem, " ");
        assert_eq!(*a, "a");
        assert_eq!(b, RhsConnector::Pin("b"));
    }
    #[test]
    #[should_panic]
    fn connection_1() {
        let (_rem, (_a, _b)) = connection(" a = 1 ").unwrap();
    }

    #[test]
    fn connection_list__0() {
        let (rem, list) = connection_list(" a = b , c = true, d= in ").unwrap();
        assert_eq!(rem, " ");
        todo!()
    }
}
