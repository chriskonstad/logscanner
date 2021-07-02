use anyhow::Result;
use console::{style, Style};
use core::cmp::Ordering;
use hdrhistogram::Histogram;
use rayon::prelude::*;
use regex::Regex;
use std::cmp::PartialOrd;
use std::convert::From;
use std::fs::File;
use std::io::{self, BufRead, Write, BufWriter, BufReader};
use std::path::PathBuf;
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

    #[structopt(long, short, parse(from_os_str))]
    input: Option<PathBuf>,

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

fn match_line(re: &Regex, line: String) -> Data {
    if let Some(captures) = re.captures(&line) {
        if let Some(m) = captures.get(1) {
            if let Ok(f) = line[m.start()..m.end()].parse::<u64>() {
                return Data::Matching {
                    range: m.range(),
                    line,
                    parsed: f,
                };
            }
        }
    }
    Data::NotMatching(line)
}

fn filter_and_sort(data: Vec<Data>, matching: bool, sorting: Sorting) -> Vec<Data> {
    match (matching, sorting) {
        (false, Sorting::Original) => data,
        (true, Sorting::Original) => data
            .into_iter()
            .filter(|d| matches!(d, Data::Matching { .. }))
            .collect::<Vec<_>>(),
        (_, Sorting::Asc) | (_, Sorting::Desc) => {
            let mut data = data
                .into_iter()
                .filter(|d| matches!(d, Data::Matching { .. }))
                .collect::<Vec<_>>();
            data.par_sort_by(|a, b| a.partial_cmp(b).unwrap());
            if sorting == Sorting::Desc {
                data.reverse();
            }
            data
        }
    }
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let re = Regex::new(&opt.expr)?;

    if opt.force_colors {
        console::set_colors_enabled(opt.force_colors);
    }

    let data = match &opt.input {
        Some(file) => {
            let f = File::open(file)?;
            io::BufReader::with_capacity(1 * 1024 * 1024, f)
                .lines()
                .map(|line| line.unwrap())
                .collect::<Vec<_>>()
        }
        None => {
            let stdin = io::stdin();
            BufReader::with_capacity(8 * 1024 * 1024, stdin.lock())
                .lines()
                .map(|line| line.unwrap())
                .collect::<Vec<_>>()
        }
    };

    let mut hist = Histogram::<u64>::new(5)?;

    let data = data
        .into_par_iter()
        .map_with(re, |re, line| match_line(&re, line))
        .collect::<Vec<_>>();

    data.iter().for_each(|d| match d {
        Data::Matching { parsed, .. } => hist += *parsed,
        _ => {}
    });

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

    // println! grabs the stdout lock each time, so we'll grab it here
    // and use BufWriter/writeln to reduce the amount of times we need to grab
    // the locks
    // We'll buffer to 1MB chunks.  Memory isn't free, but also we can burn
    // 1MB without a worry, and stdout writing is EXPENSIVE.
    // With a small 96k line input, buffering at 8KB took 300ms with 150 writes.
    // Buffering at 1MB took 33ms, with 2 write.
    let stdout = io::stdout();
    let mut out = BufWriter::with_capacity(8 * 1024 * 1024, stdout.lock());
    filter_and_sort(data, opt.matching, opt.sorting)
        .into_iter()
        .for_each(|data| match data {
            Data::NotMatching(line) => writeln!(out, "{}", line).unwrap(),
            Data::Matching {
                line,
                range,
                parsed,
            } => {
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
                writeln!(out, "{}{}{}", before, during, after).unwrap()
            }
        });
    out.flush()?;

    if opt.debug {
        println!("Number of samples: {}", hist.len());
        println!("99'th percentile:  {}", p99);
        println!("90'th percentile:  {}", p90);
        println!("50'th percentile:  {}", p50);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data1() -> Data {
        Data::Matching {
            line: "1".to_string(),
            range: 0..1,
            parsed: 1,
        }
    }

    fn data5() -> Data {
        Data::Matching {
            line: "5".to_string(),
            range: 0..1,
            parsed: 5,
        }
    }

    fn data10() -> Data {
        Data::Matching {
            line: "10".to_string(),
            range: 0..2,
            parsed: 10,
        }
    }

    fn hello() -> Data {
        Data::NotMatching("hello".to_string())
    }

    fn world() -> Data {
        Data::NotMatching("world".to_string())
    }

    fn sample_data() -> Vec<Data> {
        vec![data5(), hello(), data10(), world(), data1()]
    }

    #[test]
    fn test_match_line() {
        let re = Regex::new(r"(\d+)").unwrap();
        assert_eq!(
            Data::NotMatching("Hello".to_string()),
            match_line(&re, "Hello".to_string())
        );
        assert_eq!(
            Data::Matching {
                line: "123".to_string(),
                range: 0..3,
                parsed: 123,
            },
            match_line(&re, "123".to_string())
        );
    }

    #[test]
    fn test_match_line_bad_regex() {
        let re = Regex::new(r"(\D+)").unwrap();
        assert_eq!(
            Data::NotMatching("Hello".to_string()),
            match_line(&re, "Hello".to_string())
        );
        assert_eq!(
            Data::NotMatching("123".to_string()),
            match_line(&re, "123".to_string())
        );
    }

    #[test]
    fn test_match_line_no_capture() {
        let re = Regex::new(r"\d+").unwrap();
        assert_eq!(
            Data::NotMatching("123".to_string()),
            match_line(&re, "123".to_string())
        );
    }

    #[test]
    fn test_no_matching_no_sorting() {
        assert_eq!(
            sample_data(),
            filter_and_sort(sample_data(), false, Sorting::Original),
        );
    }

    #[test]
    fn test_matching_no_sorting() {
        assert_eq!(
            vec![data5(), data10(), data1()],
            filter_and_sort(sample_data(), true, Sorting::Original),
        );
    }

    #[test]
    fn test_no_matching_sorting_desc() {
        assert_eq!(
            vec![data10(), data5(), data1()],
            filter_and_sort(sample_data(), false, Sorting::Desc),
        );
    }

    #[test]
    fn test_matching_sorting_asc() {
        assert_eq!(
            vec![data1(), data5(), data10()],
            filter_and_sort(sample_data(), true, Sorting::Asc),
        );
    }
}
