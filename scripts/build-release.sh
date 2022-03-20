#!/bin/zsh

QUANT_DIR="./quant-live"
DIST_DIR="./dist"
DIST_QUANT_DIR="$DIST_DIR/quant-live"
PROD_DIR="./prod-scripts/."
RELEASE_FILE="./target/release/quant"
ARCHIVE_FILE="./quant.tar.gz"
LIVE_DIR_ROOT_PATH="../../"
LIVE_DIR_PATH="../quant-live"

echo "deleting the existing dist directory..."
rm -rf $DIST_DIR

echo "deleting the existing release file..."
rm -rf $RELEASE_FILE

echo "creating the release build..."
cargo build --release
mkdir -p $DIST_QUANT_DIR

echo "copying production scripts into the dist directory..."
cp -R $PROD_DIR $DIST_QUANT_DIR

echo "copying release build into the dist directory..."
cp $RELEASE_FILE $DIST_QUANT_DIR

echo "copying sample config files into the dist directory..."
cp ./sample.config.yaml ./sample.secrets.yaml $DIST_QUANT_DIR

echo "archiving the file..."
(cd $DIST_DIR && tar cvpfz $ARCHIVE_FILE $QUANT_DIR)

if [[ $OSTYPE == 'darwin'* ]]; then
  echo "opening the dist directory"
  open $DIST_DIR
fi

echo "copying sample config files into the dist directory..."
(cd $DIST_DIR && tar -xf $ARCHIVE_FILE -C $LIVE_DIR_ROOT_PATH)

echo "dist directory: $DIST_DIR"
echo "live directory: $LIVE_DIR_PATH"
