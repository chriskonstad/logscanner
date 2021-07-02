use anyhow::Result;
use console::style;
use hdrhistogram::Histogram;
use regex::Regex;
use std::io::{self, BufRead};
use structopt::StructOpt;


// TODO(ckonstad)
//  -filter
//  -sort
//  -heatmap

#[derive(Debug, StructOpt)]
struct Opt {
    /// The regex expression used to parse the logs
    expr: String,

    /// If we should highlight data used
    #[structopt(long)]
    highlight: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let re = Regex::new(&opt.expr)?;

    let stdin = io::stdin();

    let mut hist = Histogram::<u64>::new(5)?;

    stdin.lock().lines().for_each(|l| {
        let l = l.unwrap();
        if let Some(captures) = re.captures(&l) {
            if let Some(m) = captures.get(1) {
                let before = &l[0..m.start()];
                let during = &l[m.start()..m.end()];
                if let Ok(f) = during.parse::<u64>() {
                    hist += f;
                }
                let during = match opt.highlight {
                    true => style(during).yellow(),
                    false => style(during),
                };
                let after = &l[m.end()..];
                println!("{}{}{}", before, during, after);
            } else {
                println!("{}", l);
            }
        } else {
            println!("{}", l);
        }
    });

    println!("Found {} samples", hist.len());
    println!("50.0'th percentile:  {}", hist.value_at_quantile(0.5));
    println!("99.9'th percentile:  {}", hist.value_at_quantile(0.999));

    Ok(())
}
