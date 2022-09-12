//! CLI for adding modules to a rust project.
use mkmod::result::Error;
use std::path::PathBuf;
use std::io;
use clap::{command, Arg, ArgAction, value_parser};

fn main() {
    let matches = command!()
        .arg(
            Arg::new("path")
                .value_parser(value_parser!(PathBuf))
                .help("Path to the module")
        )
        .arg(
            Arg::new("dir")
                .long("dir")
                .action(ArgAction::SetTrue)
                .help("Create module as a directory")
        )
        .arg(
            Arg::new("with_test")
                .long("no-test")
                .action(ArgAction::SetFalse)
                .help("Do not add a test file")
        )
        .arg(
            Arg::new("add_to_super")
                .long("no-add")
                .action(ArgAction::SetFalse)
                .help("Do not add module to super")
        )
        .arg(
            Arg::new("super_main")
                .long("main")
                .action(ArgAction::SetTrue)
                .help("Add module to main instead of lib (only applies when adding to super for the crate root)")
        )
        .arg(
            Arg::new("public")
                .long("private")
                .action(ArgAction::SetFalse)
                .help("Add module to super as private (only applies when adding to super)")
        )
        .get_matches();

    let name = matches.get_one::<PathBuf>("path").expect("`path` must be provided");
    let dir = matches.get_flag("dir");
    let with_test = matches.get_flag("with_test");
    let add_to_super = matches.get_flag("add_to_super");
    let super_main = matches.get_flag("super_main");
    let public = matches.get_flag("public");

    let res = mkmod::main(&name, dir, with_test, add_to_super, super_main, public);
    if res.is_ok() {
        return;
    }

    // output error message
    let err = res.unwrap_err();
    let err_msg;
    match err {
        Error::Io(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            err_msg = String::from("a file of that name already exists");
        },

        _ => panic!("An unhandled error ocurred: {:?}", err),
    }

    println!("An error ocurred: {err_msg}");
}
