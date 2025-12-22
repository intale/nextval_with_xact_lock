#!/bin/bash

script_dir=$(dirname $0)

PGVER=16 ./$script_dir/run_tests.sh "$@" || exit 0
PGVER=17 ./$script_dir/run_tests.sh "$@" || exit 0
PGVER=18 ./$script_dir/run_tests.sh "$@" || exit 0
