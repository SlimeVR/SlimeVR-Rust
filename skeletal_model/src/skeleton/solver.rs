//! Contains the math necessary for the skeleton solver.
//!
//! At a high level, the skeleton has inputs and outputs. Inputs are things like
//! [`Edge::input_rot_g`], which is an (optional) constraint on the rotation of an
//! [`Edge`]. Outputs are things like [`Edge::output_rot_g`], which needs to be solved
//! for.
//!
//! The goal of the solver is to solve for all the outputs, using the inputs. For more
//! info, see the [`skeleton`](crate::skeleton) module.

use crate::skeleton::{Edge, Node};
use crate::Skeleton;

use petgraph::visit::{VisitMap, Visitable};
use std::collections::{HashSet, VecDeque};

impl Skeleton {
	/// Solves for the outputs of the skeletal model.
	///
	/// For more info on the algorithm, see [`crate::skeleton`].
	pub fn solve(&mut self) -> Result<(), SolveError> {
		self.traverse(do_fk)
	}

	/// Traverses all edges and nodes in the graph in a breadth-first search. Calls `f` at the
	/// start of each traversal.
	fn traverse<'a>(
		&'a mut self,
		mut f: impl FnMut(
			Solved<&'a mut Node, &'a mut Edge>,
			Neighbors<&'a mut Node, &'a mut Edge>,
		),
	) -> Result<(), SolveError> {
		// We want to "expand outward" from root nodes with constrained position.
		// This means we perform a breadth-first traversal with these root nodes in the
		// initial set.
		let root_nodes: VecDeque<_> = self.find_root_nodes().collect();
		if root_nodes.is_empty() {
			return Err(SolveError::NoRootNode);
		};

		let mut bfs = petgraph::visit::Bfs {
			stack: root_nodes,
			discovered: self.graph.visit_map(),
		};
		// We solve edges too, so we need to track if we visited them to avoid
		// double-solving.
		let mut visited_edges = HashSet::with_capacity(self.graph.edge_count());

		// Popping `None` means the queue is empty, so we terminate the loop.
		//
		// Note: any nodes that we pop we consider to already be fully solved, but their
		// neighboring edges and nodes may not yet be. The job of this nested while loop
		// is to munch on the neighbors of the popped node, solving them one by one, and
		// then queue the solved neighbor nodes to repeat the process.
		while let Some(popped) = bfs.next(&self.graph) {
			let mut neighbors = self.graph.neighbors(popped).detach();
			while let Some((edge, node)) = neighbors.next(&self.graph) {
				if visited_edges.contains(&edge) {
					continue; // If the edge was visited, the node definitely was too.
				}
				let is_solved = bfs.discovered.is_visited(&node);
				f(
					Solved {
						node: &mut self.graph[popped],
						edge: todo!(),
					},
					Neighbors {
						edge: &mut self.graph[edge],
						node: MaybeSolvedNode::new(&mut self.graph[node], is_solved),
					},
				);

				visited_edges.insert(edge);
			}
		}
		Ok(())
	}
}

/// Argument to [`do_fk`]. Represents the neighbors of [`Solved::node`].
#[derive(Debug)]
struct Neighbors<N, E> {
	node: MaybeSolvedNode<N>,
	edge: E,
}

/// Argument to [`do_fk`]. Represents the solved node that was just popped off of the
/// traversal queue, along with the solved edge connected to it.
#[derive(Debug)]
struct Solved<N, E> {
	node: N,
	edge: E,
}

/// A node neighboring [`Solved::node`]. This node may or may not already be solved,
/// which is why this is an enum.
#[derive(Debug)]
enum MaybeSolvedNode<N> {
	Solved(N),
	Unsolved(N),
}
impl<N> MaybeSolvedNode<N> {
	fn new(n: N, solved: bool) -> Self {
		if solved {
			Self::Solved(n)
		} else {
			Self::Unsolved(n)
		}
	}
}

/// Solves `Neighbors` by applying forward-kinematics from `solved`. This actually
/// mutates the weights of the graph.
///
/// For more info, see [`crate::skeleton`].
fn do_fk<N, E>(solved: Solved<N, E>, neighbors: Neighbors<N, E>) {
	let Solved {
		node: _s_node,
		edge: _s_edge,
	} = solved;
	let Neighbors {
		node: n_node,
		edge: _n_edge,
	} = neighbors;

	match n_node {
		MaybeSolvedNode::Solved(_node) => {
			todo!()
		}
		MaybeSolvedNode::Unsolved(_node) => {
			todo!()
		}
	}
}

#[derive(thiserror::Error, Debug)]
pub enum SolveError {
	#[error("Need at least one \"root\" `Node` (root nodes have a `input_rot_g`)")]
	NoRootNode,
}
