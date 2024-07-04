use std::fs::File;
use std::io;
use std::io::{Read, read_to_string};

// ？运算符只能用来返回 Result 类型的错误
fn read_username_from_file() -> Result<String, io::Error> {
    // let mut f = File::open("hello.txt")?;
    // let mut s = String::new();
    // f.read_to_string(&mut s);
    // Ok(s)
    // ⬇️简写
    let mut s = String::new();
    File::open("hello.txt")?.read_to_string(&mut s)?;
    Ok(s)
}

fn main() {
    // let f = File::open("hello.txt");
    // let f = match f {
    //     Ok(file) => file,
    //     Err(error) => {
    //         panic!("Error opening file {:?}", error)
    //     }
    // };

    // unwrap 是一个用于快速提取 Result 和 Option 类型的方法
    let f = File::open("hello.txt").unwrap();
    let f = File::open("hello.txt").expect("无法打开文件");

    // ？问号简化错误传递
    let s = read_username_from_file();
    println!("{:#?}", s)
}
