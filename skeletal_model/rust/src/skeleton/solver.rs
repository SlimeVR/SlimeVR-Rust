//! Contains the math necessary for the skeleton solver.
//!
//! At a high level, the skeleton has inputs and outputs. Inputs are things like
//! [`Edge::input_rot_g`], which is an (optional) constraint on the rotation of an
//! [`Edge`]. Outputs are things like [`Edge::output_rot_g`], which needs to be solved
//! for.
//!
//! The goal of the solver is to solve for all the outputs, using the inputs. For more
//! info, see the [`skeleton`](crate::skeleton) module.

#[cfg(doc)]
use crate::skeleton::Edge;

use crate::skeleton::Graph;
use crate::Skeleton;

use derive_more::From;
use petgraph::graph::{EdgeIndex, NodeIndex};
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
	fn traverse(
		&mut self,
		mut f: impl FnMut(&mut Graph, PoppedNode, Neighbors),
	) -> Result<(), SolveError> {
		// We want to "expand outward" from root nodes with constrained position.
		// This means we perform a breadth-first traversal with these root nodes in the
		// initial set.
		let root_nodes: VecDeque<_> = self.find_root_nodes().collect();
		if root_nodes.is_empty() {
			return Err(SolveError::NoRootNode);
		}

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
					&mut self.graph,
					popped.into(),
					Neighbors {
						edge,
						node: MaybeSolvedNode::new(node, is_solved),
					},
				);

				visited_edges.insert(edge);
			}
		}
		Ok(())
	}
}

/// Argument to [`do_fk`]. Represents the neighbors of a [`PoppedNode`].
#[derive(From)]
struct Neighbors {
	pub edge: EdgeIndex,
	pub node: MaybeSolvedNode,
}

/// A node neighboring [`PoppedNode`]. This node may or may not already be solved, which
/// is why this is an enum.
enum MaybeSolvedNode {
	Solved(NodeIndex),
	Unsolved(NodeIndex),
}
impl MaybeSolvedNode {
	fn new(n: NodeIndex, solved: bool) -> Self {
		if solved {
			Self::Solved(n)
		} else {
			Self::Unsolved(n)
		}
	}
}

/// Argument to [`do_fk`]. Represents the node that was just popped off of the traversal
/// queue. This node is already solved.
#[derive(From)]
struct PoppedNode(pub NodeIndex);

/// Solves `Neighbors` by applying forward-kinematics from `PoppedNode`. This actually
/// mutates the weights of the graph.
///
/// For more info, see [`crate::skeleton`].
fn do_fk(
	_g: &mut Graph,
	PoppedNode(_popped): PoppedNode,
	Neighbors { edge, node }: Neighbors,
) {
	let _edge = edge; // unused
	match node {
		MaybeSolvedNode::Solved(_node) => {
			todo!("popped -> edge <- node")
		}
		MaybeSolvedNode::Unsolved(_node) => {
			todo!("popped -> edge + node")
		}
	}
}

#[derive(thiserror::Error, Debug)]
pub enum SolveError {
	#[error("Need at least one \"root\" `Node` (root nodes have a `input_rot_g`)")]
	NoRootNode,
}
