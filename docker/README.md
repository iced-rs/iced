## Build
To use `cross` to compile for Raspberry Pi you first need to build the docker image.
Use these commands to build the images needed.

**NOTE:** Run these commands inside the `docker` folder. This is needed since `docker`
uses surrounding directories as "context" for building the image, which means it'll
copy the entire `target` directory.

### Raspberry Pi 2/3/4 (32 bits)
```
$ docker build -t iced-rs/armv7 -f Dockerfile.armv7-unknown-linux-gnueabihf .
```

### Raspberry Pi 3/4 (64 bits)
```
$ docker build -t iced-rs/aarch64 -f Dockerfile.aarch64-unknown-linux-gnu .
```