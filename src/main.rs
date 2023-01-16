use clap::Parser;
use nom::{
    bytes::complete::{tag, take_until},
    sequence::delimited,
    IResult,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    file: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();

    let Ok(file_content) = std::fs::read_to_string(&args.file) else {
        println!("Could not read file: '{}'", args.file.to_string_lossy());
        std::process::exit(-1);
    };

    println!("{:?}", parse_progam(&file_content));
}

#[derive(Debug)]
enum Intrinsics {
    If(String, String),
}

fn parse_if(input: &str) -> IResult<&str, Intrinsics> {
    let Ok((rest, _)) = tag::<&str, &str, nom::error::Error<&str>>("if ")(input) else {
        println!("expected 'if' at start of if expression");
        std::process::exit(-1);
    };

    let Ok((block, condition)) = take_until::<&str, &str, nom::error::Error<&str>>(" {")(rest) else {
        println!("Unable to parse if condition!");
        std::process::exit(-1);
    };

    let block = block.trim();
    let condition = condition.trim();

    let Ok((rest, parsed)) = delimited(
        tag::<&str, &str, nom::error::Error<&str>>("{"),
        take_until("}"),
        tag("}"),
    )(block) else {
        println!("Unable to parse block of condition!");
        std::process::exit(-1);
    };

    let block = parsed.trim();

    Ok((rest, Intrinsics::If(condition.to_owned(), block.to_owned())))
}

fn parse_progam(input: &str) -> IResult<&str, &str> {
    let a = parse_if(input);
    println!("{:?}", a);

    let t: IResult<&str, &str> = tag("if")(input);

    t
}
