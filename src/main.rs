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

#[derive(Debug)]
enum Data {
    Matching {
        line: String,
        range: std::ops::Range<usize>,
    },
    NotMatching(String),
}

#[derive(Debug, StructOpt)]
struct Opt {
    /// The regex expression used to parse the logs
    expr: String,

    /// If we should highlight data used
    #[structopt(long)]
    highlight: bool,

    /// If we should make the text bold
    #[structopt(long, short)]
    bold: bool,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let re = Regex::new(&opt.expr)?;

    let stdin = io::stdin();

    let mut hist = Histogram::<u64>::new(5)?;

    let data = stdin.lock().lines().map(|line| {
        let line = line.unwrap();
        if let Some(captures) = re.captures(&line) {
            if let Some(m) = captures.get(1) {
                if let Ok(f) = line[m.start()..m.end()].parse::<u64>() {
                    hist += f;
                }
                return Data::Matching {
                    range: m.range(),
                    line,
                };
            }
        }
        Data::NotMatching(line)
    })
    .collect::<Vec<_>>();

    data.into_iter().for_each(|data| {
        match data {
            Data::NotMatching(line) => println!("{}", line),
            Data::Matching {line, range} => {
                let before = &line[0..range.start];
                let during = &line[range.clone()];
                let during = match opt.highlight {
                    true => style(during).yellow(),
                    false => style(during),
                };
                let during = match opt.bold {
                    true => during.bold(),
                    false => during,
                };
                let after = &line[range.end..];
                println!("{}{}{}", before, during, after);
            },
        }
    });

    println!("Found {} samples", hist.len());
    println!("50.0'th percentile:  {}", hist.value_at_quantile(0.5));
    println!("99.9'th percentile:  {}", hist.value_at_quantile(0.999));

    Ok(())
}
