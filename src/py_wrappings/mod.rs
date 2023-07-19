use pyo3::prelude::*;
use crate as lib;

#[pyfunction]
fn initialize_dicts(excl_dict_path: String, add_dict_path: String) -> PyResult<(Vec<String>, Vec<String>)> {
    Ok(lib::initialize_dicts(excl_dict_path, add_dict_path))
}

#[pyfunction]
fn detect_acronyms(text: String, excl_dict: Vec<String>, add_dict: Vec<String>) -> Vec<String> {
    lib::detect_acronyms(text, &excl_dict, &add_dict)
}

#[pyfunction]
fn spellcheck_text(text: String, dict: Vec<String>) -> String {
    lib::spellcheck_text(text, &dict)
}

fn test_speed(notes_dir: String, excl_dict_path: String, add_dict_path: String) {
    lib::test_speed(notes_dir, excl_dict_path, add_dict_path)
}

#[pymodule]
fn abbreviation_detection(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(initialize_dicts, m)?)?;
    m.add_function(wrap_pyfunction!(detect_acronyms, m)?)?;
    m.add_function(wrap_pyfunction!(spellcheck_text, m)?)?;

    Ok(())
}