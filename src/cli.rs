use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use clap::{App, Arg, ArgMatches, SubCommand};
use leap_lang::{formatter, parser::parser::Parser};

pub fn run() {
    let args = cli_args();
    let command_result = match args.subcommand() {
        ("format", Some(args)) => command_format(args),
        ("verify", Some(args)) => command_verify(args),
        ("print-std", _) => print_std(),
        _ => Ok(()),
    };
    if let Err(e) = command_result {
        println!("{}", e);
        exit(1);
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
    let print_std_sub_command = SubCommand::with_name("print-std");
    let app_title = format!("Leap Language CLI v{}", env!("CARGO_PKG_VERSION"));
    App::new(app_title)
        .subcommand(format_sub_command)
        .subcommand(verify_sub_command)
        .subcommand(print_std_sub_command)
        .get_matches()
}

fn get_backup_path(path: &Path) -> Result<PathBuf, String> {
    let name = path
        .file_name()
        .ok_or("Can't find path for backup")?
        .to_owned();
    let mut parent = path.to_owned();
    parent.pop();
    for n in 1..100 {
        let suffix = OsString::from(format!("_backup{}", n));
        let mut backup_name = name.clone();
        backup_name.push(suffix);
        let mut backup_path = parent.clone();
        backup_path.push(backup_name);
        if !backup_path.exists() {
            return Ok(backup_path);
        }
    }
    Err("Can't find path for backup".to_owned())
}

fn command_format(args: &ArgMatches) -> Result<(), String> {
    let paths = args.values_of("spec").unwrap();
    let to_stdout = args.is_present("stdout");
    for path in paths {
        let path_buf = fs::canonicalize(path).map_err(|_| format!("Path error: `{}`", path))?;
        let data = fs::read_to_string(&path_buf);
        let formatted = match data {
            Ok(s) => {
                formatter::format(&s).ok_or_else(|| format!("Error formatting: `{}`", path))?
            }
            Err(_) => return Err(format!("Can't read: `{}`", path)),
        };
        if to_stdout {
            print!("{}", formatted);
        } else {
            // we was able to read file - path is correct
            let backup_path = get_backup_path(&path_buf)?;
            // create backup to prevent data loss, if something will happen during writing formatted data
            fs::rename(&path_buf, &backup_path)
                .map_err(|_| format!("Failed to backup: `{}`", path))?;
            fs::write(&path_buf, formatted).map_err(|_| format!("Failed to write: `{}`", path))?;
            // formatted data already saved, we can delete backup now
            fs::remove_file(&backup_path)
                .map_err(|_| format!("Failed delete backup: `{}`", path))?;
        }
    }
    Ok(())
}

fn command_verify(args: &ArgMatches) -> Result<(), String> {
    let paths = args.values_of("spec").unwrap();
    let result = Parser::parse_paths_iter(paths);
    if let Err(e) = result {
        Err(e.error_report())
    } else {
        Ok(())
    }
}

fn print_std() -> Result<(), String> {
    println!("{}", leap_lang::stdtypes::STD_TYPES);
    Ok(())
}
