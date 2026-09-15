use std::io::{Write, stdout};
use std::time::Instant;

use rand::thread_rng;

use crate::{
    common::{constants::EPSILON, error::SolverError, instance::Instance, solution::Solution},
    solvers::heuristic::vns::{
        greedy::build_greedy_solution, local_search::local_search_insertions,
        shaking::apply_shaking,
    },
};

pub fn solve<'a>(
    instance: &'a Instance,
    max_iterations_without_improvement: usize,
    max_shaking_intensity: usize,
) -> Result<Solution<'a>, SolverError> {
    let start_time = Instant::now();

    let (mut best_solution, mut best_state) = build_greedy_solution(instance)?;
    local_search_insertions(instance, &mut best_solution, &mut best_state);

    println!(
        "Starting solution found! Objective: {:.2} | Cost: {:.2}",
        best_solution.get_objective_value(),
        best_solution.total_cost
    );

    let mut iter_without_improvement = 0;
    let mut rng = thread_rng();

    while iter_without_improvement < max_iterations_without_improvement {
        iter_without_improvement += 1;
        print!(
            "\x1B[2KIterations without improvement: {}/{}\r",
            iter_without_improvement, max_iterations_without_improvement
        );
        let _ = stdout().flush();

        let max_shaking_limit = max_shaking_intensity.min(instance.subgroups.len());
        for shaking_intensity in 0..=max_shaking_limit {
            let mut trial_solution = best_solution.clone();
            let mut trial_state = best_state.clone();

            apply_shaking(instance, &mut trial_solution, &mut trial_state, &mut rng, shaking_intensity);
            local_search_insertions(instance, &mut trial_solution, &mut trial_state);

            if trial_solution.get_objective_value() > best_solution.get_objective_value() + EPSILON {
                best_solution = trial_solution;
                best_state = trial_state;
                iter_without_improvement = 0;

                println!(
                    "New best solution found! Objective: {:.2} | Cost: {:.2}",
                    best_solution.get_objective_value(),
                    best_solution.total_cost
                );

                break;
            }
        }
    }

    best_solution.total_cost = 0.0;

    for route in &mut best_solution.routes {
        if route.path.len() == 2 {
            route.path.truncate(1);
            route.cost = 0.0;
        }

        best_solution.total_cost += route.cost;
    }

    best_solution.duration = start_time.elapsed();
    print!("\x1B[2K");

    Ok(best_solution)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::common::instance::{Cluster, Metric, Node, Point3, Subgroup, Vehicle};
    use crate::common::solution::SolutionStatus;

    fn create_test_instance() -> Instance {
        Instance {
            name: "vns_solver_test".to_string(),
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
                Vehicle { id: 1, budget: 50.0, start_node_id: 0, end_node_id: 0 },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_vns_solve_basic() {
        let instance = create_test_instance();
        let solution = solve(&instance, 5, 2).expect("VNS solve should succeed");

        assert_eq!(solution.status, SolutionStatus::Feasible);
        assert_eq!(solution.total_score, 30.0);
        assert_eq!(solution.routes.len(), 2);
        assert!(solution.total_cost > 0.0);
    }

    #[test]
    fn test_vns_solve_cleans_empty_routes() {
        let instance = create_test_instance();
        let solution = solve(&instance, 2, 1).expect("VNS solve should succeed");

        let route = solution.routes.iter().find(|r| r.vehicle_id == 1).unwrap();
        assert_eq!(route.cost, 0.0);
        assert_eq!(route.path.len(), 1);
    }
}
