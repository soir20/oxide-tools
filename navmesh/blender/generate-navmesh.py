import argparse
import bmesh
import bpy
from collections import deque
import json
import sys

GROUP_PREFIX = "NAVMESH"

def print_debug(text, verbose=True):
    if verbose:
        print(text)


def coords(vertex):
    return (vertex.co.x, vertex.co.y, vertex.co.z)


def polygon_area_shoelace(vertices):
    n = len(vertices)
    area = 0.0
    for i in range(n):
        j = (i + 1) % n
        x1, y1 = vertices[i]
        x2, y2 = vertices[j]
        area += x1 * y2 - x2 * y1
    
    return abs(area) / 2.0


def polygonize(graph, vertex, path, visited):
    polygons = []
    while len(graph[vertex]) > 0:
        neighbor = graph[vertex].pop()
        graph[neighbor].discard(vertex)
        if neighbor in visited:
            # We found a loop, so there is a polygon, but we might be intersecting with the
            # vertex we started at. Remove extraneous parts of the path so we start at the vertex
            # we're intersecting with.
            trimmed_path = deque(path)
            while trimmed_path[0] != neighbor:
                trimmed_path.popleft()
            polygons.append(path)
        else:
            polygons.extend(polygonize(graph, neighbor, path + [neighbor], visited | {neighbor}))
    
    return polygons


def main(in_file, out_file, verbose):
    bpy.ops.wm.open_mainfile(filepath=in_file)
    layers = {}

    if bpy.context.mode != "EDIT":
        bpy.ops.object.mode_set(mode="EDIT")

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

                    bpy.ops.mesh.select_all(action="DESELECT")
                    obj.vertex_groups.active = obj.vertex_groups[vertex_group.name]
                    bpy.ops.object.vertex_group_select()
                    bpy.ops.mesh.region_to_loop()

                    graph = {}

                    for edge in [edge for edge in bmesh.from_edit_mesh(obj.data).edges if edge.select]:
                        graph.setdefault(coords(edge.verts[0]), set()).add(coords(edge.verts[1]))
                        graph.setdefault(coords(edge.verts[1]), set()).add(coords(edge.verts[0]))

                    for vertex in graph.keys():
                        layers.setdefault(layer_index, []).extend(polygonize(graph, vertex, [vertex], {vertex}))
                else:
                    print_debug(f"Skipping {vertex_group.name} because it does not start with {GROUP_PREFIX}", verbose)
        else:
            print_debug(f"Skipping {obj.name} because is not a mesh", verbose)

        output = [list(layers[key]) for key in sorted(layers.keys())]
        with open(out_file, "w") as file:
            json.dump(output, file, indent=2)


if __name__ == "__main__":
    if "--" in sys.argv:
        script_args = sys.argv[sys.argv.index("--") + 1:]
    else:
        script_args = []

    parser = argparse.ArgumentParser(description="Generates a layered navmesh from selected polygons in NAVMESH# vertex groups")
    parser.add_argument("--infile", type=str, required=True, help="Path of the input .blend file")
    parser.add_argument("--outfile", type=str, required=True, help="Path of the output .yaml file")
    parser.add_argument("--verbose", action="store_true", help="Whether to print verbose output")
    
    args, _ = parser.parse_known_args(script_args)
    main(args.infile, args.outfile, args.verbose)
