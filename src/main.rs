//some good ideas about multiple trees
//https://developerlife.com/2022/02/24/rust-non-binary-tree/

//the task is that we have several trajectories which start in one point and then branch, forming a multiple tree
//we need to create a collection of elements
//each element includes the path segments between the points of branching 
//and the numbers of the trajectories which follow this path segment
//each trajectory has its own path - the sorted collecton of points starting from zero. 
//In real situation, each path includes several thousands of points

use std::collections::{HashMap, VecDeque};

use r3bl_rs_utils::Arena;

#[derive(Clone, Debug)]
pub struct Traj {
    pub path: Vec<i32>,
}

#[derive(Clone, Debug, Default)]
pub struct Element {
    pub id: usize,
    pub path: Vec<i32>,
    pub trajs: Vec<usize>,
    pub parent: Option<usize>,
    pub children:  Vec<usize>,
}

impl Element {
    fn new(id: usize) -> Self {
        Self {
            id,
            path: vec![],
            trajs: vec![],
            parent: None,
            children:  vec![],
        }
    }

    fn set_parent(&mut self, id: usize) {
        self.parent = Some(id);
    }

    fn add_child(&mut self, id: usize) {
        self.children.push(id);
    }

    fn add_traj(&mut self, id: usize) {
        self.trajs.push(id);
    }

    fn add_point(&mut self, point: i32) {
        self.path.push(point);
    }

    fn add_children(&mut self, children: &mut Vec<usize>) {
        self.children.append(children);
    }
}

#[derive(Clone, Debug)]
pub struct HashElement {
    pub path: Vec<i32>,
    pub trajs: Vec<usize>,
    pub parent: Option<usize>,
    pub children:  Vec<usize>,
}

impl From<Element> for HashElement {
    fn from(value: Element) -> Self {
        Self {
            path: value.path,
            trajs: value.trajs,
            parent: value.parent,
            children: value.children,
        }
    }
}

pub fn branch_vector(trajs: &Vec<Traj>) -> HashMap<usize, HashElement> {
    let mut hash_tree: HashMap<usize, HashElement> = HashMap::new();

    if !trajs.is_empty() {
        let max_len = trajs
            .iter()
            .max_by_key(|t| t.path.len())
            .unwrap()
            .path
            .len();

        let mut active_parents: Vec<Element> = Vec::with_capacity(20);

        let null_parent = Element {
            id: 0,
            parent: None,
            path: vec![],
            trajs: vec![0, 1, 2, 3],
            children: vec![],
        };

        active_parents.push(null_parent);
        //primitive id fabric
        let mut count = 1usize;

        for index in 0..max_len {
            //vector of id for pushing from active parents to hash_tree
            let mut push_to_final = vec![];
            let len = active_parents.len();

            for i in 0..len {
                let mut new_parents: HashMap<i32, Element> = HashMap::new();

                let parent_trajs = active_parents[i].trajs.clone();
                for traj_id in &parent_trajs {
                    if let Some(traj) = trajs.get(*traj_id) {
                        if let Some(point) = traj.path.get(index) {
                            match new_parents.get_mut(point) {
                                Some(el) => el.add_traj(*traj_id), //el.trajs.push(*traj_id),
                                None => {
                                    new_parents.insert(
                                        *point,
                                        Element {
                                            id: count,
                                            parent: Some(active_parents[i].id),
                                            path: vec![*point],
                                            trajs: vec![*traj_id],
                                            children: vec![],
                                        },
                                    );
                                    count += 1;
                                }
                            }
                        }
                    }
                }
                match new_parents.len() {
                    0 => {
                        //no elements in trajs of this parent
                        push_to_final.push(i);
                    }
                    1 => {
                        //the i-th element of all the trajs if this parent is the same
                        let point = new_parents.into_keys().next().unwrap();
                        active_parents[i].add_point(point);
                        count -= 1;
                    }
                    _ => {
                        //several new children
                        let mut children: Vec<_> = new_parents.into_values().collect();
                        let mut children_ids = children.iter().map(|ch|ch.id).collect();
                        active_parents[i].add_children(&mut children_ids);
                        active_parents.append(&mut children);
                        push_to_final.push(i);
                    }
                }
            }
            //reverse vector to keep the correct indexes after removing
            push_to_final.reverse();
            for i in push_to_final {
                let el = active_parents.remove(i);
                hash_tree.insert(el.id, el.into());
            }
        }
        for el in active_parents.into_iter() {
            hash_tree.insert(el.id, el.into());
        }
    }

    hash_tree
}

//from Arena sources, adapted for hash map
//[DFS graph walking](https://developerlife.com/2018/08/16/algorithms-in-kotlin-5/)
//[DFS tree walking](https://stephenweiss.dev/algorithms-depth-first-search-dfs#handling-non-binary-trees)
pub fn tree_walk_dfs_for_hash_tree(hash_tree: &HashMap<usize, HashElement>, node_id: usize) -> Option<VecDeque<usize>> {
    if !hash_tree.contains_key(&node_id) {
        return None;
    }

    let mut stack: VecDeque<usize> = VecDeque::from([node_id]);
    let mut it: VecDeque<usize> = VecDeque::new();

    while let Some(node_id) = stack.pop_back() {
        let node = hash_tree.get(&node_id)?;
        dbg!(&node);
        it.push_back(node_id);
        // Note that the children ordering has to be flipped! You want to perform the
        // traversal from RIGHT -> LEFT (not LEFT -> RIGHT).
        // PTAL: <https://developerlife.com/assets/algo-ts-2-images/depth-first-search.svg>
        for child_id in node.children.iter().rev() {
                stack.push_back(*child_id);
        }
    }

    match it.len() {
        0 => None,
        _ => Some(it),
    }
}

#[derive(Clone, Debug)]
pub struct ArenaPayload {
    pub path: Vec<i32>,
    pub trajs: Vec<usize>,
}

pub fn arena_tree(trajs: &Vec<Traj>) -> Arena<ArenaPayload> {
    let mut arena = Arena::<ArenaPayload>::new();

    if !trajs.is_empty() {
        let max_len = trajs
            .iter()
            .max_by_key(|t| t.path.len())
            .unwrap()
            .path
            .len();

        let mut active_parents: Vec<usize> = Vec::with_capacity(20);
        let null_parent = ArenaPayload {
            path: vec![],
            trajs: vec![0, 1, 2, 3],
        };

        let null_id = arena.add_new_node(null_parent, None);
        active_parents.push(null_id);

        for index in 0..max_len {
            //vector of id for pushing from active parents to final vector
            let parents = active_parents.clone();

            for parent_id in parents {
                let mut new_parents: HashMap<i32, ArenaPayload> = HashMap::new();
                let parent_trajs = arena
                    .get_node_arc(parent_id)
                    .unwrap()
                    .read()
                    .unwrap()
                    .payload
                    .trajs
                    .clone();

                for traj_id in parent_trajs {
                    if let Some(traj) = trajs.get(traj_id) {
                        if let Some(point) = traj.path.get(index) {
                            match new_parents.get_mut(point) {
                                Some(el) => el.trajs.push(traj_id),
                                None => {
                                    new_parents.insert(
                                        *point,
                                        ArenaPayload {
                                            path: vec![*point],
                                            trajs: vec![traj_id],
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
                match new_parents.len() {
                    0 => {
                        //no elements in trajs of this parent
                        active_parents.retain(|id| *id != parent_id);
                    }
                    1 => {
                        //the i-th element of all the trajs if this parent is the same
                        let parent = arena.get_node_arc(parent_id).unwrap();
                        parent
                            .write()
                            .unwrap()
                            .payload
                            .path
                            .push(new_parents.into_keys().next().unwrap());
                    }
                    _ => {
                        //several new children
                        let mut add_to_arena = new_parents
                            .into_iter()
                            .map(|(_k, payload)| arena.add_new_node(payload, Some(parent_id)))
                            .collect();
                        active_parents.retain(|id| *id != parent_id);
                        active_parents.append(&mut add_to_arena);
                    }
                }
            } 
        }
    }
    arena
}

fn main() {
    let trajs = vec![
        Traj {
            path: vec![1, 2, 3, 3, 3, 3],
        },
        Traj {
            path: vec![1, 2, 3, 3, 4, 3],
        },
        Traj {
            path: vec![1, 2, 3, 4, 5, 6],
        },
        Traj {
            path: vec![1, 2, 5, 5, 5, 5, 5, 5],
        },
    ];

    let hash_tree = branch_vector(&trajs);
    //dbg!(&hash_tree.len());
    dbg!(tree_walk_dfs_for_hash_tree(&hash_tree, 0));

    //let arena = arena_tree(&trajs);
    //dbg!(&arena.tree_walk_dfs(0).unwrap());
}
