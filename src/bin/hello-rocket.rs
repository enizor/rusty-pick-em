#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

use rocket::Request;
use rocket::response::Redirect;
use rocket::request::Form;
use rocket_contrib::Template;

#[macro_use] extern crate serde_derive;

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

#[get("/login")]
fn login() -> Result<NamedFile> {
    NamedFile::open("front/login.html")
}

#[derive(FromForm, Serialize)]
struct AuthUser {
    name: String,
    password: String,
}

#[post("/login", data = "<auth_user>")]
fn authUser(auth_user: Form<AuthUser>) -> Template {
    Template::render("test", &auth_user.get())
}



fn main() {
    rocket::ignite().mount("/", routes![index, files, login, authUser])
    .attach(Template::fairing())
    .launch();
}
