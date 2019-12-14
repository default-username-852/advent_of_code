mod computer;

use std::io::{stdin, BufRead, Write, BufWriter};
use crate::computer::*;
use std::{thread, iter};
use std::sync::{mpsc, Arc, Mutex};
use std::collections::HashMap;
use std::fs::File;
use std::fmt::{Display, Formatter, Error};
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
    let done = Arc::new(Mutex::new(false));
    let done2 = Arc::clone(&done);

    let join_handle = thread::spawn(move || {
        c.run(input_receiver, output_transmitter);
        *done2.lock().unwrap() = true;
    });

    let mut frame = 1;
    let mut squares: HashMap<(i64, i64), Square> = HashMap::new();

    loop {
        if *need_input.lock().unwrap() {
            let mut output = Vec::new();

            loop {
                match output_receiver.try_recv() {
                    Ok(t) => output.push(t),
                    Err(_) => break,
                }
            }

            let mut trimmed = Vec::new();

            for i in (0..output.len() / 3).map(|e| e * 3) {
                if output[i] == -1 {
                    println!("score: {}", output[i + 2]);
                } else {
                    trimmed.extend_from_slice(&[output[i], output[i + 1], output[i + 2]]);
                }
            }

            update(trimmed.as_slice(), &mut squares);

            //let left = squares.iter().filter(|e| *e.1 == Square::Block).count();

            //println!("{} {}", frame, left);

            //if left == 0 {
            //    break;
            //}

            let ball_pos = squares.iter().find(|e| *e.1 == Square::Ball).unwrap();
            let paddle_pos = squares.iter().find(|e| *e.1 == Square::Paddle).unwrap();

            let dir_to_move = ((ball_pos.0).0 - (paddle_pos.0).0).signum();

            input_transmitter.send(dir_to_move).unwrap();

            render(&squares, frame);

            frame += 1;

            thread::sleep(Duration::from_micros(10));
        }

        if *done.lock().unwrap() {
            break;
        }
    }

    let mut output = Vec::new();

    loop {
        match output_receiver.try_recv() {
            Ok(t) => output.push(t),
            Err(_) => break,
        }
    }

    let mut trimmed = Vec::new();

    for i in (0..output.len() / 3).map(|e| e * 3) {
        if output[i] == -1 {
            println!("score: {}", output[i + 2]);
        } else {
            trimmed.extend_from_slice(&[output[i], output[i + 1], output[i + 2]]);
        }
    }

    update(trimmed.as_slice(), &mut squares);

    render(&squares, frame);

    join_handle.join().unwrap();
}

fn update(in_data: &[i64], squares: &mut HashMap<(i64, i64), Square>) {
    for i in (0..in_data.len() / 3).map(|e| e * 3) {
        let tile_type = match in_data[i + 2] {
            0 => Square::Empty,
            1 => Square::Wall,
            2 => Square::Block,
            3 => Square::Paddle,
            4 => Square::Ball,
            _ => panic!("bajs"),
        };
        let coords = (in_data[i], in_data[i + 1]);

        squares.insert(coords, tile_type);
    }
}

fn render(squares: &HashMap<(i64, i64), Square>, frame: i64) {
    let min_x = 0;
    let max_x = 41;
    let min_y = 0;
    let max_y = 24;

    let mut boxes: Vec<Vec<Square>> = iter::repeat(
        iter::repeat(Square::Empty)
            .take((max_x - min_x + 1) as usize)
            .collect::<Vec<Square>>())
        .take((max_y - min_y + 1) as usize)
        .collect();

    for square in squares {
        boxes[((square.0).1 - min_y) as usize][((square.0).0 - min_x) as usize] = square.1.clone();
    }

    let file = File::create(format!("images/frame{}.png", frame)).unwrap();
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, (max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32);

    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    let data: Vec<u8> = boxes.iter()
        .map(|row| row.iter()
            .map(|e| e.color())
            .map(|e| vec![e.0, e.1, e.2])
            .flatten()
            .collect::<Vec<u8>>())
        .flatten()
        .collect();

    writer.write_image_data(&data[..]).unwrap();

    /*let mut output = File::create(format!("frame{}.ppm", frame)).unwrap();
    output.write(format!("P3\n{} {} 255\n", max_x - min_x + 1, max_y - min_y + 1).as_bytes()).unwrap();

    output.write(boxes.iter()
        .map(|e|
            e.iter()
                .map(|e2| e2.color())
                .map(|e2| format!("{} {} {}", e2.0, e2.1, e2.2))
                .collect::<Vec<String>>()[..]
                .join(" "))
        .collect::<Vec<String>>()[..].join("\n").as_bytes()).unwrap();

    output.flush().unwrap();*/
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Square {
    Empty,
    Wall,
    Block,
    Paddle,
    Ball,
}

impl Square {
    fn color(&self) -> (u8, u8, u8) {
        match self {
            Square::Empty => (255, 255, 255),
            Square::Wall => (0, 0, 0),
            Square::Block => (0, 255, 0),
            Square::Paddle => (255, 0, 0),
            Square::Ball => (0, 0, 255),
        }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", match self {
            Square::Empty => " ",
            Square::Block => "b",
            Square::Wall => "W",
            Square::Paddle => "p",
            Square::Ball => "o",
        })
    }
}