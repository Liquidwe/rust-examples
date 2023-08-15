use std::{u32, thread::sleep, time::Duration};

// 1）闭包实现缓存器
// 记忆化、延迟计算:创建一个struct 它持有闭包及其调用结果
// 所有的闭包都需要实现至少以下trait之一:
// - Fn
// - FnMut
// - FnOnce

// 2) 闭包还能捕获他们所在的环境（变量等）
// 大部分情况下我们不需要捕获环境 所以函数不允许捕获环境
// 闭包捕获环境有内存开销

fn main() {
    genrate_workout(21, 3);

}

// fn algorithm_calculation(intensity: u32) -> u32 {
//     println!("Perform complex calculations..");
//     sleep(Duration::from_secs(2));
//     intensity
// }

struct Cacher<T> 
    where T: Fn(u32) -> u32 {
        calculation: T,
        value: Option<u32>,
    }

impl<T> Cacher<T>
where T:Fn(u32) -> u32 {
    fn new(calculation: T) -> Cacher<T> {
        Cacher { calculation, value:None }
    }

    fn value(&mut self, arg: u32) -> u32 {
        match self.value {
            Some(v) => v,
            None => {
                // self.calculation 是Cacher结构体中的一个字段 其类型是T 其中 T 是一个实现了Fn(u32) -> u32的闭包或函数 因此self.calculation是一个可以实际调用的实体
                let v = (self.calculation)(arg);
                self.value = Some(v);
                v
            }
        }
    }
}

fn genrate_workout(intensity: u32, random_number: u32) {

    // 匿名函数
    // 只定义函数 但是不实际执行
    // let result = |num| {
    //     println!("Perform complex calculations..");
    //     sleep(Duration::from_secs(2));
    //     num
    // };

    // 为了让整个匿名函数总体只需要计算一次 这里需要把它封装到 struct 中 让 stuct 持有闭包极其计算结果
    let mut result = Cacher::new(|num|{
        println!("Perform complex calculations..");
        sleep(Duration::from_secs(2));
        num
    });

    if intensity < 25 {
        println!("Fueling Fitness..{}", result.value(intensity));
        println!("Keep going. Keep going..{}", result.value(intensity))
    } else {
        if random_number == 3  {
            println!("random_number == 3")
        } else {
            println!("random_number != 3, {}", result.value(intensity))
        }
    }
} 


#[cfg(test)]
mod tests {
    #[test]
    fn call_with_different_values() {
        let mut c = super::Cacher::new(|a|a);
        let v1 = c.value(1);
        let v2 = c.value(2);
        assert_eq!(v2, 2);
    }
}
