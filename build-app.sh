#!/bin/sh
set -e

# Build app to object file.
app="$1"
app_roc="./apps/$app.roc"
if [ -f "$app_roc" ]; then
	./roc/target/release/roc build --opt-size --no-link --precompiled-host --backend thumbv7emhf $app_roc
else
	echo "$app is not an app!"
fi

# Convert it to a static library.
echo "Generating static library"
rm -f ./platform/libapp.a
arm-none-eabi-ar rcs "./platform/libapp.a" "./apps/$app.o"

# Build platform.
(cd platform && cargo build --release)
