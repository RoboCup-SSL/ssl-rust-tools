#!/bin/bash

mkdir -p venv

pushd venv

# conan virtualenv creation. Will build the libraries if they are out
# of date/missing.
conan install .. --build=missing -g virtualrunenv -s compiler.libcxx=libstdc++

# These commands are needed so that the header files can be found,
# which is necessary for bindgen to run.
cp activate_run.sh activate_run.sh.tmp
grep '^LD_LIBRARY_PATH' activate_run.sh | sed 's/lib":/include":/g' | sed 's/LD_LIBRARY_PATH/CPLUS_INCLUDE_PATH/g' >> activate_run.sh.tmp
mv activate_run.sh.tmp activate_run.sh
echo 'export CPLUS_INCLUDE_PATH' >> activate_run.sh

# This adds the LD_LIBRARY_PATH to the rust linker search path.
#
# Without this the lib/application may fail to link properly, or may
# link against the incorrect library version.
cat << 'EOF' >> activate_run.sh
function gen_rustflags {
  IFS=':' read -r -a linker_search_paths <<< "$LD_LIBRARY_PATH"
  linker_search_paths=$(printf -- "-L%s "  "${linker_search_paths[@]}")
  export RUSTFLAGS="-C link-args=${linker_search_paths}"
}

gen_rustflags
EOF

popd
