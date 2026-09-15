use std::time::Duration;

use grb::prelude::*;

use crate::common::{
    instance::Instance,
    solution::{Route, Solution, SolverMetrics},
};

use crate::solvers::exact::gurobi::variable::DecisionVariables;

pub fn parse_solution<'a>(
    model: &Model,
    variables: &DecisionVariables,
    instance: &'a Instance,
    duration: Duration,
    metrics: SolverMetrics,
    status: Status
) -> grb::Result<Solution<'a>> {
    let mut routes: Vec<Route> = Vec::new();

    for k in 0..instance.vehicles.len() {
        if let Some(route) = get_route(instance, model, variables, k)? {
            routes.push(route);
        }
    }

    let total_score: f64 = (0..instance.subgroups.len())
        .filter_map(|s| {
            let val = model.get_obj_attr(attr::X, &variables.z[s]).ok()?;
            if val >= 0.5 {
                Some(instance.subgroups[s].profit)
            } else {
                None
            }
        })
        .sum();

    let total_cost: f64 = routes.iter().map(|r| r.cost).sum();

    Ok(Solution {
        instance,
        duration,
        total_score,
        total_cost,
        routes,
        status: status.into(),
        solver: Some("Gurobi".to_string()),
        best_bound: metrics.best_bound,
        gap: metrics.gap,
        explored_nodes: metrics.explored_nodes,
    })
}

fn get_route(
    instance: &Instance,
    model: &Model,
    variables: &DecisionVariables,
    k: usize,
) -> grb::Result<Option<Route>> {
    let current_route_nodes = get_route_node(instance, model, variables, k)?;

    if current_route_nodes.is_empty() {
        return Ok(None);
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

    Ok(Some(route))
}

fn get_route_node(
    instance: &Instance,
    model: &Model,
    variables: &DecisionVariables,
    k: usize,
) -> grb::Result<Vec<usize>> {
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

            let var = &variables.x[k][current_node][next_node];
            let val = model.get_obj_attr(attr::X, var)?;

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

    Ok(current_route_nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::common::instance::{Cluster, Metric, Node, Point3, Subgroup, Vehicle};
    use crate::solvers::exact::gurobi::variable;

    fn create_test_instance() -> Instance {
        Instance {
            name: "parser_test".to_string(),
            metric: Metric::Euc2d,
            nodes: vec![
                Node { id: 0, point: Point3 { x: 0.0, y: 0.0, z: 0.0 }, ..Default::default() },
                Node { id: 1, point: Point3 { x: 0.0, y: 3.0, z: 0.0 }, ..Default::default() },
            ],
            subgroups: vec![
                Subgroup { id: 0, profit: 10.0, node_ids: vec![1], parent_cluster_ids: HashSet::from([0]) },
            ],
            clusters: vec![
                Cluster { id: 0, subgroup_ids: vec![0] },
            ],
            vehicles: vec![
                Vehicle { id: 0, budget: 20.0, start_node_id: 0, end_node_id: 0 },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_parse_solution_after_optimization() -> grb::Result<()> {
        let instance = create_test_instance();
        let env = Env::new("gurobi.log")?;
        let mut model = Model::with_env("parser_test", &env)?;

        let x = variable::initialize_x(&mut model, &instance)?;
        let y = variable::initialize_y(&mut model, &instance)?;
        let z = variable::initialize_z(&mut model, &instance)?;
        let w = variable::initialize_w(&mut model, &instance)?;

        let vars = DecisionVariables { x, y, z, w };
        
        // Solve model (empty constraints -> 0 objective)
        model.optimize()?;
        let status = model.status()?;

        let metrics = SolverMetrics {
            best_bound: Some(0.0),
            gap: Some(0.0),
            explored_nodes: Some(0),
        };

        let solution = parse_solution(&model, &vars, &instance, Duration::from_secs(1), metrics, status)?;

        assert_eq!(solution.solver, Some("Gurobi".to_string()));
        assert_eq!(solution.instance.name, "parser_test");

        Ok(())
    }
}
