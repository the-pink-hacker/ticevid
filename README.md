# TICEVid

This is still very WIP.

**TICEVid** is video player for the TI-84 Plus CE with DVD-like features over USB.
Since the calculator has no speaks, it has support for positional captions.

## Building

I'd strongly recommend NixOS or a Linux setup with Nix installed for development.

### Nix Dev Shell

All you need to do to setup the development environment is to run the following.
Do note that it'll be building LLVM with EZ80 support from source.
This means a long initial load time. This will be cached subsequent times.

```sh
nix develop
```

### Video Player

Now that we have the asset builder and CE Toolchain, we can run make to build the calculator program.

```sh
make
```

### Video ISO

The next step is to create the video iso to be uploaded to the USB drive.
The example video definition is located in `./resources/video/video.toml`.
Get the USB's device path with `fdisk -l` or `sudo fdisk -l`.
Assuming the device is at `/dev/sda`, run the following.

```sh
make video
cat bin/video.bin /dev/sda
```

## Documentation

For documentation, see the `./docs` folder.
