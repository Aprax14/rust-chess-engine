pub mod evaluation;
pub mod static_evaluation;

pub fn position_from_args(args: std::env::Args) -> String {
    let mut position = String::new();
    let mut args = args;
    args.next();

    for arg in args {
        position.push_str(arg.trim());
        position.push_str(" ");
    }
    position.trim().to_owned()
}