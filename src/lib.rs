#![allow(dead_code)]

mod abbreviation_detection;
mod py_wrappings;

use abbreviation_detection::*;
use std::{time::{self, Duration}, collections::BinaryHeap};
use csv;

pub fn test_speed(notes_dir: String, excl_dict_path: String, add_dict_path: String) {
    let mut notes = csv::Reader::from_path(notes_dir).unwrap();
    let mut writer = csv::Writer::from_path("./tmp/spellchecked_notes.csv").unwrap();
    let (excl_dict, _add_dict) = initialize_dicts(excl_dict_path, add_dict_path);

    let mut times = Vec::<Duration>::new();

    for note in notes.records() {
        let text = note.unwrap()[0].to_string();

        let start = time::Instant::now();
        let spellchecked = spellcheck_text(text, &excl_dict);
        let end = time::Instant::now();

        writer.write_record([spellchecked]).unwrap();

        times.push(end - start);
    }

    let sum: Duration = times
        .clone()
        .iter()
        .sum();
    let average = sum / times.len() as u32;

    let time_heap = BinaryHeap::from(times);
    let max = *time_heap.peek().unwrap();
    let min = *time_heap.iter().last().unwrap();

    println!("Average time: {:?}", average);
    println!("Maximum time: {:?}", max);
    println!("Minimum time: {:?}", min);
}