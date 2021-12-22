use nom::error::VerboseError;
use nom::{
  bytes::complete::{tag},
};


type IResult<'a, I, O, E = &'a str> = nom::IResult<I, O, VerboseError<E>>;

pub struct Parser {

}


impl Parser {


  pub fn parse(src: &str) -> IResult<&str, &str> {
    let (input, o) = tag("#")(src)?;

    Ok((input, o))
  }
}