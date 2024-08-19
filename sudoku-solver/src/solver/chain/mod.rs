use std::fmt::Write;

use crate::solver::{SolutionRecorder, SudokuSolver, Technique};
use crate::sudoku::{CellIndex, CellValue};

use itertools::Itertools;
use rustc_hash::FxHashMap;

pub struct Assumption {
    kind: AssumptionKind,
    cell: CellIndex,
    value: CellValue,
    added_to_solution: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssumptionKind {
    On,
    Off,
}

type EdgeId = u32;

#[derive(Debug)]
pub struct Edge {
    start: NodeId,
    end: NodeId,
    next: Option<EdgeId>,
    rev_next: Option<EdgeId>,
    /// If the edge is a chain, the start_middle node is the next node of the start node and the end node is the next node of the middle_end node.
    start_middle: Option<NodeId>,
    middle_end: Option<NodeId>,
}

// save the graph as chain foward star
pub struct Graph {
    nodes: Vec<Assumption>,
    heads: Vec<Option<EdgeId>>,
    edges: Vec<Edge>,
    rev_heads: Vec<Option<EdgeId>>,
    edge_set: FxHashMap<(NodeId, NodeId), EdgeId>,
}

type NodeId = u32;

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            heads: vec![],
            edges: vec![],
            rev_heads: vec![],
            edge_set: FxHashMap::default(),
        }
    }

    pub fn get_node(&self, idx: NodeId) -> &Assumption {
        &self.nodes[idx as usize]
    }

    pub fn get_node_mut(&mut self, idx: NodeId) -> &mut Assumption {
        &mut self.nodes[idx as usize]
    }

    pub fn get_edge_by_id(&self, idx: EdgeId) -> &Edge {
        &self.edges[idx as usize]
    }

    pub fn add_node(&mut self, assumption: Assumption) -> NodeId {
        let idx = self.nodes.len();
        self.nodes.push(assumption);
        self.heads.push(None);
        self.rev_heads.push(None);
        idx as NodeId
    }

    pub fn add_edge(&mut self, start: NodeId, end: NodeId) {
        self.add_big_edge(start, end, None, None)
    }

    pub fn add_big_edge(
        &mut self,
        start: NodeId,
        end: NodeId,
        start_middle: Option<NodeId>,
        middle_end: Option<NodeId>,
    ) {
        debug_assert_ne!(start, end);
        if self.edge_set.contains_key(&(start, end)) {
            return;
        }

        let edge_id = self.edges.len() as EdgeId;
        let old_head = self.heads[start as usize];
        self.heads[start as usize] = Some(edge_id);
        let old_rev_head = self.rev_heads[end as usize];
        self.rev_heads[end as usize] = Some(edge_id);
        self.edge_set.insert((start, end), edge_id);
        self.edges.push(Edge {
            start,
            end,
            start_middle,
            middle_end,
            next: old_head,
            rev_next: old_rev_head,
        });
    }

    pub fn get_edge(&self, start: NodeId, end: NodeId) -> Option<&Edge> {
        self.edge_set
            .get(&(start, end))
            .map(|&idx| &self.edges[idx as usize])
    }

    pub fn path_to_string(&self, sudoku: &SudokuSolver, start: NodeId, end: NodeId) -> String {
        let write_path = |path: &mut dyn Write, assumption: &Assumption, trailing_space: bool| {
            if assumption.kind == AssumptionKind::On {
                write!(
                    path,
                    "{}={}",
                    sudoku.get_cell_name(assumption.cell),
                    assumption.value
                )
                .unwrap();
            } else {
                write!(
                    path,
                    "{}<>{}",
                    sudoku.get_cell_name(assumption.cell),
                    assumption.value
                )
                .unwrap();
            }
            if trailing_space {
                write!(path, " ").unwrap();
            }
        };

        let mut path = String::new();
        let mut edge = self.get_edge(start, end).unwrap();
        while edge.start_middle.is_some() {
            write_path(&mut path, self.get_node(edge.start), true);
            edge = self.get_edge(edge.start_middle.unwrap(), end).unwrap();
        }
        write_path(&mut path, self.get_node(edge.start), true);
        write_path(&mut path, self.get_node(edge.end), false);
        path
    }
}

pub fn solve_chain(sudoku: &SudokuSolver, solution: &mut SolutionRecorder) {
    let mut graph = Graph::new();

    let mut on_assumptions = [[None; 9]; 81];
    let mut off_assumptions = [[None; 9]; 81];

    for cell in sudoku.unfilled_cells() {
        for value in sudoku.candidates(cell) {
            on_assumptions[cell as usize][value as usize - 1] = Some(graph.add_node(Assumption {
                kind: AssumptionKind::On,
                cell,
                value,
                added_to_solution: false,
            }));
            off_assumptions[cell as usize][value as usize - 1] = Some(graph.add_node(Assumption {
                kind: AssumptionKind::Off,
                cell,
                value,
                added_to_solution: false,
            }));
        }
    }

    for cell in sudoku.unfilled_cells().iter() {
        for (i, on) in on_assumptions[cell as usize]
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_some())
            .map(|(i, x)| (i, x.unwrap()))
        {
            // turning a cell's value on makes all other values in the same cell off
            for (j, off) in off_assumptions[cell as usize]
                .iter()
                .enumerate()
                .filter(|(_, x)| x.is_some())
                .map(|(i, x)| (i, x.unwrap()))
            {
                if i == j {
                    continue;
                }
                graph.add_edge(on, off);
            }

            // turning a cell's value on makes the same value in other cells within the same house off
            for other in sudoku.house_union_of_cell(cell).iter() {
                for off in off_assumptions[other as usize].iter() {
                    if let Some(&off) = off.as_ref() {
                        if graph.get_node(off).value == graph.get_node(on).value {
                            graph.add_edge(on, off);
                        }
                    }
                }
            }
        }

        // Naked Single
        // if the cell is a bivalue cell, turning one value off makes the other value on
        if sudoku.candidates(cell).size() == 2 {
            let value1 = sudoku.candidates(cell).values()[0];
            let value2 = sudoku.candidates(cell).values()[1];
            let on1 = on_assumptions[cell as usize][value1 as usize - 1].unwrap();
            let on2 = on_assumptions[cell as usize][value2 as usize - 1].unwrap();
            let off1 = off_assumptions[cell as usize][value1 as usize - 1].unwrap();
            let off2 = off_assumptions[cell as usize][value2 as usize - 1].unwrap();
            graph.add_edge(off1, on2);
            graph.add_edge(off2, on1);
        }
    }

    for value in 1..=9 {
        for house in sudoku.all_constraints() {
            // Hidden Single
            // if there are only two possible cells for a value in a house, turning one cell's value off makes the other cell's value on
            let possible_cells = sudoku.get_possible_cells_for_house_and_value(house, value);
            if possible_cells.size() == 2 {
                let cell1 = possible_cells.values()[0];
                let cell2 = possible_cells.values()[1];
                let on1 = on_assumptions[cell1 as usize][value as usize - 1].unwrap();
                let on2 = on_assumptions[cell2 as usize][value as usize - 1].unwrap();
                let off1 = off_assumptions[cell1 as usize][value as usize - 1].unwrap();
                let off2 = off_assumptions[cell2 as usize][value as usize - 1].unwrap();
                graph.add_edge(off1, on2);
                graph.add_edge(off2, on1);
            }
        }
    }

    // Expanding the graph by adding edges from a node to all other nodes it can reach.
    // Later we will check whether a node representing an "on" state can reach its corresponding "off" state,
    // which means the assumption is invalid by contradiction.
    let mut idx = 0;

    // When expanding the graph, we only expend the edges with length 1.
    // This can be done by backing up the heads and rev_heads and iterating through the edges,
    // since the new edges are always added to the front.
    let heads = graph.heads.clone();
    let rev_heads = graph.rev_heads.clone();
    while idx < graph.edges.len() {
        let u = graph.edges[idx].start;
        let v = graph.edges[idx].end;

        let mut v_to_w_ = heads[v as usize];
        while let Some(v_to_w) = v_to_w_.map(|e| graph.get_edge_by_id(e)) {
            debug_assert!(v_to_w.start == v);
            v_to_w_ = v_to_w.next;
            let w = v_to_w.end;
            if u != w {
                graph.add_big_edge(
                    u,
                    w,
                    graph.edges[idx].start_middle.or(Some(v)),
                    v_to_w.middle_end.or(Some(v)),
                );
            }
        }

        let v = graph.edges[idx].start;
        let w = graph.edges[idx].end;

        let mut u_to_v_ = rev_heads[v as usize];
        while let Some(u_to_v) = u_to_v_.map(|e| graph.get_edge_by_id(e)) {
            debug_assert!(u_to_v.end == v);
            let u = u_to_v.start;
            u_to_v_ = u_to_v.rev_next;
            if u != w {
                graph.add_big_edge(
                    u,
                    w,
                    u_to_v.start_middle.or(Some(v)),
                    graph.edges[idx].middle_end.or(Some(v)),
                );
            }
        }

        idx += 1;
    }

    // All the nodes that can reach the contradiction node are also forced to be false, that is, their opposite nodes are forced to be true.
    let check_can_reach_contradiction =
        |solution: &mut SolutionRecorder, graph: &mut Graph, contradiction: NodeId| {
            let mut edge_ = graph.rev_heads[contradiction as usize];
            while let Some(edge) = edge_.map(|e| graph.get_edge_by_id(e)) {
                edge_ = edge.rev_next;
                let node = graph.get_node(edge.start);
                let opposite_node = if node.kind == AssumptionKind::On {
                    off_assumptions[node.cell as usize][node.value as usize - 1].unwrap()
                } else {
                    on_assumptions[node.cell as usize][node.value as usize - 1].unwrap()
                };
                let opposite = graph.get_node(opposite_node);
                if !opposite.added_to_solution {
                    if opposite.kind == AssumptionKind::On {
                        solution.add_value_set(
                            Technique::Chain,
                            format!(
                                "contradiction\n{}",
                                graph.path_to_string(sudoku, edge.start, edge.end)
                            ),
                            opposite.cell,
                            opposite.value,
                        );
                    } else {
                        solution.add_elimination(
                            Technique::Chain,
                            format!(
                                "contradiction\n{}",
                                graph.path_to_string(sudoku, edge.start, edge.end)
                            ),
                            opposite.cell,
                            opposite.value,
                        );
                    }
                    graph.get_node_mut(opposite_node).added_to_solution = true;
                }
            }
        };

    // Check whether there is a contradiction in the graph, i.e. whether an "on" node can reach its corresponding "off" node.
    for cell in sudoku.unfilled_cells() {
        for value in sudoku.candidates(cell) {
            let on = on_assumptions[cell as usize][value as usize - 1].unwrap();
            let off = off_assumptions[cell as usize][value as usize - 1].unwrap();
            if let Some(_) = graph.edge_set.get(&(on, off)) {
                let eliminated_cell = graph.get_node(off).cell;
                let eliminated_value = graph.get_node(off).value;
                solution.add_elimination(
                    Technique::Chain,
                    format!(
                        "contradiction if {} is {}\n{}",
                        sudoku.get_cell_name(cell),
                        value,
                        graph.path_to_string(sudoku, on, off),
                    ),
                    eliminated_cell,
                    eliminated_value,
                );
                graph.get_node_mut(off).added_to_solution = true;
                check_can_reach_contradiction(solution, &mut graph, on);
            }
            if let Some(_) = graph.edge_set.get(&(off, on)) {
                let forced_cell = graph.get_node(on).cell;
                let forced_value = graph.get_node(on).value;
                solution.add_value_set(
                    Technique::Chain,
                    format!(
                        "contradiction if {} is not {}\n{}",
                        sudoku.get_cell_name(cell),
                        value,
                        graph.path_to_string(sudoku, off, on)
                    ),
                    forced_cell,
                    forced_value,
                );
                graph.get_node_mut(on).added_to_solution = true;
                check_can_reach_contradiction(solution, &mut graph, off);
            }
        }
    }

    // Check the nodes that are reached by all "on" nodes of a cell.
    // If all the "on" nodes of a cell reach some nodes, then the nodes are forced to be true.
    for cell in sudoku.unfilled_cells() {
        let mut reached = vec![0; graph.nodes.len()];
        for value in sudoku.candidates(cell) {
            let on = on_assumptions[cell as usize][value as usize - 1].unwrap();
            let mut edge = graph.heads[on as usize].map(|e| graph.get_edge_by_id(e));
            while let Some(e) = edge {
                reached[e.end as usize] += 1;
                edge = e.next.map(|e| graph.get_edge_by_id(e));
            }
        }
        for (i, &count) in reached.iter().enumerate() {
            if count != sudoku.candidates(cell).size() {
                continue;
            }
            let assumption = &graph.nodes[i];
            if assumption.added_to_solution {
                continue;
            }
            let all_paths = sudoku
                .candidates(cell)
                .iter()
                .map(|value| {
                    let on = on_assumptions[cell as usize][value as usize - 1].unwrap();
                    graph.path_to_string(sudoku, on, i as NodeId)
                })
                .join("\n");
            if assumption.kind == AssumptionKind::On {
                solution.add_value_set(
                    Technique::Chain,
                    format!(
                        "What ever value {} is filled, {} must be {}\n{}",
                        sudoku.get_cell_name(cell),
                        sudoku.get_cell_name(assumption.cell),
                        assumption.value,
                        all_paths,
                    ),
                    assumption.cell,
                    assumption.value,
                );
                graph.nodes[i].added_to_solution = true;
            } else {
                solution.add_elimination(
                    Technique::Chain,
                    format!(
                        "What ever the value of {} is, {} cannot be {}\n{}",
                        sudoku.get_cell_name(cell),
                        sudoku.get_cell_name(assumption.cell),
                        assumption.value,
                        all_paths,
                    ),
                    assumption.cell,
                    assumption.value,
                );
                graph.nodes[i].added_to_solution = true;
            }
        }
    }

    // Check the nodes that are reached by all "on" nodes of a value.
    // If all the "on" nodes of a value reach some nodes, then the nodes are forced to be true.
    for house in sudoku.all_constraints() {
        for value in 1..=9 {
            let all_count = sudoku
                .get_possible_cells_for_house_and_value(house, value)
                .size();

            if all_count == 0 {
                continue;
            }

            let mut reached = vec![0; graph.nodes.len()];
            for cell in sudoku
                .get_possible_cells_for_house_and_value(house, value)
                .iter()
            {
                let on = on_assumptions[cell as usize][value as usize - 1].unwrap();
                let mut edge = graph.heads[on as usize].map(|e| graph.get_edge_by_id(e));
                while let Some(e) = edge {
                    reached[e.end as usize] += 1;
                    edge = e.next.map(|e| graph.get_edge_by_id(e));
                }
            }

            for (assumption_idx, &count) in reached.iter().enumerate() {
                if count != all_count {
                    continue;
                }
                let assumption = &graph.nodes[assumption_idx];
                if assumption.added_to_solution {
                    continue;
                }
                let all_paths = sudoku
                    .get_possible_cells_for_house_and_value(house, value)
                    .iter()
                    .map(|cell| {
                        let on = on_assumptions[cell as usize][value as usize - 1].unwrap();
                        graph.path_to_string(sudoku, on, assumption_idx as NodeId)
                    })
                    .join("\n");
                if assumption.kind == AssumptionKind::On {
                    solution.add_value_set(
                        Technique::Chain,
                        format!(
                            "Where ever the value of {} is in {}, {} must be {}\n{}",
                            value,
                            house.name(),
                            sudoku.get_cell_name(assumption.cell),
                            assumption.value,
                            all_paths,
                        ),
                        assumption.cell,
                        assumption.value,
                    );
                    graph.nodes[assumption_idx].added_to_solution = true;
                } else {
                    solution.add_elimination(
                        Technique::Chain,
                        format!(
                            "Where ever the value of {} is in {}, {} cannot be {}\n{}",
                            value,
                            house.name(),
                            sudoku.get_cell_name(assumption.cell),
                            assumption.value,
                            all_paths,
                        ),
                        assumption.cell,
                        assumption.value,
                    );
                    graph.nodes[assumption_idx].added_to_solution = true;
                }
            }
        }
    }
}
