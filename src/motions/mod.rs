mod template;
use std::fs;
use std::path::*;
use std::fs::File;
use std::io::prelude::*;

pub struct Motion {
  pub name: String,
  pub version: Vec<usize>,
  pub extension: String,
  pub add: String,
  pub sub: String,
}

impl Motion {
  fn new(tmp: &template::Template, dir: &String, add: String, sub: String) -> Self {
    Motion {
      name: tmp.get_name(&add),
      add: read_file(&dir, &add),
      sub: read_file(&dir, &sub),
      version: version(tmp, &add),
      extension: tmp.extension.clone(),
    }
  }

  pub fn test(n: usize) -> Self {
    Motion {
      name: "test".to_string(),
      add: "add: ".to_string() + &n.to_string(),
      sub: "sub: ".to_string() + &n.to_string(),
      version: vec![n,n+1,n+2],
      extension: String::from(""),
    }
  }
}

fn version(tmp: &template::Template, name: &String) -> Vec<usize> {
  let mut version = Vec::new();
  for v in tmp.regex.replace_all(&name, "$1").split('.') {
    version.push(v.parse().unwrap());
  }
  version
}

fn disambiguate(tmp: &template::Template, name: &String) -> String { tmp.regex.replace_all(&name, "$1,$2") }

#[allow(unused_must_use)]
fn read_file(dir: &String, name: &str) -> String {
  let mut f = File::open(dir.clone() + r"\" + name).unwrap();
  let mut s = String::new();
  f.read_to_string(&mut s);
  s
}

fn read_directory(directory: &String) -> Vec<String> {
  let mut names = Vec::new();
  if let Ok(entries) = fs::read_dir(directory.to_string()) {
    for entry in entries {
      if let Ok(file_name) = entry.unwrap().file_name().into_string() {
        names.push(file_name);
      } else {
        panic!("File name did not contain valid Unicode data");
      }
    }
  } else {
    panic!("Directory: '{}' not found!", directory);
  }
  names
}

pub fn discover(directory: &String) -> Vec<Motion> {
  let names = read_directory(directory);
  let cookie = template::Template::get(directory, &names);
  let mut motion_names: Vec<String> = names.into_iter().filter(|name| cookie.regex.is_match(name)).collect();
  motion_names.sort();
  let mut motions_add = Vec::new();
  let mut motions_sub = Vec::new();

  while let Some(n) = motion_names.pop() {
    match cookie.get_op(&n) {
      template::Op::Add => motions_add.push(n),
      template::Op::Sub => motions_sub.push(n),
    }
  }

  let mut motions = Vec::new();

  while motions_add.len() != 0 {
    let add_name = motions_add.pop().unwrap();
    let sub_name = motions_sub.pop().unwrap();

    if disambiguate(&cookie, &add_name) == disambiguate(&cookie, &sub_name) {
      motions.push(Motion::new(&cookie, directory, add_name, sub_name));
    }
  }
  motions
}

pub fn create(directory: String, name: String) {
  let cookie = template::Template::get(&directory, &read_directory(&directory));
  let motion_last = discover(&directory).pop().unwrap();

  let mut version = motion_last.version.clone();
  let i = version.len();
  version[i - 1] += 1;
  let mut version_str = String::new();
  for j in 0..i {
    let mut s = version[j].to_string();
    while s.len() < cookie.version[j]{
      s = 0.to_string() + &s;
    }
    version_str.push_str(&s);
    version_str.push('.');
  }
  version_str.pop();

  let mut path = PathBuf::from(&directory);
  path.push(version_str + &cookie.separator + &name + ".add" + &cookie.extension);
  println!("{:?}", path);
  let mut f = File::create(path.as_path()).unwrap();
  f.write_all(cookie.add.as_bytes()).unwrap();
}
