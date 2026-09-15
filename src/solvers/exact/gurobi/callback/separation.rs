use super::max_flow::{Edge, max_flow_min_cut};
use std::collections::VecDeque;

pub fn find_fractional_subtours(
    num_nodes: usize,
    start_node: usize,
    edges: &[(usize, usize, f64)],
    y_vals: &[f64],
) -> Vec<(Vec<usize>, f64, usize)> {
    let mut old_to_new = vec![usize::MAX; num_nodes];
    let mut new_to_old = Vec::new();

    old_to_new[start_node] = 0;
    new_to_old.push(start_node);

    for &(u, v, _) in edges {
        if old_to_new[u] == usize::MAX {
            old_to_new[u] = new_to_old.len();
            new_to_old.push(u);
        }
        if old_to_new[v] == usize::MAX {
            old_to_new[v] = new_to_old.len();
            new_to_old.push(v);
        }
    }

    let num_active = new_to_old.len();
    let mut adj: Vec<Vec<Edge>> = vec![vec![]; num_active];

    for &(u, v, cap) in edges {
        let nu = old_to_new[u];
        let nv = old_to_new[v];
        let rev_u = adj[nv].len();
        let rev_v = adj[nu].len();

        adj[nu].push(Edge {
            to: nv,
            cap,
            rev_idx: rev_u,
        });
        
        adj[nv].push(Edge {
            to: nu,
            cap: 0.0,
            rev_idx: rev_v,
        });
    }

    let candidates: Vec<(usize, f64)> = y_vals
        .iter()
        .enumerate()
        .filter(|&(i, &y_val)| i != start_node && y_val > 1e-4 && old_to_new[i] != usize::MAX)
        .map(|(i, &y_val)| (old_to_new[i], y_val))
        .collect();

    let mut all_bad_tours = Vec::new();
    let src = 0;

    for (sink, y_val) in candidates {
        let mut visited = vec![false; num_active];
        let mut queue = VecDeque::new();
        queue.push_back(src);
        visited[src] = true;
        let mut reaches_sink = false;

        while let Some(u) = queue.pop_front() {
            if u == sink {
                reaches_sink = true;
            }

            for edge in &adj[u] {
                if edge.cap > 1e-6 && !visited[edge.to] {
                    visited[edge.to] = true;
                    queue.push_back(edge.to);
                }
            }
        }

        if !reaches_sink {
            let violation = y_val;
            let mut component: Vec<usize> = (0..num_active)
                .filter(|&i| !visited[i])
                .map(|i| new_to_old[i])
                .collect();

            component.sort_unstable();
            let sink_orig = new_to_old[sink];
            all_bad_tours.push((component, violation, sink_orig));

            continue;
        }

        let mut adj_run = adj.clone();
        let (max_flow, min_cut) = max_flow_min_cut(num_active, &mut adj_run, src, sink);

        let violation = y_val - max_flow;
        if violation > 1e-4 {
            let mut component: Vec<usize> = min_cut.into_iter().map(|i| new_to_old[i]).collect();
            component.sort_unstable();
            let sink_orig = new_to_old[sink];
            all_bad_tours.push((component, violation, sink_orig));
        }
    }

    all_bad_tours
}

pub fn find_invalid_subtours(
    num_nodes: usize,
    start_node: usize,
    end_node: usize,
    active_edges: &[(usize, usize)],
) -> Vec<Vec<usize>> {
    let mut graph: Vec<Vec<usize>> = vec![vec![]; num_nodes];
    for &(i, j) in active_edges {
        graph[i].push(j);
        graph[j].push(i);
    }

    let mut visited = vec![false; num_nodes];
    let mut bad_tours = Vec::new();

    for node in 0..num_nodes {
        if !visited[node] && !graph[node].is_empty() {
            let mut component = Vec::new();
            let mut stack = vec![node];

            while let Some(current) = stack.pop() {
                if !visited[current] {
                    visited[current] = true;
                    component.push(current);

                    for &neighbor in &graph[current] {
                        stack.push(neighbor);
                    }
                }
            }

            if !component.contains(&start_node) && !component.contains(&end_node) {
                bad_tours.push(component);
            }
        }
    }
    bad_tours
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_invalid_subtours_connected_to_depot() {
        // Active edges forming route 0 -> 1 -> 2 -> 0 (depot is 0)
        let active_edges = vec![(0, 1), (1, 2), (2, 0)];
        let bad_tours = find_invalid_subtours(3, 0, 0, &active_edges);
        // Valid main tour connected to depot 0 -> no invalid subtours
        assert!(bad_tours.is_empty());
    }

    #[test]
    fn test_find_invalid_subtours_disconnected_subtour() {
        // Route 0 -> 1 -> 0 (depot 0), PLUS disconnected subtour 2 -> 3 -> 2
        let active_edges = vec![(0, 1), (1, 0), (2, 3), (3, 2)];
        let bad_tours = find_invalid_subtours(4, 0, 0, &active_edges);
        assert_eq!(bad_tours.len(), 1);
        let mut tour = bad_tours[0].clone();
        tour.sort_unstable();
        assert_eq!(tour, vec![2, 3]);
    }

    #[test]
    fn test_find_fractional_subtours_unreachable_sink() {
        // Node 0 (depot), Node 1 (y=1.0) with zero capacity edge (0, 1)
        let edges = vec![(0, 1, 0.0)];
        let y_vals = vec![0.0, 1.0];
        let bad_tours = find_fractional_subtours(2, 0, &edges, &y_vals);
        // Sink 1 is unreachable from 0 -> returns fractional cut for sink node 1
        assert_eq!(bad_tours.len(), 1);
        assert_eq!(bad_tours[0].2, 1);
    }

    #[test]
    fn test_find_invalid_subtours_different_origin_destination() {
        // Active edges forming route 0 -> 1 -> 2 -> 3 (start is 0, end is 3)
        let active_edges = vec![(0, 1), (1, 2), (2, 3)];
        let bad_tours = find_invalid_subtours(4, 0, 3, &active_edges);
        // Valid main route connecting start 0 to end 3 -> no invalid subtours
        assert!(bad_tours.is_empty());

        // Route 0 -> 1 -> 3 (start 0, end 3), plus disconnected subtour 4 -> 5 -> 4
        let active_edges = vec![(0, 1), (1, 3), (4, 5), (5, 4)];
        let bad_tours = find_invalid_subtours(6, 0, 3, &active_edges);
        assert_eq!(bad_tours.len(), 1);
        let mut tour = bad_tours[0].clone();
        tour.sort_unstable();
        assert_eq!(tour, vec![4, 5]);
    }
}

