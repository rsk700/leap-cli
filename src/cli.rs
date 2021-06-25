use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use clap::{App, Arg, ArgMatches, SubCommand};
use leap_lang::formatter;

// todo: report unwrap
pub fn run() {
    let args = cli_args();
    match args.subcommand() {
        ("format", Some(args)) => format_command(args),
        // todo: verify (parse, show errors)
        _ => {}
    }
}

pub fn cli_args() -> ArgMatches<'static> {
    // todo: backup, format, delete backup
    let format_sub_command = SubCommand::with_name("format")
        .arg(
            Arg::with_name("spec")
                .multiple(true)
                .required(true)
                .help("Files containing Leap specs"),
        )
        .arg(
            Arg::with_name("stdout")
                .long("stdout")
                .short("s")
                .takes_value(false)
                .help("Print output to stdout"),
        );
    let app_title = format!("Leap Language CLI v{}", env!("CARGO_PKG_VERSION"));
    App::new(app_title)
        .subcommand(format_sub_command)
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

// todo: return Result<(), String>
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
            println!("{:?}", backup_path);
            // create backup to prevent data loss, if something will happen during writing formatted data
            fs::rename(&path, &backup_path).unwrap();
            fs::write(&path, formatted).unwrap();
            fs::remove_file(&backup_path).unwrap();
        }
    }
}
