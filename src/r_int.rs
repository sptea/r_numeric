use std::fmt;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

#[derive(Debug)]
#[allow(dead_code)]
pub struct RInt {
    bits: u32,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ParseError {
    InvalidDigit,
    Overflow,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidDigit => write!(f, "Invalid digit found"),
            ParseError::Overflow => write!(f, "Overflow occurred"),
        }
    }
}

impl std::error::Error for ParseError {}

// Parse時のステートマシンのState定義
#[derive(Clone, Copy)]
enum State {
    Start,
    InInteger,
    InFraction,
}

impl Add for RInt {
    type Output = Self;

    // + 演算子をオーバーロードするためにはAddトレイトを実装する必要があり、Resultは返せないため泣く泣くwrapping_addにしている
    fn add(self, other: Self) -> Self {
        RInt {
            bits: self.bits.wrapping_add(other.bits),
        }
    }
}

impl Sub for RInt {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        RInt {
            bits: self.bits.wrapping_sub(other.bits),
        }
    }
}

impl Mul for RInt {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        RInt {
            bits: self.bits.wrapping_mul(other.bits),
        }
    }
}

impl Div for RInt {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        RInt {
            bits: self.bits.wrapping_div(other.bits),
        }
    }
}

impl RInt {
    // 10進数表記の正数を32bitの正数型に変換する
    // 2の補数表現を利用する（最上位ビットが符号ビット）
    // e.g.
    // "5" -> 0b00000000000000000000000000000101
    // "-5" -> 0b11111111111111111111111111110001
    pub fn from_str(s: &str) -> Result<Self, ParseError> {
        // Parse時のState
        let mut state = State::Start;
        let mut bits: u32 = 0;
        let mut is_negative = false;

        for &b in s.as_bytes() {
            match (state, b) {
                (State::Start, b'+') | (State::Start, b'-') => {
                    state = State::InInteger;
                    if b == b'-' {
                        is_negative = true;
                    }
                }
                (State::Start | State::InInteger, b'0'..=b'9') => {
                    state = State::InInteger;
                    let digit = b - b'0';
                    // すでに入っている値を10倍して、新しい値を足す
                    bits = bits.checked_mul(10).ok_or(ParseError::Overflow)?;
                    bits = bits.checked_add(digit as u32).ok_or(ParseError::Overflow)?;
                }
                (State::InInteger, b'.') => {
                    // 小数点が見つかった場合は以降は小数部として処理
                    state = State::InFraction;
                }
                (State::InFraction, b'0'..=b'9') => {
                    // 小数部の数値
                    // 正数への変換なので特に何もしない
                    // 数値が入っていた場合は自動的に切り捨ての形になる
                }
                _ => {
                    // 予期しない入力（数値以外など）があった場合は回復可能エラーを戻す
                    return Err(ParseError::InvalidDigit);
                }
            }
        }

        if is_negative {
            bits = !bits + 1; // 2の補数表現にするため反転して1を足す
        }

        return Ok(RInt { bits });
    }

    // 基本はデバッグやテスト用かな
    pub fn from_u32(bits: u32) -> Self {
        RInt { bits }
    }

    // 10進表記の文字列に変換する
    pub fn to_string(&self) -> String {
        let sign = self.bits & 0b1000_0000_0000_0000_0000_0000_0000_0000;

        let value = if sign != 0 {
            // 負数の場合
            // 2の補数表現を解釈する
            let value = !self.bits + 1;
            // 10進数に変換
            // ここは一旦自前実装なしでu32のto_stringを利用
            // u32のto_stringは先頭ビットを解釈まではしてくれないのでそこは自分で対応
            format!("-{}", value.to_string())
        } else {
            // 正数の場合
            // そのまま10進数に変換
            self.bits.to_string()
        };

        value
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r = RInt::from_str("5");

        assert_eq!(r.unwrap().bits, 0b0000_0000_0000_0000_0000_0000_0000_0101);
    }

    #[test]
    fn test_new_minus() {
        let r = RInt::from_str("-5");

        assert_eq!(r.unwrap().bits, 0b1111_1111_1111_1111_1111_1111_1111_1011);
    }

    #[test]
    fn test_add_1() {
        let r1 = RInt::from_str("5").unwrap(); // 5 ->101
        let r2 = RInt::from_str("13").unwrap(); // 13 -> 1101

        let r3 = r1 + r2;

        assert_eq!(r3.bits, 0b0000_0000_0000_0000_0000_0000_0001_0010); // 18 -> 10010
    }

    #[test]
    fn test_add_2() {
        let r1 = RInt::from_str("-5").unwrap(); // -5 -> 10~//~101
        let r2 = RInt::from_str("13").unwrap(); // 13 -> 1101

        let r3 = r1 + r2;

        assert_eq!(r3.bits, 0b0000_0000_0000_0000_0000_0000_0000_1000); // 8 -> 1000
    }

    #[test]
    fn test_add_overflow1() {
        let r1 = RInt::from_u32(0b1111_1111_1111_1111_1111_1111_1111_1111);
        let r2 = RInt::from_str("1").unwrap();

        let r3 = r1 + r2;

        assert_eq!(r3.bits, 0b0000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_add_overflow2() {
        let r1 = RInt::from_u32(0b0111_1111_1111_1111_1111_1111_1111_1111);
        let r2 = RInt::from_str("1").unwrap();

        let r3 = r1 + r2;

        assert_eq!(r3.bits, 0b1000_0000_0000_0000_0000_0000_0000_0000);
    }

    #[test]
    fn test_sub() {
        let r1 = RInt::from_str("5").unwrap(); // 5 ->101
        let r2 = RInt::from_str("13").unwrap(); // 13 -> 1101

        let r3 = r1 - r2;

        assert_eq!(r3.bits, 0b1111_1111_1111_1111_1111_1111_1111_1000); // -8 -> 11111000
    }

    #[test]
    fn test_mul() {
        let r1 = RInt::from_str("5").unwrap(); // 5 ->101
        let r2 = RInt::from_str("13").unwrap(); // 13 -> 1101

        let r3 = r1 * r2;

        assert_eq!(r3.bits, 0b0000_0000_0000_0000_0000_0000_0100_0001); // 65 -> 1000001
    }

    #[test]
    fn test_div() {
        let r1 = RInt::from_str("13").unwrap(); // 5 ->1101
        let r2 = RInt::from_str("5").unwrap(); // 13 -> 101

        let r3 = r1 / r2;

        assert_eq!(r3.bits, 0b0000_0000_0000_0000_0000_0000_0010); // 2 -> 10
    }

    #[test]
    fn test_to_string() {
        let r = RInt::from_str("5").unwrap(); // 5 ->101

        assert_eq!(r.to_string(), "5");
    }

    #[test]
    fn test_to_string_negative() {
        let r = RInt::from_str("-25").unwrap(); // 5 ->101

        assert_eq!(r.to_string(), "-25");
    }
}
