WIP

# Prepare

## Pre-requisites: 

**Cuda 12.x should be installed**
This tool depends on `libtorch` for cuda accelerating.

```sh
wget -O libtorch.zip https://download.pytorch.org/libtorch/cu121/libtorch-cxx11-abi-shared-with-deps-2.2.0%2Bcu121.zip
unzip libtorch.zip
rm libtorch.zip
```

You are supposed to put unzipped libtorch directory at the root of the project, so that the environment variables can be correctly set by the sh scripts along with the repo. 

## Use compiled release

You can download the tagger and model in the [release page](https://github.com/kingwingfly/burn_nn/releases). 


## Compile yourself

### Train:
```sh
# clone the repo
scripts/train.sh
```
# License

MIT LICENSE