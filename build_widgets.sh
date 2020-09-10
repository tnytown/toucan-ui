#!/usr/bin/env bash
set -euf -o pipefail

DIR=widgets_build
mkdir $DIR && pushd $DIR
qmake ../3p/DeclarativeWidgets/declarativewidgets.pro
make
popd
