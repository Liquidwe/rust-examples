use std::cmp::Ordering;
use std::io;
use rand::Rng;

pub struct Guess {
    value: i32,
}

impl Guess {
    pub fn new(value: i32) -> Result<Guess, String> {
        if value < 1 || value > 100 {
            Err(format!("Guess value must be between 1 and 100, got {}", value))
        } else {
            Ok(Guess { value })
        }
    }

    // 提供一个公共的方法来访问私有 Guess 里面的 value 的值
    pub fn value(&self) -> i32 {
        self.value
    }
}

fn main() {
    println!("Let's guess the number!");
    let secret_number = rand::thread_rng().gen_range(1..=100);

    loop {
        println!("Guess the number:");

        let mut guess = String::new();
        io::stdin().read_line(&mut guess).expect("Failed to read line");

        let guess: i32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Please enter a valid number!");
                continue;
            }
        };

        let guess = match Guess::new(guess) {
            Ok(g) => g,
            Err(err) => {
                println!("{}", err);
                continue;
            }
        };

        match guess.value().cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
