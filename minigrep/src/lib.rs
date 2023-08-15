use std::fs;
use std::error::Error;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.filename)?;
    for line in contents.lines() {
        if line.contains(&config.query) {
            println!("{}", line);
        }
    }
    Ok(())
}

pub struct Config {
    query: String,
    filename: String,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enought arguments..")
        }
        // 这里使用 clone 的方法会牺牲一点程序性能 可以改造成使用迭代器 这样就可以直接拿到参数所有权
        // let query = args[1].clone();
        // let filename = args[2].clone();

        args.next();
        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };
        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a filename string"),
        };        

        Ok(Config { query: query, filename: filename })
    }
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    // let mut results = Vec::new();
    // for line in contents.lines() {
    //     if line.contains(query) {
    //         results.push(line);
    //     }
    // }

    // 使用迭代器: contents.lines()返回一个迭代器; filter 传入匿名函数得到一个新的迭代器; collect到一个新集合中
    let results =contents.lines()
                                    .filter(|line|line.contains(query))
                                    .collect();

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";
        assert_eq!(vec!["safe, fast, productive."], search(query, contents))
    }
}