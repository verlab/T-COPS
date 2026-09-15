import sys
import os
import json
import matplotlib.pyplot as plt
import matplotlib.cm as cm
import matplotlib.colors as mcolors
from matplotlib.patches import Polygon as mplPolygon
from mpl_toolkits.mplot3d import Axes3D
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
import numpy as np
from scipy.spatial import ConvexHull, QhullError
from shapely.geometry import MultiPoint

class Visualizer:
    def __init__(self, data, should_show=False, should_save=False, save_path_base="output",
                 thick_cluster=2, thick_subgroup=5, thick_route=3,
                 route_edge_color='black', route_edge_width=1.5,
                 depot_marker='*', depot_color='gold', depot_size=150,
                 grid_style=':'):
        
        self.data = data
        self.should_show = should_show
        self.should_save = should_save
        self.save_path_base = save_path_base
        self.thick_cluster = thick_cluster
        self.thick_subgroup = thick_subgroup
        self.thick_route = thick_route
        self.route_edge_color = route_edge_color
        self.route_edge_width = route_edge_width
        self.depot_marker = depot_marker
        self.depot_color = depot_color
        self.depot_size = depot_size
        self.grid_style = grid_style
        
        self.nodes = {n['id']: np.array([n['x'], n['y'], n.get('z', 0)]) for n in data['nodes']}
        self.subgroups = {s['id']: s for s in data['subgroups']}
        
        profits = [s['profit'] for s in self.subgroups.values()]
        
        if not profits: 
            v_min, v_max = 0, 1
        else:
            v_min, v_max = min(profits), max(profits)
            if v_min == v_max: v_max += 0.1

        self.norm = mcolors.Normalize(vmin=v_min, vmax=v_max + 0.1 * (v_max - v_min))
        self.cmap_heat = plt.get_cmap("RdYlBu_r") 
        self.route_colors = ['red', 'green', 'blue', 'orange', 'purple', 'cyan', 'brown', 'pink', 'olive', 'gray']

    def _get_subgroup_color(self, profit):
        return self.cmap_heat(self.norm(profit))

    def _get_route_color(self, index):
        return self.route_colors[index % len(self.route_colors)]

    def _draw_subgroup_3d(self, ax, pts, color):
        if len(pts) == 1:
            ax.scatter(pts[0, 0], pts[0, 1], pts[0, 2], c=[color], s=40, depthshade=False)
            return
        if len(pts) == 2:
            ax.plot(pts[:, 0], pts[:, 1], pts[:, 2], color=color, linewidth=self.thick_route, alpha=0.9)
            return
        if len(pts) == 3:
            tri = Poly3DCollection([pts], alpha=0.6)
            tri.set_facecolor(color)
            tri.set_edgecolor(color)
            ax.add_collection3d(tri)
            return
        try:
            hull = ConvexHull(pts)
            triangles = [pts[s] for s in hull.simplices]
            mesh = Poly3DCollection(triangles, alpha=0.5)
            mesh.set_facecolor(color)
            mesh.set_edgecolor(color) 
            ax.add_collection3d(mesh)
        except QhullError:
            ax.plot(pts[:, 0], pts[:, 1], pts[:, 2], 'o--', color=color, alpha=0.5)

    def _finalize_plot(self, plt):
        if self.should_save:
            filename = f"{self.save_path_base}.png"
            plt.savefig(filename, dpi=300, bbox_inches='tight')
            print(f"File saved at: {filename}")
        if self.should_show:
            plt.show()
        else:
            plt.close() 

    def plot_2d(self):
        fig, ax = plt.subplots(figsize=(10, 10))
        points_array = np.array([n[:2] for n in self.nodes.values()])
        min_x = np.min(points_array[:, 0])
        max_x = np.max(points_array[:, 0])
        dilate_size = (max_x - min_x) / 45

        # Clusters (White Background)
        for c in self.data.get('clusters', []):
            pts = self._get_cluster_points(c)
            if len(pts) > 0:
                combined_points = MultiPoint([(p[0], p[1]) for p in pts])
                concave_hull = combined_points.convex_hull
                dilated = concave_hull.buffer(dilate_size * 1.8)
                
                if dilated.geom_type == 'MultiPolygon':
                    for poly in dilated.geoms:
                        ax.add_patch(mplPolygon(poly.exterior.coords, fc='white', ec='black', linewidth=self.thick_cluster, alpha=0.3, zorder=0))
                elif dilated.geom_type == 'Polygon':
                    ax.add_patch(mplPolygon(dilated.exterior.coords, fc='white', ec='black', linewidth=self.thick_cluster, alpha=0.3, zorder=0))

        # Subgroups (Profit Color)
        for s in self.subgroups.values():
            pts = np.array([self.nodes[nid] for nid in s['node_ids']])
            if len(pts) == 0: continue
            color = self._get_subgroup_color(s['profit'])
            cluster_points = MultiPoint([(p[0], p[1]) for p in pts])
            dilated = cluster_points.convex_hull.buffer(dilate_size)
            
            if dilated.geom_type == 'MultiPolygon':
                for poly in dilated.geoms:
                    ax.add_patch(mplPolygon(poly.exterior.coords, fc=color, ec=color, linewidth=self.thick_subgroup, alpha=0.7, zorder=1))
            elif dilated.geom_type == 'Polygon':
                ax.add_patch(mplPolygon(dilated.exterior.coords, fc=color, ec=color, linewidth=self.thick_subgroup, alpha=0.7, zorder=1))
                
            ax.scatter(pts[:, 0], pts[:, 1], color='black', marker='.', s=150, alpha=0.8, zorder=2)

        # Routes
        depot_ids = set()
        for route_obj in self.data.get('routes', []):
            path = route_obj.get('path', [])
            if not path: continue
            
            depot_ids.add(path[0])
            depot_ids.add(path[-1])
            
            valid_pts = [self.nodes[nid] for nid in path if nid in self.nodes]
            if not valid_pts: continue
            pts = np.array(valid_pts)
            c = self._get_route_color(route_obj['vehicle_id'])
            
            if self.route_edge_width > 0:
                ax.plot(pts[:,0], pts[:,1], '-', color=self.route_edge_color, 
                        linewidth=self.thick_route + self.route_edge_width, zorder=3)
            
            ax.plot(pts[:,0], pts[:,1], '-', color=c, 
                    linewidth=self.thick_route, label=f"Vehicle {route_obj['vehicle_id']}", zorder=4)

        if not depot_ids and 0 in self.nodes:
            depot_ids.add(0)
            
        label_added = False
        for d_id in depot_ids:
            if d_id in self.nodes:
                depot = self.nodes[d_id]
                ax.scatter(depot[0], depot[1], color=self.depot_color, edgecolors='black', 
                           linewidths=1.5, marker=self.depot_marker, s=self.depot_size, zorder=5, 
                           label='Depot' if not label_added else "")
                label_added = True

        ax.set_aspect('auto')
        
        if self.grid_style:
            ax.grid(True, linestyle=self.grid_style, alpha=0.6)
        else:
            ax.grid(False)
        
        if len(self.data.get('routes', [])) > 0:
            ax.legend(loc='upper left', bbox_to_anchor=(1.02, 1), borderaxespad=0, frameon=True, fontsize=9)
        plt.subplots_adjust(right=0.8)
        
        sm = cm.ScalarMappable(cmap=self.cmap_heat, norm=self.norm)
        sm.set_array([])
        cbar = plt.colorbar(sm, ax=ax, fraction=0.04, pad=0.08, orientation='horizontal')
        cbar.set_label('Reward', fontsize=12)
        
        self._finalize_plot(plt)

    def plot_3d(self):
        fig = plt.figure(figsize=(12, 10))
        ax = fig.add_subplot(111, projection='3d')
        
        for c in self.data.get('clusters', []):
            points = self._get_cluster_points(c)
            if len(points) >= 4:
                try:
                    hull = ConvexHull(points)
                    for simplex in hull.simplices:
                        cycle = np.append(simplex, simplex[0])
                        ax.plot(points[cycle, 0], points[cycle, 1], points[cycle, 2], color='#CCCCCC', linestyle='--', linewidth=0.8, alpha=0.5)
                except QhullError: pass

        for s in self.subgroups.values():
            pts = np.array([self.nodes[nid] for nid in s['node_ids']])
            if len(pts) == 0: continue
            color = self._get_subgroup_color(s['profit'])
            self._draw_subgroup_3d(ax, pts, color)

        for nid, n in self.nodes.items():
            ax.scatter(n[0], n[1], n[2], c='black', marker='o', s=20, depthshade=False)

        depot_ids = set()
        for route_obj in self.data.get('routes', []):
            path = route_obj.get('path', [])
            if not path: continue
            
            depot_ids.add(path[0])
            depot_ids.add(path[-1])
            
            valid_pts = [self.nodes[nid] for nid in path if nid in self.nodes]
            if not valid_pts: continue
            pts = np.array(valid_pts)
            c = self._get_route_color(route_obj['vehicle_id'])
            
            if self.route_edge_width > 0:
                ax.plot(pts[:,0], pts[:,1], pts[:,2], '-', color=self.route_edge_color, 
                        linewidth=self.thick_route + self.route_edge_width)
            
            ax.plot(pts[:,0], pts[:,1], pts[:,2], '-', color=c, 
                    linewidth=self.thick_route, label=f"Vehicle {route_obj['vehicle_id']}")

        if not depot_ids and 0 in self.nodes:
            depot_ids.add(0)
            
        label_added = False
        for d_id in depot_ids:
            if d_id in self.nodes:
                depot = self.nodes[d_id]
                ax.scatter(depot[0], depot[1], depot[2], color=self.depot_color, edgecolors='black', 
                           linewidths=1.5, marker=self.depot_marker, s=self.depot_size, depthshade=False, zorder=5, 
                           label='Depot' if not label_added else "")
                label_added = True

        if len(self.data.get('routes', [])) > 0:
            ax.legend(loc='upper left', bbox_to_anchor=(1.1, 1), borderaxespad=0, frameon=True, fontsize=8)

        ax.set_box_aspect([1, 1, 1]) 
        x_limits = ax.get_xlim3d()
        y_limits = ax.get_ylim3d()
        z_limits = ax.get_zlim3d()
        x_range = abs(x_limits[1] - x_limits[0])
        y_range = abs(y_limits[1] - y_limits[0])
        z_range = abs(z_limits[1] - z_limits[0])
        max_range = max(x_range, y_range, z_range)
        x_mid = np.mean(x_limits)
        y_mid = np.mean(y_limits)
        z_mid = np.mean(z_limits)
        ax.set_xlim3d([x_mid - max_range / 2, x_mid + max_range / 2])
        ax.set_ylim3d([y_mid - max_range / 2, y_mid + max_range / 2])
        ax.set_zlim3d([z_mid - max_range / 2, z_mid + max_range / 2])
        
        if self.grid_style:
            ax.grid(True, linestyle=self.grid_style, alpha=0.6)
        else:
            ax.grid(False)

        plt.subplots_adjust(right=0.8)
        
        sm = cm.ScalarMappable(cmap=self.cmap_heat, norm=self.norm)
        sm.set_array([])
        cbar = plt.colorbar(sm, ax=ax, fraction=0.04, pad=0.08, orientation='horizontal')
        cbar.set_label('Reward', fontsize=12)

        self._finalize_plot(plt)

    def _get_cluster_points(self, cluster):
        pts = []
        for sid in cluster['subgroup_ids']:
            if sid in self.subgroups:
                for nid in self.subgroups[sid]['node_ids']:
                    if nid in self.nodes: pts.append(self.nodes[nid])
        return np.array(pts)

if __name__ == "__main__":
    if len(sys.argv) < 2: 
        print("Usage: python visualizer.py <file.json> [show] [save]")
        sys.exit(1)
        
    file_path = sys.argv[1]
    args = sys.argv[2:]
    
    should_show = any(arg.lower() in ["show", "--show"] for arg in args)
    should_save = any(arg.lower() in ["save", "--save"] for arg in args)

    with open(file_path, 'r') as f: 
        data = json.load(f)
        
    plot = Visualizer(
        data, 
        should_show=should_show, 
        should_save=should_save, 
        save_path_base=os.path.splitext(file_path)[0],
        thick_cluster=2,      
        thick_subgroup=5,     
        thick_route=3,
        route_edge_color='black',
        route_edge_width=1.0,
        depot_marker='X',     
        depot_color='gold',   
        depot_size=100,       
        grid_style='None'        
    )
    
    plot.plot_2d() if data.get("mode", "2d") == "2d" else plot.plot_3d()