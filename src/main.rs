mod computer;
mod helpers;

use std::io::{stdin, BufRead};
use crate::computer::*;
use crate::helpers::*;
use std::thread;
use std::sync::{mpsc, Arc};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

fn main() {
    let stdin = stdin();

    let mut lines = stdin.lock().lines().map(|e| e.unwrap());

    let mut program: Vec<i64> = lines.next().unwrap().split(",").map(|e| e.parse::<i64>().unwrap()).collect();

    program[0] = 2;

    let c = Computer::new(program.clone());
    let (input_transmitter, input_receiver) = mpsc::channel();
    let (output_transmitter, output_receiver) = mpsc::channel();

    let need_input = Arc::clone(&c.waiting);

    let join_handle = thread::spawn(move || {
        c.run(input_receiver, output_transmitter);
    });

    let mut result = Vec::new();

    loop {
        match output_receiver.recv_timeout(Duration::from_secs(1)) {
            Ok(t) => result.push(t as u8 as char),
            Err(_) => break,
        }
    }

    println!("{}", result.iter().collect::<String>());

    let mut chars = result.iter();
    let mut grid = Vec::new();

    loop {
        let row = chars.by_ref().take_while(|e| **e != '\n').map(|e| match e {
            '.' => Square::Empty,
            '#' => Square::Scaffold,
            '^' => Square::Robot,
            a => panic!("{}", a),
        }).collect::<Vec<Square>>();

        if row == vec![] {
            break;
        }

        grid.push(row);
    }

    let lazy_grid: HashSet<Point> = grid.iter()
        .enumerate()
        .flat_map(|row| row.1.iter()
            .enumerate()
            .filter(|e| *e.1 != Square::Empty)
            .map(move |e| (e.0 as i64, row.0 as i64)))
        .map(|e| Point::from(e))
        .collect();

    let mut robot_pos: Point = grid.iter()
        .enumerate()
        .find_map(|row| row.1.iter()
            .enumerate()
            .find_map(|e| if *e.1 == Square::Robot { Some(e.0) } else { None })
            .map(|e| (e as i64, row.0 as i64)))
        .unwrap().into();
    let mut facing = Direction::Up;

    loop {
        let mut steps = 0;
        loop {
            let new_pos = robot_pos + facing.offset();
            if !lazy_grid.contains(&new_pos) {
                break;
            }
            steps += 1;
            robot_pos = new_pos;
        }

        print!("{},", steps);

        let can_turn_left = lazy_grid.contains(&(robot_pos + facing.rotate_left ().offset()));
        let can_turn_right= lazy_grid.contains(&(robot_pos + facing.rotate_right().offset()));

        if can_turn_left && can_turn_right {
            panic!("bajs");
        } else if can_turn_left {
            facing = facing.rotate_left();
            print!("L,");
        } else if can_turn_right {
            print!("R,");
            facing = facing.rotate_right();
        } else {
            break;
        }
    }

    //println!();

    for c in "A,B,A,B,C,C,B,C,B,A\nR,12,L,8,R,12\nR,8,R,6,R,6,R,8\nR,8,L,8,R,8,R,4,R,4\nn\n".as_bytes() {
        input_transmitter.send(*c as i64).unwrap();
    }

    loop {
        match output_receiver.recv() {
            Ok(t) => {
                if t < std::u8::MAX as i64 {
                    print!("{}", t as u8 as char);
                } else {
                    println!("{}", t);
                }
            },
            Err(_) => break,
        }
    }

    join_handle.join().unwrap();
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum Square {
    Empty,
    Scaffold,
    Robot,
}