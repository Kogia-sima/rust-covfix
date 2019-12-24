$TARGET="$($Env:PLATFORM_TARGET)-pc-windows-msvc"
$PACKAGE_NAME="rust-covfix-win-$($Env:PLATFORM_TARGET)"

cargo build --target "$TARGET" --release --verbose

mkdir "$PACKAGE_NAME"
copy ".\target\$TARGET\release\rust-covfix.exe" "$PACKAGE_NAME\"
copy ".\README.md" "$PACKAGE_NAME\"
copy ".\LICENSE" "$PACKAGE_NAME\"

7z a "$PACKAGE_NAME.zip" "$PACKAGE_NAME"

Push-AppveyorArtifact "$PACKAGE_NAME.zip"
