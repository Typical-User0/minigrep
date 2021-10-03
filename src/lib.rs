use colored::Colorize;
use std::env;
use std::error::Error;
use std::fs;
use std::process::exit;


fn parse_string<'a>(
    s: &'a str,
    search: &'a str,
    case_sensitive: bool,
) -> Result<(usize, usize), String> {
    let mut string = s.to_string();
    let mut search_str = search.to_string();

    if !case_sensitive {
        string = s.to_lowercase();
        search_str = search.to_lowercase();
    }

    let start = string.find(&search_str).ok_or("Not found")?;
    let end = search.len() + start;

    Ok((start, end))
}

pub struct Config<'a> {
    pub query: String,
    pub filenames: Vec<&'a String>,
    pub case_sensitive: bool,
}

pub struct MatchingLine<'a> {
    pub text: &'a str,
    pub searched_text: &'a str,
    pub line_number: usize,
}

impl<'a> MatchingLine<'a> {
    pub fn new(text: &'a str, searched_text: &'a str, line_number: usize) -> Self {
        MatchingLine {
            text,
            searched_text,
            line_number,
        }
    }
}

impl<'a> Config<'a> {
    pub fn new(args: &'a [String]) -> Result<Self, &str> {

        if args.len() < 3 {
            return Err("Not enough arguments");
        }

        let query = args[1].clone();
        let mut filenames: Vec<&String> = Vec::new();

        for i in 2..args.len() {
            filenames.push(&args[i]);
        }

        let case_sensitive = env::var("CASE_SENSITIVE").is_ok();

        return Ok(Config {
            query,
            filenames,
            case_sensitive,
        });
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    for i in 0..config.filenames.len() {
        let contents = fs::read_to_string(&config.filenames[i])?;

        let results = search(&config.query, &contents, config.case_sensitive);

        if results.len() >= 1 {
            println!("\n{}", config.filenames[i].blue());
            for line in results {
                let (start, end) =
                    parse_string(line.text, line.searched_text, config.case_sensitive)
                        .unwrap_or_else(|err| {
                            eprintln!("Problem parsing rows: {}", err);
                            exit(1);
                        });

                println!(
                    "{}: {}{}{}",
                    line.line_number.to_string().green(),
                    &line.text[0..start],
                    &line.text[start..end].red(),
                    &line.text[end..]
                );
            }
        }
    }

    return Ok(());
}

pub fn search<'a>(
    query: &'a str,
    contents: &'a str,
    case_sensitive: bool,
) -> Vec<MatchingLine<'a>> {
    let mut results = Vec::new();
    let mut line_num: usize = 0;

    if case_sensitive {
        for line in contents.lines() {
            line_num += 1;

            if line.contains(&query.to_lowercase()) {
                results.push(MatchingLine::new(line, query, line_num));
            }
        }
    } else {
        for line in contents.lines() {
            line_num += 1;
            if line.to_lowercase().contains(&query.to_lowercase()) {
                results.push(MatchingLine::new(line, query, line_num));
            }
        }
    }

    return results;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_matched_lines(expected: &[MatchingLine], result: &[MatchingLine]) {
        for i in 0..expected.len() {
            assert_eq!(expected[i].searched_text, result[i].searched_text);
            assert_eq!(expected[i].line_number, result[i].line_number);
            assert_eq!(expected[i].text, result[i].text);
        }
    }

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duck tape.";
        let expected = [MatchingLine::new("safe, fast, productive.", "duct", 2)];
        let result = search(query, contents, true);

        compare_matched_lines(&expected, &result);
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";
        let expected = [
            MatchingLine::new("Rust:", "rUsT", 1),
            MatchingLine::new("Trust me.", "rUsT", 4),
        ];
        let result = search(query, contents, false);

        compare_matched_lines(&expected, &result);
    }
}
