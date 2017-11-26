#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::io::Result;
use std::fs::File;
use rocket::response::NamedFile;
use std::path::{Path, PathBuf};

#[get("/")]
fn index() -> Result<NamedFile> {
    NamedFile::open("vue/dist/index.html")
}

#[get("/<file..>", rank = 5)]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("vue/dist/").join(file)).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![index, files]).launch();
}
