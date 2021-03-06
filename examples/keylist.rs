#![allow(unused_must_use)]
extern crate getopts;
extern crate gpgme;

use std::env;
use std::io;
use std::io::prelude::*;
use std::process::exit;

use getopts::Options;

use gpgme::ops;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} [options] [USERID]+", program);
    write!(io::stderr(), "{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optflag("h", "help", "display this help message");
    opts.optflag("", "openpgp", "use the OpenPGP protocol (default)");
    opts.optflag("", "cms", "use the CMS protocol");
    opts.optflag("", "local", "use GPGME_KEYLIST_MODE_LOCAL");
    opts.optflag("", "extern", "use GPGME_KEYLIST_MODE_EXTERN");
    opts.optflag("", "sigs", "use GPGME_KEYLIST_MODE_SIGS");
    opts.optflag("", "sig-notations", "use GPGME_KEYLIST_MODE_SIG_NOTATIONS");
    opts.optflag("", "ephemeral", "use GPGME_KEYLIST_MODE_EPHEMERAL");
    opts.optflag("", "validate", "use GPGME_KEYLIST_MODE_VALIDATE");

    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(fail) => {
            print_usage(program, &opts);
            writeln!(io::stderr(), "{}", fail);
            exit(1);
        }
    };

    if matches.opt_present("h") {
        print_usage(program, &opts);
        return;
    }

    let proto = if matches.opt_present("cms") {
        gpgme::PROTOCOL_CMS
    } else {
        gpgme::PROTOCOL_OPENPGP
    };

    let mut mode = ops::KeyListMode::empty();
    if matches.opt_present("local") {
        mode.insert(ops::KEY_LIST_MODE_LOCAL);
    }
    if matches.opt_present("extern") {
        mode.insert(ops::KEY_LIST_MODE_EXTERN);
    }
    if matches.opt_present("sigs") {
        mode.insert(ops::KEY_LIST_MODE_SIGS);
    }
    if matches.opt_present("sig-notations") {
        mode.insert(ops::KEY_LIST_MODE_SIG_NOTATIONS);
    }
    if matches.opt_present("ephemeral") {
        mode.insert(ops::KEY_LIST_MODE_EPHEMERAL);
    }
    if matches.opt_present("validate") {
        mode.insert(ops::KEY_LIST_MODE_VALIDATE);
    }

    let mut ctx = gpgme::create_context().unwrap();
    ctx.set_protocol(proto).unwrap();
    ctx.set_key_list_mode(mode).unwrap();

    let mut keys = ctx.find_keys(matches.free).unwrap();
    for key in keys.by_ref().filter_map(Result::ok) {
        println!("keyid   : {}", key.id().unwrap_or("?"));
        println!("fpr     : {}", key.fingerprint().unwrap_or("?"));
        println!("caps    : {}{}{}{}",
                 if key.can_encrypt() { "e" } else { "" },
                 if key.can_sign() { "s" } else { "" },
                 if key.can_certify() { "c" } else { "" },
                 if key.can_authenticate() { "a" } else { "" });
        println!("flags   :{}{}{}{}{}{}",
                 if key.is_secret() { " secret" } else { "" },
                 if key.is_revoked() { " revoked" } else { "" },
                 if key.is_expired() { " expired" } else { "" },
                 if key.is_disabled() { " disabled" } else { "" },
                 if key.is_invalid() { " invalid" } else { "" },
                 if key.is_qualified() { " qualified" } else { "" });
        for (i, user) in key.user_ids().enumerate() {
            println!("userid {}: {}", i, user.uid().unwrap_or("[none]"));
            println!("valid  {}: {:?}", i, user.validity())
        }
        println!("");
    }

    if keys.finish().unwrap().truncated() {
        writeln!(io::stderr(),
                "{}: key listing unexpectedly truncated",
                program);
    }
}
