use clap::{App, Arg, SubCommand};

pub fn get_matches<'a>() -> clap::ArgMatches<'a> {
    let db_file = Arg::with_name("DB_FILE")
        .help("Path to frecency database file")
        .required(true);

    let entry_file = Arg::with_name("ENTRY_FILE")
        .help("Path to list of entries, separated by newlines")
        .required(true);

    App::new("frece")
        .version("1.0.5")
        .author("Mateen Ulhaq <mulhaq2005@gmail.com>")
        .about("Frecency indexed database")
        .subcommand(
            SubCommand::with_name("add")
                .about("Add entry to database")
                .arg(db_file.clone().index(1))
                .arg(
                    Arg::with_name("ENTRY")
                        .help("Entry to add")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("increment")
                .about("Increases an entry's count and resets its timer")
                .arg(db_file.clone().index(1))
                .arg(
                    Arg::with_name("ENTRY")
                        .help("Entry to increment")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Creates a database file from given list of entries")
                .arg(db_file.clone().index(1))
                .arg(entry_file.clone().index(2)),
        )
        .subcommand(
            SubCommand::with_name("print")
                .about("Prints list of frecency sorted entries")
                .arg(db_file.clone().index(1))
                .arg(
                    Arg::with_name("sort")
                        .help("Sort method")
                        .long("sort")
                        .takes_value(true)
                        .default_value("frecency")
                        .possible_values(&[
                            "none",
                            "alphabetical",
                            "frecency",
                            "frequency",
                            "recency",
                        ]),
                )
                .arg(
                    Arg::with_name("verbose")
                        .help("Outputs frecency, counts, date, and entries")
                        .short("v")
                        .long("verbose"),
                ),
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("Set an entry's frequency count and last access time")
                .arg(db_file.clone().index(1))
                .arg(
                    Arg::with_name("ENTRY")
                        .help("Entry to modify")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("count")
                        .help("Frequency count")
                        .long("count")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("time")
                        .help("Last access time")
                        .long("time")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("update")
                .about("Updates a database file from given list of entries")
                .arg(db_file.clone().index(1))
                .arg(entry_file.clone().index(2))
                .arg(
                    Arg::with_name("purge-old")
                        .help("Purge any entries *not* in ENTRY_FILE")
                        .long("purge-old"),
                ),
        )
        .get_matches()
}
