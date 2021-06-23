#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use sampicore as lib;
fn main() {
    let matches = clap_app!(sampic =>
        (version: "0.2.0")
        (about: "Takes pictures and generates links")
        (@setting SubcommandRequiredElseHelp)
        (@setting ColoredHelp)
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
        (@subcommand config =>
            (about: "Manage sampic configuration.")
            (@setting SubcommandRequiredElseHelp)
            (@setting ColoredHelp)
            (@subcommand set =>
                (about: "Set sampic configuration.")
                (@setting ArgRequiredElseHelp)
                (@arg NAME: +required "Name of configuration to set.")
                (@arg VALUE: +required "Value to set.")
            )
            (@subcommand list =>
                (about: "List current sampic configuration values.")
            )
        )
    )
    .get_matches();
    let message: String = match matches.subcommand_name() {
        Some("local") => lib::local_screenshot(),
        Some("s3") => lib::s3_screenshot(),
        Some("upload") => lib::upload_screenshot(),
        Some("server") => rocket::ignite()
            .mount("/", routes![lib::server::upload])
            .launch()
            .to_string(),
        Some("config") => {
            let subcommand = matches.subcommand_matches("config").unwrap();
            match subcommand.subcommand_name() {
                Some("set") => {
                    let set_matches = subcommand.subcommand_matches("set").unwrap();
                    let name = set_matches.value_of("NAME").unwrap().to_string();
                    let value = set_matches.value_of("VALUE").unwrap().to_string();
                    lib::config::set(name, value).unwrap();
                    return ();
                }
                Some("list") => {
                    let list: String = lib::config::list().unwrap();
                    println!("{}", list);
                    return ();
                }
                Some(_) | None => "Ok".to_string(),
            };
            return ();
        }
        Some(_) | None => "Do something!".to_string(),
    };
    println!("{}", message);
}
