# obj-export
**obj-export** is a simple command line tool written in Rust. It converts .gznk ForgeLight chunks and assets' colliders to .obj files.

## Options
Run the tool with the `--help` flag to view a list of the tool's options.

```
Usage: navmesh-obj-export [OPTIONS] --path <DIR> --zone <ZONE> --merge-radius <RADIUS>

Options:
  -p, --path <DIR>             Path to assets directory
  -z, --zone <ZONE>            Name of the zone asset (without the .gzne extension)
  -r, --merge-radius <RADIUS>  Radius in which to merge vertices
  -o, --output <FILE>          Path to outout file. If unspecified, prints to stdout
  -h, --help                   Print help
  -V, --version                Print version
```

For example,
```shell
$ cargo run --release -- -p C:/path/to/packed/assets -z Combat_Umbara_South_01 -r 0.01 -o umbara.obj
```
will generate a .obj file for the `Combat_Umbara_South_01` zone and output to umbara.obj, combining points within ~0.01 of each other.

## Building the Tool
[Install Rust](https://www.rust-lang.org/tools/install). Then run `cargo build` or `cargo build --release` (for an optimized build) from the command line.
