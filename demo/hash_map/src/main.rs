use std::collections::HashMap;

fn main() {
    let teams = vec![String::from("Blue"), String::from("Yellow"), String::from("White")];
    let scores = vec![10, 20];
    let sum: HashMap<_, _> = teams.iter().zip(scores.iter()).collect();
    println!("{:?}", sum);

    let blue_name = String::from("Blue");
    let blue_score = sum.get(&blue_name);
    match blue_score {
        Some(s) => println!("{}", s),
        None => println!("no score..")
    }

    for (k, v) in &sum {
        println!("{}: {}", k, v)
    }

    let text = "hello world wonderful world";
    let mut map = HashMap::new();
    for word in text.split_whitespace() {
        // 如果值存在 返回一个可变引用 后面使用*count+1赋值
        let count = map.entry(word).or_insert(0);
        *count += 1
    }
    println!("{:#?}", map)
}
