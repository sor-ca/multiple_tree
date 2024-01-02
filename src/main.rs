use std::collections::HashMap;

use r3bl_rs_utils::Arena;

#[derive(Clone, Debug)]
pub struct Traj {
    pub path: Vec<i32>,
}

#[derive(Clone, Debug)]
pub struct Element {
    pub id: usize,
    pub path: Vec<i32>,
    pub trajs: Vec<usize>,
    pub parent: usize,
    pub children:  Vec<usize>,
}

pub fn branch_vector(trajs: &Vec<Traj>) -> Vec<Element> {
    let mut final_vec: Vec<Element> = Vec::with_capacity(100);

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
            parent: 0,
            path: vec![],
            trajs: vec![0, 1, 2, 3],
            children: vec![],
        };

        active_parents.push(null_parent);
        //primitive id fabric
        let mut count = 1usize;

        for index in 0..max_len {
            //vector of id for pushing from active parents to final vector
            let mut push_to_final = vec![];
            let len = active_parents.len();

            for i in 0..len {
                let mut new_parents: HashMap<i32, Element> = HashMap::new();

                let parent_trajs = active_parents[i].trajs.clone();
                for traj_id in &parent_trajs {
                    if let Some(traj) = trajs.get(*traj_id) {
                        if let Some(point) = traj.path.get(index) {
                            match new_parents.get_mut(point) {
                                Some(el) => el.trajs.push(*traj_id),
                                None => {
                                    new_parents.insert(
                                        *point,
                                        Element {
                                            id: count,
                                            parent: active_parents[i].id,
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
                        active_parents[i]
                            .path
                            .push(new_parents.into_keys().next().unwrap());
                        count -= 1;
                    }
                    _ => {
                        //several new children
                        let mut children: Vec<_> = new_parents.into_values().collect();
                        let children_ids = children.iter().map(|ch|ch.id).collect();
                        active_parents[i].children = children_ids;
                        active_parents.append(&mut children);
                        push_to_final.push(i);
                    }
                }
            }
            //reverse vector to keep the correct indexes after removing
            push_to_final.reverse();
            for i in push_to_final {
                let final_el = active_parents.remove(i);
                final_vec.push(final_el);
            }
        }
        final_vec.append(&mut active_parents);
    }

    final_vec
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

    let vec = branch_vector(&trajs);
    dbg!(&vec.len());

    let arena = arena_tree(&trajs);
    dbg!(&arena.tree_walk_dfs(0).unwrap());
}
