git config --global core.symlinks true
cd ..
git submodule update --init --recursive
cd demo
python helper.py
