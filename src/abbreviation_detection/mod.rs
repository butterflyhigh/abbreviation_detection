use std::{collections::HashMap, cmp::min};
use regex::Regex;
use csv;

pub fn spellcheck_text(text: String, dict: &Vec<String>) -> String {
    let mut spellchecked_text = String::new();
    let sep_re = Regex::new("[^\\w\\/\\[\\]\\(\\)\\*|]").unwrap();
    let special_re = Regex::new("[^\\/\\[\\]\\(\\)\\*|]").unwrap();
    let words = sep_re.split(&text).map(|x| x.to_string()).collect::<Vec<String>>();

    for word in words {
        if word.len() <= 1 || special_re.is_match(&word) {
            spellchecked_text += &format!(" {word}");
        } else {
            match find_levenshtein_matches(word.clone(), 1, dict) {
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

pub fn detect_acronyms(text: String, excl_dict: &Vec<String>, add_dict: &Vec<String>) -> Vec<String> {
    let rm_re = Regex::new("[\\.\\?!,\\(\\)\\d:;-_+=|]").unwrap();
    let tag_re = Regex::new("\\*\\*[^\\[]*\\[[^\\]]*\\]").unwrap();
    let vowel_re = Regex::new("[aeuoi]").unwrap();
    let no_tags = tag_re.replace_all(&text, "");
    let clean_text = rm_re.replace_all(&no_tags, " ");
    let words = clean_text.split(" ");

    // Rules:
    // * Acronyms will always be < 5 characters
    // * All words without vowls are acronyms
    // * Acronyms won't be in dict

    let abbr: Vec<String> = words
        .map(|word| word.to_lowercase().trim().to_string())
        .filter(|word| 
            word.len() > 1 
            && word.len() < 5
            && !excl_dict.contains(word)
            && (
                add_dict.contains(word)
                || !vowel_re.is_match(word)
            )
        )
        .collect();

    abbr
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

pub fn find_levenshtein_matches(word: String, threshold: usize, dict: &Vec<String>) -> LevenshteinMatch {
    let cleaned_word = word.to_lowercase().trim().to_string();

    match dict.binary_search(&cleaned_word) {
        Ok(_) => {
            return LevenshteinMatch::ExactMatch(cleaned_word);
        }
        Err(_) => {
            let mut res_vec = Vec::<String>::new();

            for row in dict {
                if cleaned_word.eq(row) {
                    return LevenshteinMatch::ExactMatch(cleaned_word);
                } else {
                    if damerau_levenshtein(&word, &row, threshold) <= threshold {
                        res_vec.push(row.to_owned());
                    }
                }
            }

            return LevenshteinMatch::ApproximateMatches(res_vec);
        }
    }
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

#[derive(Debug)]
pub enum LevenshteinMatch {
    ExactMatch(String),
    ApproximateMatches(Vec<String>)
}