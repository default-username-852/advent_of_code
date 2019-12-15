mod computer;

use std::io::{stdin, BufRead, Write, BufWriter};
use crate::computer::*;
use std::{thread, iter};
use std::sync::{mpsc, Arc, Mutex};
use std::collections::{HashMap, VecDeque, HashSet};
use std::fs::File;
use std::fmt::{Display, Formatter, Error};
use std::time::Duration;
use std::ops::{Sub, Add, Div, Mul};

#[derive(Clone, Debug, Hash, Copy, Eq, PartialEq)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    fn euclidian(&self, other: &Point) -> f64 {
        (((self.x - other.x) as f64).powi(2) + ((self.y - other.y) as f64).powi(2)).sqrt()
    }

    fn manhattan(&self, other: &Point) -> i64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    fn neighbours(&self) -> Vec<Point> {
        let mut out = Vec::new();

        out.push(*self + Point::from((0, 1)));
        out.push(*self + Point::from((0, -1)));
        out.push(*self + Point::from((1, 0)));
        out.push(*self + Point::from((-1, 0)));

        out
    }
}

impl From<(i64, i64)> for Point {
    fn from(a: (i64, i64)) -> Self {
        Self { x: a.0, y: a.1 }
    }
}

impl Mul<i64> for Point {
    type Output = Point;

    fn mul(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Point> for i64 {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Point {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Div<i64> for Point {
    type Output = Point;

    fn div(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<Point> for i64 {
    type Output = Point;

    fn div(self, rhs: Point) -> Self::Output {
        Point {
            x: rhs.x / self,
            y: rhs.y / self,
        }
    }
}

impl Add<i64> for Point {
    type Output = Point;

    fn add(self, rhs: i64) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn main() {
    let stdin = stdin();

    let mut lines = stdin.lock().lines().map(|e| e.unwrap());

    let mut program: Vec<i64> = lines.next().unwrap().split(",").map(|e| e.parse::<i64>().unwrap()).collect();

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

    let mut position = Point::new(0, 0);
    let mut squares = HashMap::new();

    squares.insert(position, Square::Empty);

    pretty_print(&squares, &position);

    let mut steps = 0;

    loop {
        let mut closest_point = Point::from((std::i32::MAX as i64, std::i32::MAX as i64));
        let mut best_dist = closest_point.manhattan(&position);

        for square in &squares {
            if *square.1 == Square::Wall {
                continue;
            }

            let neighbours = square.0.neighbours();
            for neighbour in &neighbours {
                if !squares.contains_key(&neighbour) {
                    let dist = position.manhattan(&neighbour);

                    if dist < best_dist {
                        best_dist = dist;
                        closest_point = *neighbour;
                    }
                }
            }
        }

        let path = bfs_path(&position, &closest_point, &squares);

        let mut movement = Vec::new();

        for i in 1..path.len() {
            let from = path[i - 1];
            let to = path[i];

            movement.push(match to - from {
                Point { x: 1, y: 0 } => 4,
                Point { x: -1, y: 0 } => 3,
                Point { x: 0, y: 1 } => 1,
                Point { x: 0, y: -1 } => 2,
                _ => panic!("{:?} {:?}", from, to),
            });
        }

        steps += movement.len();

        if movement.len() == 0 {
            break; //fulhack för att avgöra om den är klar
        }

        for command in movement {
            input_transmitter.send(command).unwrap();

            let offset = Point::from(match command {
                3 => (-1, 0),
                4 => (1, 0),
                1 => (0, 1),
                2 => (0, -1),
                _ => panic!("prutt"),
            });

            let tile_moving_to = position + offset;

            let response = output_receiver.recv().unwrap();

            match response {
                0 => {
                    squares.insert(tile_moving_to, Square::Wall);
                }
                1 => {
                    squares.insert(tile_moving_to, Square::Empty);
                    position = tile_moving_to;
                }
                2 => {
                    squares.insert(tile_moving_to, Square::Goal);
                    println!("found goal");
                    position = tile_moving_to;
                }
                _ => { panic!("bajs") }
            }

            pretty_print(&squares, &position);
        }
    }

    println!("took {} steps", steps);
    println!("finished mapping");

    let origin = Point::new(0, 0);
    let goal = *squares.iter()
        .find(|e| *e.1 == Square::Goal).unwrap().0;

    let p1 = bfs_path(&origin, &goal, &squares).len() - 1;
    let cost = bfs_len(&goal, &squares);

    println!("part 1: {}", p1);
    println!("part 2: {}", cost);

    join_handle.join().unwrap();
}

fn dimensions(squares: &HashMap<Point, Square>) -> ((i64, i64), (i64, i64)) {
    let mut min_x = 0;
    let mut max_x = 0;
    let mut min_y = 0;
    let mut max_y = 0;

    for square in squares {
        min_x = min_x.min((square.0).x);
        max_x = max_x.max((square.0).x);
        min_y = min_y.min((square.0).y);
        max_y = max_y.max((square.0).y);
    }

    ((min_x, max_x), (min_y, max_y))
}

fn pretty_print(squares: &HashMap<Point, Square>, robot_pos: &Point) {
    let ((min_x, max_x), (min_y, max_y)) = dimensions(squares);

    let mut boxes: Vec<Vec<Square>> = iter::repeat(
        iter::repeat(Square::Unexplored)
            .take((max_x - min_x + 1) as usize)
            .collect::<Vec<Square>>())
        .take((max_y - min_y + 1) as usize)
        .collect();

    for square in squares {
        boxes[((square.0).y - min_y) as usize][((square.0).x - min_x) as usize] = square.1.clone();
    }

    boxes[(robot_pos.y - min_y) as usize][(robot_pos.x - min_x) as usize] = Square::Robot;

    println!("{}", boxes.iter()
        .map(|row| row.iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()[..].join(" "))
        .collect::<Vec<String>>()[..].join("\n"));
    println!();
}

fn bfs_len(start: &Point, squares: &HashMap<Point, Square>) -> u64 {
    let mut q = VecDeque::new();
    let mut discovered = HashSet::new();

    discovered.insert(*start);

    q.push_back((*start, 0));

    let mut max = 0;

    while q.len() > 0 {
        let v = q.pop_front().unwrap();

        for neighbour in v.0.neighbours() {
            if !discovered.contains(&neighbour) {
                if let Some(t) = squares.get(&neighbour) {
                    if *t == Square::Empty {
                        discovered.insert(neighbour);
                        max = max.max(v.1 + 1);
                        q.push_back((neighbour, v.1 + 1));
                    }
                }
            }
        }
    }

    max
}

fn bfs_path(start: &Point, end: &Point, squares: &HashMap<Point, Square>) -> Vec<Point> {
    let mut path = Vec::new();

    //Do BFS to find path to closest unexplored tile
    let mut q = VecDeque::new();
    let mut discovered = HashSet::new();

    discovered.insert(*start);

    q.push_back((*start, vec![*start]));

    while q.len() > 0 {
        let v = q.pop_front().unwrap();
        if v.0 == *end {
            path = v.1;
            break;
        } else {
            for neighbour in v.0.neighbours() {
                if (squares.contains_key(&neighbour) || neighbour == *end) && !discovered.contains(&neighbour) {
                    if !(neighbour == *end) {
                        if *squares.get(&neighbour).unwrap() == Square::Empty || *squares.get(&neighbour).unwrap() == Square::Goal {
                            discovered.insert(neighbour);
                            let mut new_path = v.1.clone();
                            new_path.push(neighbour);
                            q.push_back((neighbour, new_path));
                        }
                    } else {
                        discovered.insert(neighbour);
                        let mut new_path = v.1.clone();
                        new_path.push(neighbour);
                        q.push_back((neighbour, new_path));
                    }
                }
            }
        }
        //println!("{:?} {:?}", v.0, q);
    }

    path
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Square {
    Unexplored,
    Empty,
    Wall,
    Goal,
    Robot,
}

impl Square {}

impl Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", match self {
            Square::Empty => ".",
            Square::Wall => "#",
            Square::Goal => "G",
            Square::Unexplored => " ",
            Square::Robot => "R",
        })
    }
}
