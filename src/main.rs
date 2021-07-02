use anyhow::Result;
use console::{Style, style};
use core::cmp::Ordering;
use hdrhistogram::Histogram;
use regex::Regex;
use std::convert::From;
use std::cmp::PartialOrd;
use std::io::{self, BufRead};
use structopt::{clap::arg_enum, StructOpt};


// TODO(ckonstad)
//  -context? (can we sort + context?)

arg_enum! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum Sorting {
        Original,
        Asc,
        Desc,
    }
}

#[derive(Debug, PartialEq)]
enum Data {
    Matching {
        line: String,
        range: std::ops::Range<usize>,
        parsed: u64,
    },
    NotMatching(String),
}

impl PartialOrd for Data {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let a = match self {
            Data::Matching { parsed, .. } => Some(parsed),
            _ => None,
        };
        let b = match other {
            Data::Matching { parsed, .. } => Some(parsed),
            _ => None,
        };

        match (a, b) {
            (Some(a), Some(b)) => a.partial_cmp(b),
            (_, _) => None,
        }
    }
}

#[derive(Debug)]
enum Percentile {
    P99,
    P90,
    P50,
    Other,
}

impl From<Percentile> for Style {
    fn from(p: Percentile) -> Self {
        match p {
            Percentile::P99 => Style::new().red(),
            Percentile::P90 => Style::new().yellow(),
            Percentile::P50 => Style::new().green(),
            Percentile::Other => Style::new().blue(),
        }
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    /// The regex expression used to parse the logs
    expr: String,

    /// If we should highlight data used.  This disables the heatmap.
    #[structopt(long)]
    highlight: bool,

    /// If we should make the text bold
    #[structopt(long, short)]
    bold: bool,

    /// If we should print some debug stats at the end
    #[structopt(long)]
    debug: bool,

    /// If we should force color output
    #[structopt(short, long)]
    force_colors: bool,

    /// If we should only print matching lines
    #[structopt(short, long)]
    matching: bool,

    /// If we should only print matching lines
    #[structopt(
        short,
        long,
        possible_values = &Sorting::variants(),
        case_insensitive = true,
        default_value="original",
    )]
    sorting: Sorting,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let re = Regex::new(&opt.expr)?;

    if opt.force_colors {
        console::set_colors_enabled(opt.force_colors);
    }

    let stdin = io::stdin();
    let mut hist = Histogram::<u64>::new(5)?;

    let data = stdin.lock().lines().map(|line| {
        let line = line.unwrap();
        if let Some(captures) = re.captures(&line) {
            if let Some(m) = captures.get(1) {
                if let Ok(f) = line[m.start()..m.end()].parse::<u64>() {
                    hist += f;

                    return Data::Matching {
                        range: m.range(),
                        line,
                        parsed: f,
                    };
                }
            }
        }
        Data::NotMatching(line)
    })
    .collect::<Vec<_>>();

    let p99 = hist.value_at_quantile(0.99);
    let p90 = hist.value_at_quantile(0.90);
    let p50 = hist.value_at_quantile(0.50);

    let to_percentile = |val| {
        if p99 <= val {
            Percentile::P99
        } else if p90 <= val {
            Percentile::P90
        } else if p50 <= val {
            Percentile::P50
        } else {
            Percentile::Other
        }
    };

    let data = match (opt.matching, opt.sorting) {
        (false, Sorting::Original) => data,
        (true, Sorting::Original) => data
            .into_iter()
            .filter(|d| matches!(d, Data::Matching {..}))
            .collect::<Vec<_>>(),
        (_, Sorting::Asc) | (_, Sorting::Desc) => {
            let mut data = data
                .into_iter()
                .filter(|d| matches!(d, Data::Matching {..}))
                .collect::<Vec<_>>();
            data.sort_by(|a, b| a.partial_cmp(b).unwrap());
            if opt.sorting == Sorting::Desc {
                data.reverse();
            }
            data
        }
    };

    data.into_iter().for_each(|data| {
        match data {
            Data::NotMatching(line) => println!("{}", line),
            Data::Matching {line, range, parsed} => {
                let before = &line[0..range.start];
                let during = &line[range.clone()];
                let during = match opt.highlight {
                    true => style(during).yellow(),
                    false => {
                        let p = to_percentile(parsed);
                        Style::from(p).apply_to(during)
                    }
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

    if opt.debug {
        println!("Number of samples: {}", hist.len());
        println!("99'th percentile:  {}", hist.value_at_quantile(0.99));
        println!("90'th percentile:  {}", hist.value_at_quantile(0.9));
        println!("50'th percentile:  {}", hist.value_at_quantile(0.5));
    }

    Ok(())
}
