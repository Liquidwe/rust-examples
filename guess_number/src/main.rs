use std::cmp::Ordering;
use std::io;
use rand::Rng;

fn main() {
    println!("lets guess number!");
    let secret_number = rand::thread_rng().gen_range(1..101);
    loop {
        println!("guess number:");
        let mut guess = String::new();
        io::stdin().read_line(&mut guess).expect("error..");
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };
        println!("you number: {}", guess);
        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You Win!");
                break
            }
        }
    }
}
