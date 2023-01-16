use clap::Parser;

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

    println!("{}", file_content);
}
