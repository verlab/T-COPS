use crate::common::{instance::Instance, solution::Solution};
use crate::solvers::heuristic::vns::{
    neighborhoods::evaluate_subgroup_insertion, state::SearchState,
};

pub fn local_search_insertions(
    instance: &Instance,
    solution: &mut Solution,
    state: &mut SearchState,
) {
    while let Some((new_sol, new_state)) = find_best_improving_insertion(instance, solution, state)
    {
        *solution = new_sol;
        *state = new_state;
    }
}

fn find_best_improving_insertion<'a>(
    instance: &Instance,
    solution: &Solution<'a>,
    state: &SearchState,
) -> Option<(Solution<'a>, SearchState)> {
    let mut best_trial = None;
    let mut best_obj_value = solution.get_objective_value();

    for subgroup_id in 0..instance.subgroups.len() {
        if state.subgroup_nodes_count.contains_key(&subgroup_id) {
            continue;
        }

        if let Some((trial_sol, trial_state)) =
            evaluate_subgroup_insertion(instance, solution, state, subgroup_id)
        {
            let trial_obj_value = trial_sol.get_objective_value();

            if trial_obj_value > best_obj_value {
                best_obj_value = trial_obj_value;
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
    use std::time::Duration;
    use crate::common::instance::{Cluster, Metric, Node, Point3, Subgroup, Vehicle};
    use crate::common::solution::{Route, SolutionStatus};

    fn create_test_instance() -> Instance {
        Instance {
            name: "ls_test".to_string(),
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
    fn test_local_search_insertions() {
        let instance = create_test_instance();

        let route = Route {
            vehicle_id: 0,
            path: vec![0, 0],
            cost: 0.0,
        };
        let mut state = SearchState::default();
        state.visited_nodes.insert(0);

        let mut solution = Solution {
            instance: &instance,
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

        // Run local search - it should insert both subgroup 0 (profit 10) and subgroup 1 (profit 20)
        local_search_insertions(&instance, &mut solution, &mut state);

        assert_eq!(solution.total_score, 30.0);
        assert!(solution.routes[0].path.contains(&1));
        assert!(solution.routes[0].path.contains(&2));
        assert!(state.visited_nodes.contains(&1));
        assert!(state.visited_nodes.contains(&2));
    }
}
