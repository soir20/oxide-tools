# generate-navmesh
**generate-navmesh** is a command-line tool to generate navmeshes for Oxide based on the output of the obj-export tool.

## Usage
You must **manually** add walkable vertices to vertex groups `NAVMESH0`, `NAVMESH1`, etc. Each group corresponds to a layer
in the navmesh. A vertex may belong to multiple groups and will be stitched together.

Each layer must be a single polygon with holes in the middle, i.e. separate rooms with their own mesh must be separate
layers. For each layer, the polygon with the largest area will be the walkable area, while smaller polygons in the middle
will be obstacles.

## Options
Run the tool with the `--help` flag to view a list of the tool's options.

```
Generates a layered navmesh from selected polygons in NAVMESH# vertex groups

options:
  -h, --help         show this help message and exit
  --infile INFILE    Path of the input .blend file
  --outfile OUTFILE  Path of the output .json file
  --verbose          Whether to print verbose output
```

For example,
```shell
$ blender -b --python generate-navmesh.py -- --infile JediTemple.blend --outfile 'jeditemple.json'
```
will generate a .json file containing navmesh data for the vertex groups in JediTemple.blend.
