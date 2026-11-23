mod cli;
mod output;
mod prompt;
mod skill;

fn main() {
    let code = cli::run();
    std::process::exit(code);
}
