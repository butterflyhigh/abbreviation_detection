# How to use in Python

0. Make sure Cargo is installed
1. Create a VENV and activate it
2. `pip install maturin`
3. `maturin build`
4. `pip install target/wheels/(whatever's in here)`

Then the package will be activated in the VENV

At least it should be, if it isn't try `maturin develop`
