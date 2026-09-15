use rand::Rng;
use rand::seq::SliceRandom;

use crate::{
    common::{instance::Instance, solution::Solution},
    solvers::heuristic::vns::{
        neighborhoods::{drop_subgroup, evaluate_subgroup_insertion},
        state::SearchState,
    },
};

pub fn apply_shaking<R: Rng>(
    instance: &Instance,
    solution: &mut Solution,
    state: &mut SearchState,
    rng: &mut R,
    shaking_intensity: usize,
) {
    apply_destruction_phase(instance, solution, state, rng, shaking_intensity);
    apply_kick_phase(instance, solution, state, rng);
}

fn apply_destruction_phase<R: Rng>(
    instance: &Instance,
    solution: &mut Solution,
    state: &mut SearchState,
    rng: &mut R,
    shaking_intensity: usize,
) {
    let active_subgroups: Vec<usize> = state.subgroup_nodes_count.keys().copied().collect();

    if active_subgroups.is_empty() {
        return;
    }

    let amount_to_drop = shaking_intensity.min(active_subgroups.len());
    let to_remove: Vec<&usize> = active_subgroups
        .choose_multiple(rng, amount_to_drop)
        .collect();

    for &sg_id in to_remove {
        drop_subgroup(instance, solution, state, sg_id);
    }
}

fn apply_kick_phase<R: Rng>(
    instance: &Instance,
    solution: &mut Solution,
    state: &mut SearchState,
    rng: &mut R,
) {
    let unvisited_subgroups: Vec<usize> = (0..instance.subgroups.len())
        .filter(|subgroup_id| !state.subgroup_nodes_count.contains_key(subgroup_id))
        .collect();

    if let Some(&random_subgroup) = unvisited_subgroups.choose(rng)
        && let Some((new_sol, new_state)) =
            evaluate_subgroup_insertion(instance, solution, state, random_subgroup)
    {
        *solution = new_sol;
        *state = new_state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::collections::HashSet;
    use std::time::Duration;
    use crate::common::instance::{Cluster, Metric, Node, Point3, Subgroup, Vehicle};
    use crate::common::solution::{Route, SolutionStatus};

    fn create_test_instance() -> Instance {
        Instance {
            name: "shaking_test".to_string(),
            metric: Metric::Euc2d,
            nodes: vec![
                Node { id: 0, point: Point3 { x: 0.0, y: 0.0, z: 0.0 }, ..Default::default() },
                Node { id: 1, point: Point3 { x: 0.0, y: 3.0, z: 0.0 }, ..Default::default() },
                Node { id: 2, point: Point3 { x: 4.0, y: 0.0, z: 0.0 }, ..Default::default() },
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
                Vehicle { id: 0, budget: 50.0, start_node_id: 0, end_node_id: 0 },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_apply_destruction_phase() {
        let instance = create_test_instance();
        let mut rng = StdRng::seed_from_u64(42);

        // Create solution with Subgroup 0 and Subgroup 1 active
        let (mut solution, mut state) = (
            Solution {
                instance: &instance,
                duration: Duration::from_secs(0),
                total_score: 30.0,
                total_cost: 14.0,
                routes: vec![Route { vehicle_id: 0, path: vec![0, 1, 2, 0], cost: 14.0 }],
                status: SolutionStatus::Feasible,
                solver: None,
                best_bound: None,
                gap: None,
                explored_nodes: None,
            },
            SearchState {
                visited_nodes: [0, 1, 2].into_iter().collect(),
                cluster_locks: [(0, 0), (1, 1)].into_iter().collect(),
                subgroup_nodes_count: [(0, 1), (1, 1)].into_iter().collect(),
            },
        );

        // Apply destruction phase dropping 1 subgroup
        apply_destruction_phase(&instance, &mut solution, &mut state, &mut rng, 1);

        // Exactly one subgroup should be dropped
        assert_eq!(state.subgroup_nodes_count.len(), 1);
        assert!(solution.total_score < 30.0);
    }

    #[test]
    fn test_apply_destruction_phase_empty_active_subgroups() {
        let instance = create_test_instance();
        let mut rng = StdRng::seed_from_u64(42);

        let (mut solution, mut state) = (
            Solution {
                instance: &instance,
                duration: Duration::from_secs(0),
                total_score: 0.0,
                total_cost: 0.0,
                routes: vec![Route { vehicle_id: 0, path: vec![0, 0], cost: 0.0 }],
                status: SolutionStatus::Feasible,
                solver: None,
                best_bound: None,
                gap: None,
                explored_nodes: None,
            },
            SearchState::default(),
        );

        apply_destruction_phase(&instance, &mut solution, &mut state, &mut rng, 1);
        assert_eq!(solution.total_score, 0.0);
    }

    #[test]
    fn test_apply_kick_phase() {
        let instance = create_test_instance();
        let mut rng = StdRng::seed_from_u64(42);

        let (mut solution, mut state) = (
            Solution {
                instance: &instance,
                duration: Duration::from_secs(0),
                total_score: 0.0,
                total_cost: 0.0,
                routes: vec![Route { vehicle_id: 0, path: vec![0, 0], cost: 0.0 }],
                status: SolutionStatus::Feasible,
                solver: None,
                best_bound: None,
                gap: None,
                explored_nodes: None,
            },
            SearchState::default(),
        );

        apply_kick_phase(&instance, &mut solution, &mut state, &mut rng);

        // One subgroup should be kicked/inserted into the solution
        assert!(solution.total_score > 0.0);
        assert_eq!(state.subgroup_nodes_count.len(), 1);
    }

    #[test]
    fn test_apply_shaking_full() {
        let instance = create_test_instance();
        let mut rng = StdRng::seed_from_u64(123);

        let (mut solution, mut state) = (
            Solution {
                instance: &instance,
                duration: Duration::from_secs(0),
                total_score: 30.0,
                total_cost: 14.0,
                routes: vec![Route { vehicle_id: 0, path: vec![0, 1, 2, 0], cost: 14.0 }],
                status: SolutionStatus::Feasible,
                solver: None,
                best_bound: None,
                gap: None,
                explored_nodes: None,
            },
            SearchState {
                visited_nodes: [0, 1, 2].into_iter().collect(),
                cluster_locks: [(0, 0), (1, 1)].into_iter().collect(),
                subgroup_nodes_count: [(0, 1), (1, 1)].into_iter().collect(),
            },
        );

        apply_shaking(&instance, &mut solution, &mut state, &mut rng, 2);
        assert!(solution.total_score == 10.0);
    }
}
