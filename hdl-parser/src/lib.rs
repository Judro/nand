use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::{digit1, multispace0},
    combinator::not,
    sequence::{delimited, preceded},
    IResult,
};

pub struct Chip<'s> {
    pub name: &'s str,
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
pub fn chip_head(input: &str) -> IResult<&str, &str> {
    preceded(delimited(skip_white, tag("CHIP"), skip_white), identifier)(input)
}

fn skip_white(input: &str) -> IResult<&str, &str> {
    multispace0(input)
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
        assert_eq!(res, "ALU_01");
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
        assert_eq!(res, "ALU_01");
    }
}
