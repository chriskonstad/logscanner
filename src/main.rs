use anyhow::Result;
use console::style;
use regex::Regex;
use std::io::{self, BufRead};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    expr: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let re = Regex::new(&opt.expr)?;

    let stdin = io::stdin();
    stdin.lock().lines().for_each(|l| {
        let l = l.unwrap();
        if let Some(captures) = re.captures(&l) {
            if let Some(m) = captures.get(1) {
                let before = &l[0..m.start()];
                let during = &l[m.start()..m.end()];
                let after = &l[m.end()..];
                println!("{}{}{}", before, style(during).yellow(), after);
            } else {
                println!("{}", l);
            }
        } else {
            println!("{}", l);
        }
    });

    Ok(())
}
