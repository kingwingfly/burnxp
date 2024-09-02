WIP

# Prepare

## Pre-requisites:

No matter you decide to use the compiled release or compile the project yourself,
you need to do following preparation.

### 1. Non-macOS
**Cuda 12.x should be installed**

This tool depends on `libtorch` for cuda accelerating.

```sh
wget -O libtorch.zip https://download.pytorch.org/libtorch/cu121/libtorch-cxx11-abi-shared-with-deps-2.2.0%2Bcu121.zip
unzip libtorch.zip
rm libtorch.zip
```
You are supposed to put unzipped libtorch directory at the root of the project,
so that the environment variables can be correctly set by the sh scripts along with the repo.

### 2. MacOS
```sh
python3 -m venv pytorch
source pytorch/bin/activate
pip install torch==2.2.0 numpy==1.26.4 setuptools
```

## Use compiled release

You can download the tagger and model in the [release page](https://github.com/kingwingfly/burn_nn/releases).

## Compile yourself

### Train:
```sh
git clone git@github.com:kingwingfly/burn_nn.git
# Non-macOS
scripts/train.sh
# MacOS
scripts/train_macOS.sh
```
# License

MIT LICENSE
