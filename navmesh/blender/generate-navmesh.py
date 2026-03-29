import argparse
import bpy
import json
import sys

GROUP_PREFIX = "NAVMESH"

def print_debug(text, verbose=True):
    if verbose:
        print(text)


def main(navmesh_name, in_file, out_file, verbose):
    bpy.ops.wm.open_mainfile(filepath=in_file)
    layers = {}

    for obj in bpy.context.scene.objects:
        if obj.type == "MESH":
            for vertex_group in obj.vertex_groups:
                if vertex_group.name.startswith(GROUP_PREFIX):
                    [_, layer_index] = vertex_group.name.split(GROUP_PREFIX, 1)
                    try:
                        layer_index = int(layer_index)
                    except ValueError as err:
                        print(f"Could not determine layer index for group {vertex_group.name}")
                        continue

                    group_vertices = set([v.index for v in obj.data.vertices if vertex_group.index in [g.group for g in v.groups]])
    
                    outer_edge_vertices = set()
                    for edge in obj.data.edges:
                        if edge.vertices[0] in group_vertices and edge.vertices[1] in group_vertices:
                            if len(edge.link_faces) == 1:
                                outer_edge_vertices.add(edge.vertices[0])
                                outer_edge_vertices.add(edge.vertices[1])

                    layers.setdefault(layer_index, set())
                    layers[layer_index] |= outer_edge_vertices
                else:
                    print_debug(f"Skipping {vertex_group.name} because it does not start with {GROUP_PREFIX}", verbose)
        else:
            print_debug(f"Skipping {obj.name} because is not a mesh", verbose)

        output = {
            navmesh_name: [layers[key] for key in sorted(layers.keys())]
        }
        with open(out_file, "w") as file:
            json.dump(output, file)


if __name__ == "__main__":
    if "--" in sys.argv:
        script_args = sys.argv[sys.argv.index("--") + 1:]
    else:
        script_args = []

    parser = argparse.ArgumentParser(description="Generates a layered navmesh from selected polygons in NAVMESH# vertex groups")
    parser.add_argument("--name", type=str, required=True, help="Name of the navmesh")
    parser.add_argument("--infile", type=str, required=True, help="Path of the input .blend file")
    parser.add_argument("--outfile", type=str, required=True, help="Path of the output .yaml file")
    parser.add_argument("--verbose", action="store_true", help="Whether to print verbose output")
    
    args, _ = parser.parse_known_args(script_args)
    main(args.name, args.infile, args.outfile, args.verbose)
