use std::time::Duration;

use good_lp::Solution as SolutionTrait;

use crate::common::{
    instance::Instance,
    solution::{Route, Solution, SolutionStatus},
};

use crate::solvers::exact::good_lp::ilp::DecisionVariables;

pub fn parse_solution<'a, S: SolutionTrait>(
    solution: S,
    variables: DecisionVariables,
    instance: &'a Instance,
    duration: Duration,
) -> Solution<'a> {
    let mut routes: Vec<Route> = Vec::new();
    for k in 0..instance.vehicles.len() {
        match get_route(instance, &solution, &variables, k) {
            Some(route) => routes.push(route),
            None => continue,
        }
    }

    let total_score: f64 = (0..instance.subgroups.len())
        .filter(|&s| solution.value(variables.z[s]) >= 0.5)
        .map(|s| instance.subgroups[s].profit)
        .sum();

    let total_cost = routes.iter().map(|r| r.cost).sum();

    Solution {
        instance,
        duration,
        total_score,
        total_cost,
        routes,
        status: SolutionStatus::Optimal,
        solver: Some("Good LP".to_string()),
        best_bound: None,
        gap: None,
        explored_nodes: None,
    }
}

fn get_route<S: SolutionTrait>(
    instance: &Instance,
    solution: &S,
    variables: &DecisionVariables,
    k: usize,
) -> Option<Route> {
    let current_route_nodes = get_route_node(instance, solution, variables, k);

    if current_route_nodes.is_empty() {
        return None;
    }

    let mut route_cost = 0.0;

    for i in 0..current_route_nodes.len() - 1 {
        let current_id = current_route_nodes[i];
        let next_id = current_route_nodes[i + 1];

        route_cost += instance.get_distance(current_id, next_id);
    }

    let route = Route {
        path: current_route_nodes,
        cost: route_cost,
        vehicle_id: k,
    };

    Some(route)
}

fn get_route_node<S: SolutionTrait>(
    instance: &Instance,
    solution: &S,
    variables: &DecisionVariables,
    k: usize,
) -> Vec<usize> {
    let mut current_route_nodes: Vec<usize> = Vec::new();

    let mut current_node = instance.vehicles[k].start_node_id;
    let vehicle_end_node = instance.vehicles[k].end_node_id;
    current_route_nodes.push(current_node);

    let mut found_next;
    let num_nodes = instance.nodes.len();

    for _ in 0..num_nodes + 2 {
        found_next = false;

        for next_node in 0..num_nodes {
            if current_node == next_node {
                continue;
            }

            let val = solution.value(variables.x[k][current_node][next_node]);

            if val >= 0.5 {
                current_route_nodes.push(next_node);
                current_node = next_node;
                found_next = true;
                break;
            }
        }

        if !found_next || current_node == vehicle_end_node {
            break;
        }
    }

    current_route_nodes
}
