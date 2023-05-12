use crate::{Error, Vector3};

use hashbrown::HashMap;

use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::{
    hash::Hash,
    ops::Deref,
};

pub(crate) const OCTREE_CHILDREN: usize = 8;

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Octant {
    LeftRearBase = 0,
    RightRearBase = 1,
    LeftRearTop = 2,
    RightRearTop = 3,
    LeftFrontBase = 4,
    RightFrontBase = 5,
    LeftFrontTop = 6,
    RightFrontTop = 7,
}

impl From<usize> for Octant {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::LeftRearBase,
            1 => Self::RightRearBase,
            2 => Self::LeftRearTop,
            3 => Self::RightRearTop,
            4 => Self::LeftFrontBase,
            5 => Self::RightFrontBase,
            6 => Self::LeftFrontTop,
            7 => Self::RightFrontTop,
            _ => unreachable!("Unexpected Octant number"),
        }
    }
}

impl Octant {
    fn offset(&self) -> Vector3<u32> {
        match self {
            Self::LeftRearBase => Vector3::from([0, 0, 0]),
            Self::RightRearBase => Vector3::from([1, 0, 0]),
            Self::LeftRearTop => Vector3::from([0, 0, 1]),
            Self::RightRearTop => Vector3::from([1, 0, 1]),
            Self::LeftFrontBase => Vector3::from([0, 1, 0]),
            Self::RightFrontBase => Vector3::from([1, 1, 0]),
            Self::LeftFrontTop => Vector3::from([0, 1, 1]),
            Self::RightFrontTop => Vector3::from([1, 1, 1]),
        }
    }

    fn vector_diff(rhs: Vector3<u32>, lhs: Vector3<u32>) -> Self {
        if lhs.z < rhs.z {
            if lhs.y < rhs.y {
                if lhs.x < rhs.x {
                    Self::LeftRearBase
                } else {
                    Self::RightRearBase
                }
            } else {
                if lhs.x < rhs.x {
                    Self::LeftFrontBase
                } else {
                    Self::RightFrontBase
                }
            }
        } else {
            if lhs.y < rhs.y {
                if lhs.x < rhs.x {
                    Self::LeftRearTop
                } else {
                    Self::RightRearTop
                }
            } else {
                if lhs.x < rhs.x {
                    Self::LeftFrontTop
                } else {
                    Self::RightFrontTop
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum NodeType<T> {
    Leaf(T),
    Internal,
}

impl<T> Default for NodeType<T> {
    fn default() -> Self {
        Self::Internal
    }
}

struct ChildInfo {
    dimension: u32,
    dimension_3d: Vector3<u32>,
    octant: Octant,
}

#[derive(Default, Clone)]
pub(crate) struct Node<T>
where
    T: Default + Eq + PartialEq + Clone + Copy + Hash,
{
    ty: NodeType<T>,
    min_position: Vector3<u32>,
    dimension: u32,
    children: [Option<Box<Node<T>>>; OCTREE_CHILDREN],
}

impl<T> Node<T>
where
    T: Default + Eq + PartialEq + Clone + Copy + Hash,
{
    /// Creates a new `Node<T>` with the given bounds.
    pub(crate) fn new(min_position: Vector3<u32>, dimension: u32) -> Self {
        Self {
            ty: NodeType::Leaf(Default::default()),
            min_position,
            dimension,
            ..Default::default()
        }
    }

    /// Inserts a new leaf `Node` at the given position, if possible.
    pub(crate) fn insert(
        &mut self,
        position: Vector3<u32>,
        min_dimension: u32,
        do_simplify: bool,
        data: T,
    ) -> Result<(), Error> {
        if !self.contains(position) {
            return Err(Error::InvalidPosition {
                x: position.x,
                y: position.y,
                z: position.z,
            });
        }

        if self.dimension == min_dimension {
            self.ty = NodeType::Leaf(data);
            return Ok(());
        }

        let ChildInfo {
            dimension: child_dimension,
            dimension_3d,
            octant,
        } = self.child_info(position).unwrap();

        if self.is_leaf() && child_dimension == min_dimension {
            for i in 0..OCTREE_CHILDREN {
                if i != octant as usize {
                    let new_octant = Octant::from(i);
                    let mut new_node =
                        Node::<T>::new(self.child_min_position(dimension_3d, new_octant), child_dimension);
                    new_node.ty = NodeType::Leaf(*self.leaf_data().unwrap());
                    self.children[new_octant as usize] = Some(Box::new(new_node));
                }
            }
        }

        if let Some(ref mut child) = &mut self.children[octant as usize] {
            child
                .as_mut()
                .insert(position, min_dimension, do_simplify, data)
                .unwrap();
        } else {
            let mut node = Box::new(Node::<T>::new(
                self.child_min_position(dimension_3d, octant),
                child_dimension,
            ));
            node.insert(position, min_dimension, do_simplify, data).unwrap();
            self.children[octant as usize] = Some(node);
        };
        self.ty = NodeType::Internal;
        if do_simplify {
            self.simplify();
        }
        Ok(())
    }

    /// Removes the `Node` at the given position, if possible.
    pub(crate) fn clear(&mut self, position: Vector3<u32>, min_dimension: u32) -> Result<(), Error> {
        if self.contains(position) {
            let ChildInfo {
                dimension,
                dimension_3d: _,
                octant,
            } = self.child_info(position).unwrap();

            if self.is_leaf() && dimension == min_dimension {
                for i in 0..OCTREE_CHILDREN {
                    self.children[i] = None;
                }
            } else if self.children[octant as usize].as_ref().is_some() {
                self.children[octant as usize]
                    .as_mut()
                    .unwrap()
                    .clear(position, min_dimension)
                    .unwrap();
                self.children[octant as usize].as_mut().unwrap().ty = if self.is_leaf() || dimension == min_dimension {
                    NodeType::Leaf(Default::default())
                } else {
                    NodeType::Internal
                };
            }

            Ok(())
        } else {
            Err(Error::InvalidPosition {
                x: position.x,
                y: position.y,
                z: position.z,
            })
        }
    }

    /// Gets data from a `Node` at the given position, if possible.
    pub(crate) fn get(&self, position: Vector3<u32>) -> Option<&T> {
        if !self.contains(position) {
            return None;
        }
        return match &self.ty {
            NodeType::Leaf(data) => Some(data),
            _ => {
                let ChildInfo {
                    dimension: _,
                    dimension_3d: _,
                    octant,
                } = self.child_info(position).unwrap();
                match &self.children[octant as usize] {
                    Some(child) => child.get(position),
                    _ => None,
                }
            }
        };
    }

    /// Simplifies the `Node`.
    ///
    /// If all children are leaf `Node`s with identical data, destroy all children,
    /// and mark the `Node` as a leaf containing that data.
    pub(crate) fn simplify(&mut self) -> bool {
        let mut data = None;
        for i in 0..OCTREE_CHILDREN {
            if let Some(child) = &self.children[i] {
                if child.is_leaf() {
                    let leaf_data = child.leaf_data();

                    if data.as_ref().is_none() {
                        data = match child.ty {
                            NodeType::Leaf(d) => Some(d),
                            _ => panic!("Leaf Node `ty` member is not NodeType::Leaf(T) when it should be!"),
                        };
                    } else if *data.as_ref().unwrap() != *leaf_data.unwrap() {
                        return false;
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        self.ty = NodeType::Leaf((data.unwrap()).clone());
        self.children = [None, None, None, None, None, None, None, None];
        true
    }

    /// Simplifies node and children recursively
    pub(crate) fn simplify_recursive(&mut self) -> bool {
        let mut leaf_children = 0;
        for i in 0..OCTREE_CHILDREN {
            if let Some(ref mut child) = &mut self.children[i] {
                match child.ty {
                    NodeType::Internal => {
                        if child.simplify_recursive() {
                            leaf_children += 1
                        }
                    }
                    NodeType::Leaf(_) => {
                        leaf_children += 1;
                    }
                };
            } else {
                return false;
            }
        }
        if leaf_children == OCTREE_CHILDREN {
            self.simplify()
        } else {
            false
        }
    }

    /// Returns a higher LOD of the current `Node`.
    ///
    /// For all children of a leaf `Node`, take the most common data of all children,
    /// destroy all children, and mark the `Node` as a leaf containing that data.
    pub(crate) fn lod(&mut self) {
        let mut all_data = Vec::<T>::new();
        for (_i, c) in self.children.iter_mut().enumerate().map(|(i, c)| (i, c)) {
            if let Some(c) = c {
                if c.is_leaf() {
                    let leaf_data = c.leaf_data();
                    if leaf_data.is_some() {
                        all_data.push(match &c.ty {
                            NodeType::Leaf(d) => *d,
                            _ => panic!("Leaf Node `ty` member is not NodeType::Leaf(T) when it should be!"),
                        });
                    }
                } else {
                    c.lod();
                }
            } else {
                return;
            }
        }

        // Counting how many times a certain data value is present inside the children
        let counts = all_data.drain(..).fold(HashMap::new(), |mut acc, v| {
            acc.entry(v).and_modify(|e| *e += 1).or_insert(1);
            acc
        });

        if !counts.is_empty() {
            self.ty = NodeType::Leaf(counts.into_iter().max_by_key(|(_, count)| *count).unwrap().0);
        }

        self.children.fill(None);
    }

    /// Returns the dimension of the `Node`.
    pub(crate) fn dimension(&self) -> u32 {
        self.dimension
    }

    /// Returns whether the `Node` contains the given position.
    pub(crate) fn contains(&self, position: Vector3<u32>) -> bool {
        position.x >= self.min_position.x
            && position.x < self.min_position.x + self.dimension
            && position.y >= self.min_position.y
            && position.y < self.min_position.y + self.dimension
            && position.z >= self.min_position.z
            && position.z < self.min_position.z + self.dimension
    }

    /// Get leaf data from this `Node`.
    pub(crate) fn leaf_data(&self) -> Option<&T> {
        match &self.ty {
            NodeType::Leaf(data) => Some(&data),
            _ => None,
        }
    }

    fn child_info(&self, position: Vector3<u32>) -> Option<ChildInfo> {
        if self.contains(position) {
            let dimension = self.dimension / 2;
            let dimension_3d = Vector3::from([dimension, dimension, dimension]);
            let midpoint = self.min_position + dimension_3d;
            let octant = Octant::vector_diff(midpoint, position);

            Some(ChildInfo {
                dimension,
                dimension_3d,
                octant,
            })
        } else {
            None
        }
    }

    fn child_min_position(&self, dimension_3d: Vector3<u32>, octant: Octant) -> Vector3<u32> {
        self.min_position + dimension_3d.component_mul(&octant.offset())
    }

    fn child_count(&self) -> usize {
        self.children
            .iter()
            .fold(0, |acc, child| if child.deref().is_some() { acc + 1 } else { acc })
    }

    fn is_leaf(&self) -> bool {
        matches!(self.ty, NodeType::Leaf(_))
    }
}

use bendy::encoding::{Error as BencodeError, SingleItemEncoder, ToBencode};
impl<T> ToBencode for Node<T>
where
    T: Default + Clone + Eq + PartialEq + Copy + Hash + ToBencode + FromBencode,
{
    const MAX_DEPTH: usize = 4;
    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), BencodeError> {
        //Collect al Nodes into an array for serialization
        let mut all_nodes = Vec::<(&Node<T>, [Option<usize>; OCTREE_CHILDREN])>::new(); // Node reference and the index of each child in the same array
        let mut nodes_to_process = VecDeque::new(); // Index values of unprocessed Nodes in `all_nodes`
        nodes_to_process.push_front(0);
        all_nodes.push((self, [None; OCTREE_CHILDREN]));
        while 0 < nodes_to_process.len() {
            let current_node_index = nodes_to_process.remove(0).unwrap();
            assert!(
                current_node_index < all_nodes.len(),
                "Node to process out of bounds! {current_node_index} / {:?}",
                all_nodes.len()
            );
            let (current_node, mut indexed_children) = all_nodes[current_node_index];
            for i in 0..OCTREE_CHILDREN {
                if let Some(c) = current_node.children[i].as_ref() {
                    //If the yet unprocessed Node has a child; push it to the end of the `all_nodes` vector, and mark it to be processed
                    indexed_children[i] = Some(all_nodes.len());
                    nodes_to_process.push_back(all_nodes.len());
                    all_nodes.push((c, [None; OCTREE_CHILDREN]));
                }
            }
            all_nodes[current_node_index] = (current_node, indexed_children);
        }

        // println!("Encode:");
        // let mut n_i = 0;
        // for n in all_nodes.iter() {
        //     let d_ty = match n.0.ty {
        //         NodeType::Internal => format!("INTERNAL"),
        //         NodeType::Simplified => format!("SIMPLIFIED"),
        //         NodeType::Leaf(d) => format!("{:?}", d),
        //     };

        //     let d_bounds = format!("{:?};{:?}", n.0.min_position, n.0.dimension());
        //     let mut d_children = "[".to_owned();
        //     for c in n.1 {
        //         match c {
        //             Some(index) => d_children.push_str(format!("{index},").as_str()),
        //             _ => d_children.push_str("x,"),
        //         }
        //     }
        //     d_children.push_str("]");
        //     println!("Nodes[{}]: [{}][{}]:{}", n_i, d_ty, d_bounds, d_children);
        //     n_i += 1;
        // }

        // Serialize the array
        encoder.emit_list(|e| {
            e.emit_int(all_nodes.len())?;
            for (node_ref, node_children) in all_nodes.iter() {
                //emit Node without children
                match node_ref.ty {
                    NodeType::Internal => e.emit_str("###iNtErNaL###")?,
                    NodeType::Leaf(d) => {
                        e.emit_str("###lEaF###")?;
                        e.emit(d)?
                    }
                }

                //emit min_position
                e.emit_int(node_ref.min_position.x)?;
                e.emit_int(node_ref.min_position.y)?;
                e.emit_int(node_ref.min_position.z)?;

                //emit dimension
                e.emit_int(node_ref.dimension)?;

                //emit Node child array index values
                e.emit_list(|e2| {
                    // the value 0 can be used safely here, becaue the root node is at index 0; and it's child for noone
                    for i in 0..OCTREE_CHILDREN {
                        e2.emit_int(node_children[i].unwrap_or(0))?;
                    }
                    Ok(())
                })?;
            }
            Ok(())
        })
    }
}

use bendy::decoding::{FromBencode, Object};
impl<T> FromBencode for Node<T>
where
    T: Default + Clone + Eq + PartialEq + Copy + Hash + FromBencode,
{
    fn decode_bencode_object(data: Object) -> Result<Self, bendy::decoding::Error> {
        //Read in serialized array containing Node information
        match data {
            Object::List(mut list) => {
                let mut all_nodes = Vec::<(Option<Node<T>>, [Option<usize>; OCTREE_CHILDREN])>::new(); // The actual Node to be built and the helper index values for its children
                let node_count = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse().unwrap()),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "Integer, size of all_nodes Vec",
                        "Something else",
                    )),
                }?;

                for _ in 0..node_count {
                    use std::string::String;
                    let mut is_leaf = false;
                    let mut ty = match String::decode_bencode_object(list.next_object()?.unwrap())?.as_str() {
                        "###iNtErNaL###" => Ok(NodeType::Internal),
                        "###lEaF###" => {
                            is_leaf = true;
                            Ok(NodeType::Internal)
                        }
                        s => Err(bendy::decoding::Error::unexpected_token(
                            "NodeType markers",
                            format!("{:?}", s),
                        )),
                    }?;
                    if is_leaf {
                        ty = NodeType::<T>::Leaf(T::decode_bencode_object(list.next_object()?.unwrap())?)
                    }
                    let x = match list.next_object()?.unwrap() {
                        Object::Integer(i) => Ok(i.parse::<u32>().unwrap()),
                        _ => Err(bendy::decoding::Error::unexpected_token(
                            "Integer for Node min_position x",
                            "not Integer",
                        )),
                    }?;
                    let y = match list.next_object()?.unwrap() {
                        Object::Integer(i) => Ok(i.parse::<u32>().unwrap()),
                        _ => Err(bendy::decoding::Error::unexpected_token(
                            "Integer for Node min_position y",
                            "not Integer",
                        )),
                    }?;
                    let z = match list.next_object()?.unwrap() {
                        Object::Integer(i) => Ok(i.parse::<u32>().unwrap()),
                        _ => Err(bendy::decoding::Error::unexpected_token(
                            "Integer for Node min_position z",
                            "not Integer",
                        )),
                    }?;
                    let min_position = Vector3::<u32> { x, y, z };
                    let dimension = match list.next_object()?.unwrap() {
                        Object::Integer(i) => Ok(i.parse::<u32>().unwrap()),
                        _ => Err(bendy::decoding::Error::unexpected_token(
                            "Integer for Node dimension",
                            "not Integer",
                        )),
                    }?;
                    let mut children: [Option<usize>; OCTREE_CHILDREN] = [None; OCTREE_CHILDREN];
                    match list.next_object()?.unwrap() {
                        Object::List(mut child_list) => Ok(for i in 0..OCTREE_CHILDREN {
                            children[i] = match child_list.next_object()?.unwrap() {
                                Object::Integer(i) => Ok(Some(i.parse::<usize>().unwrap())),
                                _ => Err(bendy::decoding::Error::unexpected_token("List", "not List")),
                            }?;
                            if children[i].unwrap() == 0 {
                                // 0 index value represents None in the helper index structure
                                children[i] = None;
                            }
                        }),
                        _ => Err(bendy::decoding::Error::unexpected_token("List", "not List")),
                    }?;
                    all_nodes.push((
                        Some(Node::<T> {
                            ty,
                            min_position,
                            dimension,
                            ..Default::default()
                        }),
                        children,
                    ));
                }

                // println!("Decode:");
                // let mut n_i = 0;
                // for n in all_nodes.iter() {
                //     let d_ty = match n.0.as_ref().unwrap().ty {
                //         NodeType::Internal => format!("INTERNAL"),
                //         NodeType::Leaf(d) => format!("{:?}", d),
                //     };

                //     let d_bounds = format!(
                //         "{:?};{:?}",
                //         n.0.as_ref().unwrap().min_position,
                //         n.0.as_ref().unwrap().dimension()
                //     );
                //     let mut d_children = "[".to_owned();
                //     for c in n.1 {
                //         match c {
                //             Some(index) => d_children.push_str(format!("{index},").as_str()),
                //             _ => d_children.push_str("x,"),
                //         }
                //     }
                //     d_children.push_str("]");
                //     println!("Nodes[{}]: [{}][{}]:{}", n_i, d_ty, d_bounds, d_children);
                //     n_i += 1;
                // }

                //Construct the tree structure from the serialized array
                let mut stack: VecDeque<(usize, usize, usize)> = VecDeque::new(); // Index of the Node, and index of its parent(who put it on the stack) along with the index of the child the Node is(parent's child index)
                stack.push_back((0, 0, 0));

                while 0 < stack.len() {
                    let (current_node, current_node_parent, parent_child_index) = stack.back().unwrap();
                    let mut current_child_index = 0; //Also contains the index of the child in which the helper index values and the Node<T>.children contents differ
                    for child_index in 0..OCTREE_CHILDREN {
                        if all_nodes[*current_node].1[child_index].is_none() //If the helper inde value
                            || all_nodes[*current_node].0.as_ref().unwrap().children[child_index].is_some()
                        {
                            current_child_index += 1;
                        } else {
                            break;
                        }
                    }
                    if current_child_index < OCTREE_CHILDREN {
                        stack.push_back((
                            all_nodes[*current_node].1[current_child_index].unwrap(),
                            *current_node,
                            current_child_index,
                        ));
                    } else {
                        //children are ready! let's push this item into a Box, add the dependency to its parent and remove it from stack!
                        //except for the root Node
                        if 0 != *current_node {
                            // move box into its parent Node
                            let node = std::mem::replace(&mut all_nodes[*current_node].0, None).unwrap(); //Move Node into a box
                            all_nodes[*current_node_parent].0.as_mut().unwrap().children[*parent_child_index] =
                                Some(Box::new(node));
                        }
                        stack.pop_back();
                    }
                }

                // Return the root Node
                Ok(std::mem::replace(&mut all_nodes[0].0, None).unwrap())
            }
            _ => Err(bendy::decoding::Error::unexpected_token("List", "not List")),
        }
    }
}
