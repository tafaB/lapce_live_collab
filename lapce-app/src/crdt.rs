use std::cmp::Ordering;
use std::usize;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

// structure to be created for wach of the characters
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Id{
    //[<number/digit> , <Bering/user>]
    pub number: u32,
    pub user: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Character{
    pub pos_id: Vec<Id>,
    pub action_id: u32,
    //the actual character entered
    pub value: char,
}


// -------------------------------------------------
// |         GENERATE POSITION INDETIFIER          |
// -------------------------------------------------
fn pos_id_to_decimal(x: Vec<Id>) -> Decimal {
    let mut res: Vec<u32> = Vec::new();
    for i in x {
        res.push(i.number);
    }
    let decimal: Decimal = res
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &digit)| Decimal::from(digit) * Decimal::new(10_i64.pow(i as u32), 0))
        .sum();
    let result: Decimal = decimal / Decimal::new(10_i64.pow(res.len() as u32), 0);
    result.normalize()
}

fn add(x: Decimal, dif: Decimal) -> Decimal {
    let mut res: Decimal = Decimal::new(1, 1);
    let mut s = dif.to_string();
    s.remove(0);
    s.remove(0);
    for i in s.chars() {
        res /= Decimal::from_str("10.0").unwrap();
        if i != '0' {
            if i != '1' {
                res *= Decimal::from_str("10.0").unwrap();
            }
            break;
        }
    }
    println!("{} + {}", x, res);
    (x + res).normalize()
}

fn decimal_to_pos_id_vec(start: Vec<Id>, x: Decimal, end: Vec<Id>, new_user_id: u32) -> Vec<Id> {
    let number_str = x.to_string();
    let digits = number_str.chars().skip(2).map(|c| c.to_digit(10).unwrap()).collect::<Vec<u32>>();
    let mut res: Vec<Id> = Vec::new();
    let mut j: usize = start.len();
    for i in 0..digits.len() {
        if start.len() > i && start[i].number == digits[i] {
            res.push(start[i].clone());
        } else if end.len() > i && end[i].number == digits[i] {
            res.push(end[i].clone());
        } else {
            j = i;
            break;
        }
    }
    for i in j..digits.len() {
        res.push(Id {
            number: digits[i],
            user: new_user_id,
        });
    }
    res
}


pub fn generate_pos_id(pos_id_1:Vec<Id>, pos_id_2:Vec<Id>, new_user_id: u32) -> Vec<Id>{
    let mut res:Vec<Id> = Vec::new();
    //first character entry
    //not needed, but counting will start from 0.01 instead of 0.1
    if pos_id_1.len() == 0 && pos_id_2.len() == 0 {
        res.push(Id{
            number: 1,
            user: new_user_id,
        });
        return res;
    }

    let mut pos_id_1 = pos_id_1;
    let mut pos_id_2 = pos_id_2;
    let mut begin = Id{
        number: 0,
        user: new_user_id,
    };
    let mut end = Id{
        number: 9,
        user: new_user_id,
    };
    if pos_id_1.len() == 0 {
        pos_id_1.push(begin.clone());
    }
    if pos_id_2.len() == 0 {
        pos_id_2.push(end.clone());
    }

    if pos_id_1.len() !=0 {
        begin.number = pos_id_1[0].number;
        begin.user = pos_id_1[0].user;
    }
    if pos_id_2.len() !=0 {
        end.number = pos_id_2[0].number;
        end.user = pos_id_2[0].user;
    }

    // ---------------------------------------
    // Decision making :
    // ---------------------------------------
    if begin.number == end.number && begin.user == end.user {
        res.push(begin);
        res.extend(generate_pos_id(pos_id_1[1..].to_vec(), pos_id_2[1..].to_vec(), new_user_id));
    }
    else if begin.number == end.number && begin.user < end.user {
        res.push(begin);
        res.extend(generate_pos_id(pos_id_1[1..].to_vec(), Vec::new(), new_user_id));
    }
    else if begin.number != end.number {
        let x_decimal:Decimal = pos_id_to_decimal(pos_id_1.clone());
        let y_decimal:Decimal = pos_id_to_decimal(pos_id_2.clone());
        let dif:Decimal = (x_decimal - y_decimal).abs();
        println!("x= {} - y = {} = dif ={}", x_decimal, y_decimal, dif);
        res.extend(decimal_to_pos_id_vec(pos_id_1.clone(), add(x_decimal,dif), pos_id_2.clone(), new_user_id));
    }
    else {
        panic!("Ordering is done wrong");
    }
    return res;
}


// -------------------------------------------------
// |         COMPARE POSITION INDENTIFIERS         |
// -------------------------------------------------

pub fn comp_id(a:Id, b:Id) -> Ordering {
    if a.number < b.number {
        return Ordering::Less;
    }
    if a.number > b.number {
        return Ordering::Greater;
    }
    if a.user < b.user {
        return Ordering::Less;
    }
    if a.user > b.user {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

pub fn comp_pos(a:&Vec<Id>, b:&Vec<Id>) -> Ordering {
    for i in 0..a.len().min(b.len()) {
        let res = comp_id(a[i].clone(), b[i].clone());
        if res != Ordering::Equal {
            return res;
        }
    }
    if a.len() < b.len() {
        return Ordering::Less;
    }
    if a.len() > b.len() {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

pub fn comp_character(a:&Character, b:&Character) -> Ordering{
    return comp_pos(&a.pos_id, &b.pos_id);
}