use std::time::Duration;

use crate::common::{
    constants::EPSILON,
    error::SolverError,
    instance::Instance,
    solution::{Route, Solution, SolutionStatus},
};
use crate::solvers::heuristic::vns::{
    neighborhoods::evaluate_subgroup_insertion, state::SearchState,
};

pub fn build_greedy_solution(
    instance: &'_ Instance,
) -> Result<(Solution<'_>, SearchState), SolverError> {
    let (mut solution, mut state) = initialize_empty_solution(instance);

    greedily_insert_subgroups(instance, &mut solution, &mut state);

    Ok((solution, state))
}

fn initialize_empty_solution(instance: &'_ Instance) -> (Solution<'_>, SearchState) {
    let mut state = SearchState::default();
    let mut routes = Vec::with_capacity(instance.vehicles.len());
    let mut initial_total_cost = 0.0;

    for vehicle in &instance.vehicles {
        let start = vehicle.start_node_id;
        let end = vehicle.end_node_id;

        state.visited_nodes.insert(start);
        state.visited_nodes.insert(end);

        for &depot in &[start, end] {
            for &sg_id in &instance.nodes[depot].parent_subgroup_ids {
                for &c_id in &instance.subgroups[sg_id].parent_cluster_ids {
                    state.cluster_locks.insert(c_id, sg_id);
                }
                state.subgroup_nodes_count.insert(sg_id, 1);
            }
        }

        let base_cost = instance.get_distance(start, end);
        initial_total_cost += base_cost;

        routes.push(Route {
            path: vec![start, end],
            cost: base_cost,
            vehicle_id: vehicle.id,
        });
    }

    let solution = Solution {
        instance,
        duration: Duration::from_secs(0),
        total_score: 0.0,
        total_cost: initial_total_cost,
        routes,
        status: SolutionStatus::Feasible,
        solver: None,
        best_bound: None,
        gap: None,
        explored_nodes: None,
    };

    (solution, state)
}

fn greedily_insert_subgroups(
    instance: &Instance,
    solution: &mut Solution,
    state: &mut SearchState,
) {
    while let Some((new_sol, new_state)) = find_best_subgroup_insertion(instance, solution, state) {
        *solution = new_sol;
        *state = new_state;
    }
}

fn find_best_subgroup_insertion<'a>(
    instance: &Instance,
    solution: &Solution<'a>,
    state: &SearchState,
) -> Option<(Solution<'a>, SearchState)> {
    let mut best_trial = None;
    let mut best_ratio = 0.0;

    for subgroup_id in 0..instance.subgroups.len() {
        if state.subgroup_nodes_count.contains_key(&subgroup_id) {
            continue;
        }

        if let Some((trial_sol, trial_state)) =
            evaluate_subgroup_insertion(instance, solution, state, subgroup_id)
        {
            let delta_cost = trial_sol.total_cost - solution.total_cost;
            let profit = instance.subgroups[subgroup_id].profit;
            let ratio = profit / (delta_cost + EPSILON);

            if ratio > best_ratio {
                best_ratio = ratio;
                best_trial = Some((trial_sol, trial_state));
            }
        }
    }

    best_trial
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::common::instance::{Cluster, Metric, Node, Point3, Subgroup, Vehicle};

    fn create_test_instance() -> Instance {
        Instance {
            name: "greedy_test".to_string(),
            metric: Metric::Euc2d,
            nodes: vec![
                Node {
                    id: 0,
                    point: Point3 { x: 0.0, y: 0.0, z: 0.0 },
                    parent_subgroup_ids: HashSet::from([0]),
                },
                Node {
                    id: 1,
                    point: Point3 { x: 0.0, y: 3.0, z: 0.0 },
                    parent_subgroup_ids: HashSet::from([1]),
                },
                Node {
                    id: 2,
                    point: Point3 { x: 4.0, y: 0.0, z: 0.0 },
                    parent_subgroup_ids: HashSet::from([2]),
                },
            ],
            subgroups: vec![
                Subgroup { id: 0, profit: 5.0, node_ids: vec![0], parent_cluster_ids: HashSet::from([0]) },
                Subgroup { id: 1, profit: 50.0, node_ids: vec![1], parent_cluster_ids: HashSet::from([1]) },
                Subgroup { id: 2, profit: 20.0, node_ids: vec![2], parent_cluster_ids: HashSet::from([2]) },
            ],
            clusters: vec![
                Cluster { id: 0, subgroup_ids: vec![0] },
                Cluster { id: 1, subgroup_ids: vec![1] },
                Cluster { id: 2, subgroup_ids: vec![2] },
            ],
            vehicles: vec![
                Vehicle { id: 0, budget: 30.0, start_node_id: 0, end_node_id: 0 },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_initialize_empty_solution() {
        let instance = create_test_instance();
        let (solution, state) = initialize_empty_solution(&instance);

        assert_eq!(solution.routes.len(), 1);
        assert_eq!(solution.routes[0].path, vec![0, 0]);
        assert_eq!(solution.total_cost, 0.0);
        assert!(state.visited_nodes.contains(&0));
        assert_eq!(state.cluster_locks.get(&0), Some(&0));
        assert_eq!(state.subgroup_nodes_count.get(&0), Some(&1));
    }

    #[test]
    fn test_build_greedy_solution() {
        let instance = create_test_instance();
        let (solution, state) = build_greedy_solution(&instance).unwrap();

        // Subgroup 1 has highest profit/cost ratio (50 / 6 = 8.33), so it should be inserted first
        assert!(solution.total_score >= 50.0);
        assert!(solution.total_cost <= 30.0);
        assert!(state.visited_nodes.contains(&1));
    }

    #[test]
    fn test_initialize_and_build_different_origin_destination() {
        use crate::common::instance::{Metric, Point3};
        let instance = Instance {
            name: "greedy_diff_endpoints".to_string(),
            metric: Metric::Euc2d,
            nodes: vec![
                Node { id: 0, point: Point3 { x: 0.0, y: 0.0, z: 0.0 }, ..Default::default() },
                Node { id: 1, point: Point3 { x: 0.0, y: 3.0, z: 0.0 }, ..Default::default() },
                Node { id: 2, point: Point3 { x: 4.0, y: 0.0, z: 0.0 }, ..Default::default() },
                Node { id: 3, point: Point3 { x: 4.0, y: 3.0, z: 0.0 }, ..Default::default() },
            ],
            subgroups: vec![
                Subgroup { id: 0, profit: 10.0, node_ids: vec![1], parent_cluster_ids: HashSet::from([0]) },
                Subgroup { id: 1, profit: 20.0, node_ids: vec![2], parent_cluster_ids: HashSet::from([1]) },
            ],
            clusters: vec![
                Cluster { id: 0, subgroup_ids: vec![0] },
                Cluster { id: 1, subgroup_ids: vec![1] },
            ],
            vehicles: vec![
                Vehicle { id: 0, budget: 50.0, start_node_id: 0, end_node_id: 3 },
            ],
            ..Default::default()
        };

        let (init_sol, init_state) = initialize_empty_solution(&instance);
        assert_eq!(init_sol.routes[0].path, vec![0, 3]);
        assert_eq!(init_sol.total_cost, 5.0);
        assert!(init_state.visited_nodes.contains(&0));
        assert!(init_state.visited_nodes.contains(&3));

        let (solution, _state) = build_greedy_solution(&instance).unwrap();
        assert_eq!(solution.routes[0].path.first(), Some(&0));
        assert_eq!(solution.routes[0].path.last(), Some(&3));
        assert_eq!(solution.total_score, 30.0);
    }
}

