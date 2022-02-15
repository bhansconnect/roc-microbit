#!/bin/sh

# Build app to object file.
app="$1"
app_roc="./apps/$app.roc"
if [ -f "$app_roc" ]; then
	./roc/target/release/roc build --optimize --no-link --precompiled-host --backend thumbv7emhf $app_roc
else
	echo "$app is not an app!"
fi

# Convert it to a static library.
echo "Generating static library"
rm ./platform/libapp.a
arm-none-eabi-ar rcs "./platform/libapp.a" "./apps/$app.o"

# Build platform.
(cd platform && cargo embed --release)
