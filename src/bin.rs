#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
mod lib;
#[macro_use]
extern crate clap;

fn main() {
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (author: "Daniel V. <dowy.vz6@gmail.com>")
        (about: "Takes pictures and generates links")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand local =>
            (about: "Takes a screenshot, saves it locally and returns it's path.")
        )
        (@subcommand s3 =>
            (about: "Takes a screenshot, saves it in s3 and returns it's link.")
        )
        (@subcommand upload =>
            (about: "Takes a screenshot, sends it to sampic and returns it's link.")
        )
        (@subcommand server =>
            (about: "Runs a sampic server.")
        )
    )
    .get_matches();
    match matches.subcommand_name() {
        Some("local") => lib::local_screenshot(),
        Some("s3") => lib::s3_screenshot(),
        Some("upload") => lib::upload_screenshot(),
        Some("server") => rocket::ignite()
            .mount("/", routes![lib::server::upload])
            .launch()
            .to_string(),
        Some(_) | None => "Do something!".to_string(),
    };
}
