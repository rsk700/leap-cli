use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use clap::{App, Arg, ArgMatches, SubCommand};
use leap_lang::{formatter, parser::parser::Parser};

// todo: report unwrap
pub fn run() {
    let args = cli_args();
    match args.subcommand() {
        ("format", Some(args)) => format_command(args),
        ("verify", Some(args)) => verify_command(args),
        _ => {}
    }
}

pub fn cli_args() -> ArgMatches<'static> {
    let spec_arg = Arg::with_name("spec")
        .multiple(true)
        .required(true)
        .help("Files containing Leap specs");
    let format_sub_command = SubCommand::with_name("format").arg(&spec_arg).arg(
        Arg::with_name("stdout")
            .long("stdout")
            .short("s")
            .takes_value(false)
            .help("Print output to stdout"),
    );
    let verify_sub_command = SubCommand::with_name("verify").arg(&spec_arg);
    let app_title = format!("Leap Language CLI v{}", env!("CARGO_PKG_VERSION"));
    App::new(app_title)
        .subcommand(format_sub_command)
        .subcommand(verify_sub_command)
        .get_matches()
}

fn get_backup_path(path: &Path) -> PathBuf {
    let name = path.file_name().unwrap().to_owned();
    let mut parent = path.to_owned();
    parent.pop();
    for n in 1..100 {
        let suffix = OsString::from(format!("_backup{}", n));
        let mut backup_name = name.clone();
        backup_name.push(suffix);
        let mut backup_path = parent.clone();
        backup_path.push(backup_name);
        if !backup_path.exists() {
            return backup_path;
        }
    }
    panic!("can't find path for backup");
}

// todo: return Enum(Ok, Fail)
fn format_command(args: &ArgMatches) {
    let paths = args.values_of("spec").unwrap();
    let to_stdout = args.is_present("stdout");
    for path in paths {
        // todo: catch unwrap
        let path = fs::canonicalize(path).unwrap();
        let data = fs::read_to_string(&path);
        let formatted = match data {
            // todo: report error on failed format
            Ok(s) => formatter::format(&s).unwrap(),
            Err(e) => panic!("{}", e),
        };
        if to_stdout {
            print!("{}", formatted);
        } else {
            // we was able to read file - path is correct
            let backup_path = get_backup_path(&path);
            // create backup to prevent data loss, if something will happen during writing formatted data
            fs::rename(&path, &backup_path).unwrap();
            fs::write(&path, formatted).unwrap();
            // formatted data already saved, we can delete backup now
            fs::remove_file(&backup_path).unwrap();
        }
    }
}

// todo: return Enum(Ok, Fail)
fn verify_command(args: &ArgMatches) {
    let paths = args.values_of("spec").unwrap();
    let result = Parser::parse_paths_iter(paths);
    if let Err(e) = result {
        // todo: return non zero exit code on programm close
        println!("{}", e.error_report());
    }
}
