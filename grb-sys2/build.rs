use std::env;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use std::process::Command;

use regex::Regex;

#[derive(Debug)]
enum Error {
  DoesNotExist(PathBuf),
  GurobiHomeNotGiven,
  GurobiClNotFound,
  CannotParseGurobiVersion,
}

impl Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::DoesNotExist(p) => f.write_fmt(format_args!(
        "GUROBI_HOME is set to {:?} but {:?} does not exist (or is not a directory)",
        p.parent().unwrap(),
        p
      )),
      Error::GurobiHomeNotGiven => f.write_str("GUROBI_HOME not set"),
      Error::GurobiClNotFound => f.write_str("gurobi_cl not found"),
      Error::CannotParseGurobiVersion => f.write_str("Cannot get Gurobi version"),
    }
  }
}

impl std::error::Error for Error {}

fn get_gurobi_home() -> Result<PathBuf, Error> {
  let path = env::var("GUROBI_HOME").map_err(|_| Error::GurobiHomeNotGiven)?;

  // You cannot unset environment variables in the config.toml so this is the next best thing.
  if path.is_empty() {
    return Err(Error::GurobiHomeNotGiven);
  }

  let path: PathBuf = path.into();

  path.canonicalize().map_err(|_| Error::DoesNotExist(path))
}

fn get_gurobi_library(gurobi_home: &Path) -> Result<String, Error> {
  let gurobi_cl = gurobi_home.join("bin").join("gurobi_cl");

  if !gurobi_cl.exists() {
    return Err(Error::GurobiClNotFound);
  }

  let output = Command::new(gurobi_cl)
    .arg("--version")
    .output()
    .map_err(|_| Error::CannotParseGurobiVersion)?
    .stdout;
  let output = String::from_utf8(output).map_err(|_| Error::CannotParseGurobiVersion)?;

  let re = Regex::new(r"Gurobi Optimizer version (\d+).(\d+).(\d+)").unwrap();
  let captures = re.captures(&output).unwrap();

  let major = &captures[1];
  let minor = &captures[2];

  Ok(format!("gurobi{}{}", major, minor))
}

fn try_main() -> Result<(), Error> {
  let gurobi_home = get_gurobi_home()?;

  let library = get_gurobi_library(&gurobi_home)?;

  println!(
    "cargo:rustc-link-search=native={}",
    gurobi_home.join("lib").display()
  );
  println!("cargo:rustc-link-lib=dylib={}", library);

  Ok(())
}

fn main() {
  if let Err(e) = try_main() {
    eprintln!("{}", e);
    std::process::exit(1);
  }
}
