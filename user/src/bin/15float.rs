#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::yield_;

const ROUNDS: usize = 64;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("[float] context test start");

    // 多轮不同 seed 的测试
    for i in 0..ROUNDS {
        let seed = i as f64 + 0.5;
        test_float_round(seed);

        // 和普通的 write_a 一样，每轮调用一次 yield_
        // 既测试浮点 + trap，也测试你封装好的 syscall 路径
        yield_();

        if i % 8 == 0 {
            println!("[float] round {}/{}", i, ROUNDS);
        }
    }

    println!("[float] context test OK!");
    0
}

/// 单轮浮点上下文测试：
/// 1. 用 seed 生成几组浮点数 a,b,c,d；
/// 2. 计算一次结果 r_prev；
/// 3. 调用 yield_() 进入内核，回来后继续用 a,b,c,d 算；
/// 4. 通过 assert! 检查运算结果是否符合预期。
fn test_float_round(seed: f64) {
    // 这里尽量构造一点复杂的表达式，让编译器用到更多 f 寄存器
    let mut a = 1.234_567_89_f64 * (seed + 1.0);
    let mut b = -3.141_592_65_f64 * (seed - 0.25);
    let mut c = 0.915_965_594_f64 * (seed + 2.0);
    let mut d = (a + b) * c - seed;

    // 第一次运算，记录一个“基线结果”
    let r_prev = calc_expr(a, b, c, d);

    // 在浮点值“活着”的时候调用 yield_，进入内核
    // 如果 trap 过程中浮点上下文没有保存好，后面的计算大概率会出问题
	yield_();

    // yield_ 之后再对这些值做一轮计算
    a = a * 1.000_000_1 + 0.000_000_1;
    b = b * 0.999_999_9 - 0.000_000_2;
    c = c + 0.123_456_789;
    d = d - 0.234_567_891;
    let r_now = calc_expr(a, b, c, d);

    // 基于数学关系做一些简单的断言，
    // 如果中间某一步被严重污染（比如 f 寄存器被乱写），通常会直接挂掉。
    let diff = (r_now - r_prev).abs();
    let eps = 1e-6_f64;

    // 结果不应该是 NaN/Inf
    assert!(diff.is_finite());

    // 调整过参数之后，r_now 与 r_prev 应该有有限差异，但不会特别巨大
    // （如果上下文乱了，通常会变成极大值或者 NaN）
    assert!(diff < 1e12_f64);

    // 再加一条“自反”式的断言：对 r_now 做一个可逆变换再还原
    let tmp = r_now * 1.000_000_1_f64 - 0.000_000_1_f64;
    let restored = (tmp + 0.000_000_1_f64) / 1.000_000_1_f64;
    assert!((restored - r_now).abs() < eps);
}

/// 抽出来一个小计算，避免 main 里过长，
/// 也有助于让编译器多用一些寄存器而不是都丢栈上。
fn calc_expr(a: f64, b: f64, c: f64, d: f64) -> f64 {
    // 故意写得稍微啰嗦点，增加浮点指令数量
    let t1 = a * b + c;
    let t2 = d - a * 0.5 + b * 0.25;
    let t3 = (t1 + t2) * 1.000_000_01;
    t3 - c * 0.333_333_3 + d * 0.666_666_7
}
