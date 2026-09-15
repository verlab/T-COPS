use core::f64;

use crate::{
    common::{constants::EPSILON, instance::Instance, solution::Solution},
    solvers::heuristic::vns::state::SearchState,
};

struct InsertionSpot {
    vehicle_id: usize,
    path_id: usize,
    cost_delta: f64,
}

pub fn evaluate_subgroup_insertion<'a>(
    instance: &Instance,
    solution: &Solution<'a>,
    state: &SearchState,
    subgroup_id: usize,
) -> Option<(Solution<'a>, SearchState)> {
    for &cluster_id in &instance.subgroups[subgroup_id].parent_cluster_ids {
        if let Some(&locked_sg) = state.cluster_locks.get(&cluster_id)
            && locked_sg != subgroup_id
        {
            return None;
        }
    }

    let mut trial_sol = solution.clone();
    let mut trial_state = state.clone();

    for &node_id in &instance.subgroups[subgroup_id].node_ids {
        match find_best_spot_for_node(instance, &trial_sol, node_id) {
            Some(spot) => {
                trial_sol.routes[spot.vehicle_id]
                    .path
                    .insert(spot.path_id, node_id);

                trial_sol.routes[spot.vehicle_id].cost += spot.cost_delta;
                trial_sol.total_cost += spot.cost_delta;
                trial_state.visited_nodes.insert(node_id);
            }
            None => return None,
        }
    }

    trial_sol.total_score += instance.subgroups[subgroup_id].profit;
    for &cluster_id in &instance.subgroups[subgroup_id].parent_cluster_ids {
        trial_state.cluster_locks.insert(cluster_id, subgroup_id);
    }
    trial_state
        .subgroup_nodes_count
        .insert(subgroup_id, instance.subgroups[subgroup_id].node_ids.len());

    Some((trial_sol, trial_state))
}

fn find_best_spot_for_node(
    instance: &Instance,
    solution: &Solution,
    node_id: usize,
) -> Option<InsertionSpot> {
    if instance
        .vehicles
        .iter()
        .any(|v| v.start_node_id == node_id || v.end_node_id == node_id)
    {
        return None;
    }

    let mut best_spot = None;
    let mut best_cost = f64::MAX;

    for (vehicle_id, route) in solution.routes.iter().enumerate() {
        let vehicle = &instance.vehicles[vehicle_id];

        for i in 0..(route.path.len() - 1) {
            let prev = route.path[i];
            let next = route.path[i + 1];

            let added = instance.get_distance(prev, node_id) + instance.get_distance(node_id, next);
            let removed = instance.get_distance(prev, next);
            let delta = added - removed;

            if route.cost + delta <= vehicle.budget && delta < best_cost - EPSILON {
                best_cost = delta;
                best_spot = Some(InsertionSpot {
                    vehicle_id,
                    path_id: i + 1,
                    cost_delta: delta,
                });
            }
        }
    }

    best_spot
}

pub fn drop_subgroup(
    instance: &Instance,
    solution: &mut Solution,
    state: &mut SearchState,
    subgroup_id: usize,
) {
    for &node_id in &instance.subgroups[subgroup_id].node_ids {
        remove_node_from_routes(instance, solution, state, node_id);
    }

    solution.total_score -= instance.subgroups[subgroup_id].profit;
    for &cluster_id in &instance.subgroups[subgroup_id].parent_cluster_ids {
        state.cluster_locks.remove(&cluster_id);
    }
    state.subgroup_nodes_count.remove(&subgroup_id);
}

fn remove_node_from_routes(
    instance: &Instance,
    solution: &mut Solution,
    state: &mut SearchState,
    node_id: usize,
) {
    if instance
        .vehicles
        .iter()
        .any(|v| v.start_node_id == node_id || v.end_node_id == node_id)
    {
        return;
    }

    for route in &mut solution.routes {
        let intermediate_nodes = &route.path[1..route.path.len() - 1];

        if let Some(internal_pos) = intermediate_nodes.iter().position(|&n| n == node_id) {
            let pos = internal_pos + 1;

            let prev = route.path[pos - 1];
            let next = route.path[pos + 1];

            let removed_distance =
                instance.get_distance(prev, node_id) + instance.get_distance(node_id, next);
            let direct_distance = instance.get_distance(prev, next);
            let delta = direct_distance - removed_distance;

            route.path.remove(pos);
            route.cost += delta;
            solution.total_cost += delta;
            state.visited_nodes.remove(&node_id);

            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::time::Duration;
    use crate::common::instance::{Cluster, Metric, Node, Point3, Subgroup, Vehicle};
    use crate::common::solution::{Route, SolutionStatus};

    fn helper_create_test_instance() -> Instance {
        Instance {
            name: "test_inst".to_string(),
            metric: Metric::Euc2d,
            nodes: vec![
                Node { id: 0, point: Point3 { x: 0.0, y: 0.0, z: 0.0 }, ..Default::default() },
                Node { id: 1, point: Point3 { x: 0.0, y: 3.0, z: 0.0 }, ..Default::default() },
                Node { id: 2, point: Point3 { x: 4.0, y: 0.0, z: 0.0 }, ..Default::default() },
                Node { id: 3, point: Point3 { x: 100.0, y: 100.0, z: 0.0 }, ..Default::default() },
            ],
            subgroups: vec![
                Subgroup { id: 0, profit: 10.0, node_ids: vec![1], parent_cluster_ids: HashSet::from([0]) },
                Subgroup { id: 1, profit: 20.0, node_ids: vec![2], parent_cluster_ids: HashSet::from([0]) },
                Subgroup { id: 2, profit: 30.0, node_ids: vec![3], parent_cluster_ids: HashSet::from([1]) },
                Subgroup { id: 3, profit: 40.0, node_ids: vec![0], parent_cluster_ids: HashSet::from([2]) }, // Depot node inside subgroup
            ],
            clusters: vec![
                Cluster { id: 0, subgroup_ids: vec![0, 1] },
                Cluster { id: 1, subgroup_ids: vec![2] },
                Cluster { id: 2, subgroup_ids: vec![3] },
            ],
            vehicles: vec![
                Vehicle { id: 0, budget: 20.0, start_node_id: 0, end_node_id: 0 },
            ],
            ..Default::default()
        }
    }

    fn helper_create_initial_solution<'a>(instance: &'a Instance) -> (Solution<'a>, SearchState) {
        let route = Route {
            vehicle_id: 0,
            path: vec![0, 0],
            cost: 0.0,
        };
        let mut state = SearchState::default();
        state.visited_nodes.insert(0);

        let solution = Solution {
            instance,
            duration: Duration::from_secs(0),
            total_score: 0.0,
            total_cost: 0.0,
            routes: vec![route],
            status: SolutionStatus::Feasible,
            solver: None,
            best_bound: None,
            gap: None,
            explored_nodes: None,
        };

        (solution, state)
    }

    #[test]
    fn test_evaluate_subgroup_insertion_success() {
        let instance = helper_create_test_instance();
        let (solution, state) = helper_create_initial_solution(&instance);

        let res = evaluate_subgroup_insertion(&instance, &solution, &state, 0);
        assert!(res.is_some());
        let (trial_sol, trial_state) = res.unwrap();

        assert_eq!(trial_sol.total_score, 10.0);
        assert_eq!(trial_sol.routes[0].path, vec![0, 1, 0]);
        assert_eq!(trial_sol.total_cost, 6.0); // (0,0)->(0,3)->(0,0) = 3 + 3 = 6
        assert!(trial_state.visited_nodes.contains(&1));
        assert_eq!(trial_state.cluster_locks.get(&0), Some(&0));
        assert_eq!(trial_state.subgroup_nodes_count.get(&0), Some(&1));
    }

    #[test]
    fn test_evaluate_subgroup_insertion_cluster_locked() {
        let instance = helper_create_test_instance();
        let (solution, mut state) = helper_create_initial_solution(&instance);

        // Lock Cluster 0 to Subgroup 0
        state.cluster_locks.insert(0, 0);

        // Try inserting Subgroup 1 which also belongs to Cluster 0 -> should fail
        let res = evaluate_subgroup_insertion(&instance, &solution, &state, 1);
        assert!(res.is_none());
    }

    #[test]
    fn test_evaluate_subgroup_insertion_budget_exceeded() {
        let instance = helper_create_test_instance();
        let (solution, state) = helper_create_initial_solution(&instance);

        // Subgroup 2 contains node 3 at (100,100), distance ~282 > budget 20 -> fail
        let res = evaluate_subgroup_insertion(&instance, &solution, &state, 2);
        assert!(res.is_none());
    }

    #[test]
    fn test_evaluate_subgroup_insertion_depot_node_fail() {
        let instance = helper_create_test_instance();
        let (solution, state) = helper_create_initial_solution(&instance);

        // Subgroup 3 contains node 0 (start/end depot node) -> fail
        let res = evaluate_subgroup_insertion(&instance, &solution, &state, 3);
        assert!(res.is_none());
    }

    #[test]
    fn test_drop_subgroup() {
        let instance = helper_create_test_instance();
        let (solution, state) = helper_create_initial_solution(&instance);

        let (mut trial_sol, mut trial_state) = evaluate_subgroup_insertion(&instance, &solution, &state, 0).unwrap();
        assert_eq!(trial_sol.total_score, 10.0);
        assert_eq!(trial_sol.routes[0].path, vec![0, 1, 0]);

        drop_subgroup(&instance, &mut trial_sol, &mut trial_state, 0);

        assert_eq!(trial_sol.total_score, 0.0);
        assert_eq!(trial_sol.total_cost, 0.0);
        assert_eq!(trial_sol.routes[0].path, vec![0, 0]);
        assert!(!trial_state.visited_nodes.contains(&1));
        assert!(!trial_state.cluster_locks.contains_key(&0));
        assert!(!trial_state.subgroup_nodes_count.contains_key(&0));
    }

    #[test]
    fn test_remove_depot_node_from_routes_does_nothing() {
        let instance = helper_create_test_instance();
        let (mut solution, mut state) = helper_create_initial_solution(&instance);

        let initial_path = solution.routes[0].path.clone();
        remove_node_from_routes(&instance, &mut solution, &mut state, 0);

        assert_eq!(solution.routes[0].path, initial_path);
    }

    #[test]
    fn test_evaluate_subgroup_insertion_multi_parent_cluster_locking() {
        let mut instance = helper_create_test_instance();
        // Make subgroup 0 belong to both cluster 0 and cluster 1
        instance.subgroups[0].parent_cluster_ids = HashSet::from([0, 1]);

        let (solution, state) = helper_create_initial_solution(&instance);

        // Insert subgroup 0 -> should lock both cluster 0 and cluster 1
        let (trial_sol, trial_state) = evaluate_subgroup_insertion(&instance, &solution, &state, 0).unwrap();
        assert_eq!(trial_state.cluster_locks.get(&0), Some(&0));
        assert_eq!(trial_state.cluster_locks.get(&1), Some(&0));

        // Subgroup 2 belongs to cluster 1 -> trying to insert it now must fail
        let res2 = evaluate_subgroup_insertion(&instance, &trial_sol, &trial_state, 2);
        assert!(res2.is_none());
    }
}
