extern crate regex;

use regex::Regex;
use std::io::{self, BufRead};
use std::iter::Iterator;
use std::process::{Command, Stdio};
use std::env;

mod xdo_handler;

fn main() {
    let args: Vec<String> = env::args().collect();
    let acceleration: f32;
    if args.len() > 1 {
        acceleration = args[1].parse::<f32>().unwrap_or(1.0);
    } else {
        acceleration = 1.0;
    }

    let output = Command::new("stdbuf")
        .arg("-o0")
        .arg("libinput")
        .arg("debug-events")
        .stdout(Stdio::piped())
        .spawn()
        .expect("can not exec libinput")
        .stdout
        .expect("libinput has no stdout");

    let mut xdo_handler = xdo_handler::start_handler();

    let mut xsum: f32 = 0.0;
    let mut ysum: f32 = 0.0;
    let pattern = Regex::new(r"[\s]+|/|\(").unwrap();

    for line in io::BufReader::new(output).lines() {
        let line = line.unwrap();
        if line.contains("GESTURE_") {
            // event10  GESTURE_SWIPE_UPDATE +3.769s	4  0.25/ 0.48 ( 0.95/ 1.85 unaccelerated)
            let mut parts: Vec<&str> = pattern.split(&line).filter(|c| !c.is_empty()).collect();
            let action = parts[1];
            if action == "GESTURE_SWIPE_UPDATE" && parts.len() != 9 {
                parts.remove(2);
            }
            let finger = parts[3];
            if finger != "3" && !action.starts_with("GESTURE_HOLD"){
                xdo_handler.mouse_up(1);
                continue;
            }
            let cancelled = parts.len() > 4 && parts[4] == "cancelled";

            match action {
                "GESTURE_SWIPE_BEGIN" => {
                    xsum = 0.0;
                    ysum = 0.0;
                    xdo_handler.mouse_down(1);
                }
                "GESTURE_SWIPE_UPDATE" => {
                    let x: f32 = parts[4].parse().unwrap();
                    let y: f32 = parts[5].parse().unwrap();
                    xsum += x * acceleration;
                    ysum += y * acceleration;
                    if xsum.abs() > 1.0 || ysum.abs() > 1.0 {
                        xdo_handler.move_mouse_relative(xsum as i32, ysum as i32);
                        xsum = 0.0;
                        ysum = 0.0;
                    }
                }
                "GESTURE_SWIPE_END" => {
                    xdo_handler.move_mouse_relative(xsum as i32, ysum as i32);
                    if cancelled {
                        xdo_handler.mouse_up(1);
                    } else {
                        xdo_handler.mouse_up_delay(1, 600);
                    }
                }
                "GESTURE_HOLD_BEGIN" => {
                    // Ignore
                }
                "GESTURE_HOLD_END" => {
                    // Ignore accidental holds when repositioning
                    if !cancelled {
                        xdo_handler.mouse_up(1);
                    }
                }
                _ => {
                    // GESTURE_PINCH_*,
                    xdo_handler.mouse_up(1);
                }
            }
        } else {
            xdo_handler.mouse_up(1);
        }
    }
}
