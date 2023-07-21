# How to use in Python

0. Make sure Cargo is installed
1. Create a VENV and activate it
2. `pip install maturin`
3. `maturin build`
4. `pip install target/wheels/(whatever's in here)`

Then the package will be activated in the VENV

At least it should be, if it isn't try `maturin develop`

Available functions:
* initialize_dicts(excl_dict_path, add_dict_path): This returns a tuple of two lists of Strings. excl_dict_path is a csv file of non-acronym words, add_dict_path is a csv file of known medical abbreviations. In this repo, it's /data/dict/excl_dict.csv and /data/dict/med_abbr.csv.
* detect_acronyms(text, excl_dict, add_dict): This returns a list of detected acronyms. I'm planning on editing it to return a better format that also represents where the acronyms are in the text, since it's a pain to chain into a pipeline as-is.
* spellcheck_text(text, dict): This does what it says on the tin. I'd recommend concatenating the excl_dict and add_dict for the dict variable.
