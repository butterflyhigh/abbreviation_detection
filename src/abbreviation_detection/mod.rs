#![allow(dead_code)]

use std::{collections::{BTreeMap, HashMap}, cmp::min, ops::Range};
use itertools::Itertools;
use regex::Regex;
use csv;

pub fn spellcheck_text(text: String, freqlist: &BTreeMap<String, String>, dict: &Vec<String>) -> String {
    let mut spellchecked_text = String::new();
    let sep_re = Regex::new("[^\\w\\/\\[\\]\\(\\)\\*|]").unwrap();
    let words = sep_re.split(&text).map(|x| x.to_string()).collect::<Vec<String>>();

    for word in words {
        if word.len() <= 2 {
            spellchecked_text += &format!(" {word}");
        } else {
            match find_levenshtein_matches(word.clone(), 1, &freqlist, dict) {
                LevenshteinMatch::ExactMatch(txt) => {
                    spellchecked_text += &format!(" {txt}");
                },
                LevenshteinMatch::ApproximateMatches(matches) => {
                    if matches.len() < 1 {
                        spellchecked_text += &format!(" {word}");
                    } else if matches.len() < 2 {
                        spellchecked_text += &format!(" {}", matches[0]);
                    } else {
                        spellchecked_text += &format!(" {}", find_most_likely_word(matches, text.clone()));
                    }
                }
            }
        }
    }

    spellchecked_text
}

pub fn find_levenshtein_matches(word: String, threshold: usize, freqlist: &BTreeMap<String, String>, _dict: &Vec<String>) -> LevenshteinMatch {
    let cleaned_word = word.to_lowercase().trim().to_string();

    let freqlist_matches = levenshtein_match_dict(cleaned_word.clone(), threshold, &freqlist.iter().map(|(x, _)| x.to_owned()).collect_vec(), true);  

    /* match freqlist_matches {
        LevenshteinMatch::ApproximateMatches(v) => {
            if v.len() < 1 {
                return levenshtein_match_dict(cleaned_word, threshold, dict, false);
            } else {
                return LevenshteinMatch::ApproximateMatches(v)
            }
        }
        LevenshteinMatch::ExactMatch(_) => {
            return freqlist_matches;
        }
    } */

    freqlist_matches
}

pub fn levenshtein_match_dict(cleaned_word: String, threshold: usize, dict: &Vec<String>, break_at_first: bool) -> LevenshteinMatch {
    match dict.binary_search(&cleaned_word) {
        Ok(_) => {
            return LevenshteinMatch::ExactMatch(cleaned_word);
        }
        Err(_) => {
            let mut res_vec = Vec::<String>::new();

            // first search freqlist bc its more reliable
            for row in dict {
                let dict_word = row;
                if cleaned_word.eq(dict_word) {
                    return LevenshteinMatch::ExactMatch(cleaned_word);
                } else {
                    if damerau_levenshtein(&cleaned_word, &dict_word, threshold) <= threshold {
                        res_vec.push(dict_word.to_owned());

                        if break_at_first {
                            return LevenshteinMatch::ApproximateMatches(res_vec);
                        }
                    }
                }
            }

            return LevenshteinMatch::ApproximateMatches(res_vec);
        }
    }
}

pub fn build_freqlist(path: String) -> BTreeMap<String, String> {
    let map = csv::Reader::from_path(path)
        .unwrap()
        .records()
        .map(|x| x
            .unwrap()
            .iter()
            .map(|y| y.to_string())
            .collect_tuple()
            .unwrap()
        )
        .collect::<BTreeMap<String, String>>();

    map
}

pub fn find_most_likely_word(matches: Vec<String>, source_text: String) -> String {
    let map = build_freq_map(source_text.split(" ").map(|x| x.to_string()).collect::<Vec<String>>());
    let mut max = 0;
    let mut most_frequent_word = String::new();
    let mut most_frequent_match = String::new();

    for (word, freq) in map {
        if freq > max {
            max = freq;
            most_frequent_word = word.clone();

            if matches.contains(&word) {
                most_frequent_match = word;
            }
        }
    }
    
    if !most_frequent_match.is_empty() {
        return most_frequent_match;
    } else {
        return most_frequent_word;
    }
}

pub fn build_freq_map(words: Vec<String>) -> HashMap<String, usize> {
    let mut map = HashMap::<String, usize>::new();

    for word in words {
        match map.get_mut(&word) {
            Some(freq) => {
                *freq += 1;
            }
            None => {
                map.insert(word, 1);
           }
        }
    }

    map
}

fn replace_with_space(text: &String, re: &Regex) -> (Vec<Range<usize>>, String) {
    let matches = re.find_iter(text);
    let ranges = matches.map(|x| x.range()).collect::<Vec<Range<usize>>>();

    // replace matches with space
    let mut repd = String::new();
    for (i, c) in text.char_indices() {
        match ranges.iter().find(|range| range.contains(&i)) {
            Some(_) => {
                repd.push(' ');
            },
            None => {
                repd.push(c);
            },
        }
    }

    (ranges, repd)
}

pub fn detect_acronyms(text: String, excl_dict: &Vec<String>, add_dict: &Vec<String>) -> HashMap<(usize, usize), String> {
    let rm_re = Regex::new("[\\.\\?!,\\(\\)\\d:;-_+=|]").unwrap();
    let tag_re = Regex::new("\\*\\*[^\\[]*\\[[^\\]]*\\]").unwrap();
    let vowel_re = Regex::new("[aeuoi]").unwrap();

    // returns the ranges of char indices that were replaced and the string with matched characters replaced with spaces
    let (mut ranges, mapped_no_tags) = replace_with_space(&text, &tag_re);
    let (cleaned_ranges, mapped_clean_text) = replace_with_space(&mapped_no_tags, &rm_re);
    let words = mapped_clean_text.split(" ").map(|x| x.to_owned()).collect::<Vec<String>>();

    ranges.extend(cleaned_ranges);

    let mut impossible_bigrams_rdr = csv::Reader::from_path("./data/dict/bad_bigrams.csv").unwrap();
    let mut impossible_bigrams_re_match = String::new();
    let mut bigram_rcrds = impossible_bigrams_rdr.records();
    impossible_bigrams_re_match.push_str(&bigram_rcrds.next().unwrap().unwrap()[0]);

    for bigram in bigram_rcrds {
        let txt = bigram.unwrap()[0].to_string();
        impossible_bigrams_re_match.push_str(&format!("|{}", txt));
    }

    let bigram_re = Regex::new(&format!("[^ ]*({})[^ ]*", impossible_bigrams_re_match)).unwrap();

    let mut problematic_rdr = csv::Reader::from_path("./data/dict/problematic.csv").unwrap();
    let problematic: Vec<String> = problematic_rdr.records().map(|x| x.unwrap()[0].to_string()).collect();

    // Rules:
    // * Acronyms will always be < 5 characters, unless they fit an "always" rule
    // * Acronyms won't be in dict
    // * Every word in the medical abbreviation dictionary is an acronym
    // * Every word with an illegal bigram is an acronym

    // TODO: Make it so for words in both the add dict and the excl dict, decides based on popularity

    let abbrs: BTreeMap<usize, Option<String>> = words
        .iter()
        .enumerate()
        .map(|(i, word)| (i, word.to_lowercase().trim().to_string()))
        .map(|(i, word)| 
            if word.len() > 1 { 
                (i, Some(word)) 
            } else { 
                (i, None) 
            }
        ) // len >= 1 is so important it gets its own filter
        .map(|(i, word)| 
            {
                match word {
                    Some(abbr) => {
                        if  ((
                                abbr.len() < 5 && 
                                !excl_dict.contains(&abbr)
                            ) || (
                                bigram_re.is_match(&abbr) ||
                                !vowel_re.is_match(&abbr) ||
                                add_dict.contains(&abbr)
                            )) && !problematic.contains(&abbr) {
                                (i, Some(abbr))
                        } else {
                            (i, None)
                        }
                    }
                    None => (i, None)
                }
            }
        )
        .collect();

    convert_to_hashmap(&abbrs, words)
}

fn convert_to_hashmap(input_map: &BTreeMap<usize, Option<String>>, words: Vec<String>) -> HashMap<(usize, usize), String> {
    let mut result_map = HashMap::<(usize, usize), String>::new();
    let mut start_idx = 0;
    let mut end_idx;

    for (key, value) in input_map {
        match value {
            Some(word) => {
                let word_start = words[*key].find(word).unwrap_or(0);
                start_idx += word_start;
                end_idx = start_idx + word.len();
                result_map.insert((start_idx, end_idx), word.to_owned());
                start_idx = end_idx + 1;
            }
            None => {
                let not_abbr_word = &words[*key];
                start_idx += not_abbr_word.len() + 1;
            }
        }
    }

    result_map
}

pub fn initialize_dicts(excl_dict_path: String, add_dict_path: String) -> (Vec<String>, Vec<String>) {
    let mut excl_reader = csv::Reader::from_path(excl_dict_path).unwrap();
    let mut add_reader = csv::Reader::from_path(add_dict_path).unwrap();

    let excl_dict = excl_reader
        .records()
        .map(|x| x.unwrap()[0].to_string().to_lowercase().trim().to_string())
        .collect::<Vec<String>>();

    let add_dict = add_reader
        .records()
        .map(|x| x.unwrap()[0].to_string().to_lowercase().trim().to_string())
        .collect::<Vec<String>>();

    (excl_dict, add_dict)
}

pub fn damerau_levenshtein(s: &str, t: &str, breakpoint: usize) -> usize {
    // get length of unicode chars
    let len_s = s.chars().count();
    let len_t = t.chars().count();
    let max_distance = len_t + len_s;

    // initialize the matrix
    let mut mat: Vec<Vec<usize>> = vec![vec![0; len_t + 2]; len_s + 2];
    mat[0][0] = max_distance;
    for i in 0..(len_s + 1) {
        mat[i+1][0] = max_distance;
        mat[i+1][1] = i;
    }
    for i in 0..(len_t + 1) {
        mat[0][i+1] = max_distance;
        mat[1][i+1] = i;
    }

    let mut char_map: HashMap<char, usize> = HashMap::new();
    // apply edit operations
    for (i, s_char) in s.chars().enumerate() {
        let mut db = 0;
        let i = i + 1;
        
        for (j, t_char) in t.chars().enumerate() {
            let j = j + 1;
            let last = *char_map.get(&t_char).unwrap_or(&0);

            let cost = if s_char == t_char { 0 } else { 1 };
            mat[i+1][j+1] = min4(
                mat[i+1][j] + 1,     // deletion
                mat[i][j+1] + 1,     // insertion 
                mat[i][j] + cost,    // substitution
                mat[last][db] + (i - last - 1) + 1 + (j - db - 1) // transposition
            );

            // that's like s_char == t_char but more efficient
            if cost == 0 {
                db = j;
            } else if mat[len_s + 1][len_t + 1] > breakpoint {
                return mat[len_s + 1][len_t + 1] // i guess this is the thing?
            }
        }

        char_map.insert(s_char, i);
    }

    return mat[len_s + 1][len_t + 1];
}

pub fn min4(a: usize, b: usize, c: usize, d: usize) -> usize {
   return min(min(min(a, b), c), d); 
}

pub fn generate_problematic() {
    let mut rdr = csv::Reader::from_path("./data/dict/top_10k.csv").unwrap();
    let mut writer = csv::Writer::from_path("./data/dict/problematic.csv").unwrap();
    let (excl_dict, add_dict) = initialize_dicts("./data/dict/excl_dict.csv".into(), "./data/dict/med_abbr.csv".into());

    for row in rdr.records() {
        let word = row.unwrap()[0].to_string();

        match detect_acronyms(word, &excl_dict, &add_dict).iter().last() {
            Some((_, abbr)) => {
                writer.write_record([abbr]).unwrap();
            }
            None => {}
        }
    }
}

#[derive(Debug)]
pub enum LevenshteinMatch {
    ExactMatch(String),
    ApproximateMatches(Vec<String>)
}