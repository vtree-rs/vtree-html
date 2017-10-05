use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use nodes::{self, AllNodes};
use vtree::diff;
use stdweb::web;

#[derive(Debug)]
pub struct Differ<'a> {
    nodes: &'a mut HashMap<diff::Path, web::Node>,
}

impl <'a> Differ<'a> {
    fn new(nodes: &'a mut HashMap<diff::Path, web::Node>) -> Differ<'a> {
        Differ {
            nodes: nodes,
        }
    }

    fn get_node(&self, path: &diff::Path) -> Option<web::Node> {
        self.nodes.get(path)
    }
}

fn param_to_ffi(id: usize, key: &str, value: &nodes::ParamValue, remove_false: bool) -> Option<FFIDiff> {
    let (tag, u) = match value {
        &nodes::ParamValue::String(ref v) => {
            (
                FFIDiffTag::ParamSet,
                FFIDiffUnion {
                    param_set: FFIDiffParamSet {
                        key: FFIString::new(key),
                        value: FFIString::new(v),
                    },
                },
            )
        }
        &nodes::ParamValue::Str(v) => {
            (
                FFIDiffTag::ParamSet,
                FFIDiffUnion {
                    param_set: FFIDiffParamSet {
                        key: FFIString::new(key),
                        value: FFIString::new(v),
                    },
                },
            )
        }
        &nodes::ParamValue::Bool(v) => {
            if !v {
                if !remove_false {
                    return None;
                }

                (
                    FFIDiffTag::ParamRemoved,
                    FFIDiffUnion {
                        param_set_to_true: FFIString::new(key),
                    },
                )
            } else {
                (
                    FFIDiffTag::ParamSetToTrue,
                    FFIDiffUnion {
                        param_set_to_true: FFIString::new(key),
                    },
                )
            }
        }
    };

    Some(FFIDiff {
        id: id,
        tag: tag,
        u: u,
    })
}

fn params_to_ffi<'a>(id: usize, params: &'a nodes::Params, remove_false: bool) -> impl Iterator<Item=FFIDiff> + 'a {
    params.params.iter().filter_map(move |(ref key, ref value)| param_to_ffi(id, key, value, remove_false))
}

// fn diff_params_to_ffi<'a>(id: usize, curr: &'a nodes::Params, last: &'a nodes::Params)
//     -> impl Iterator<Item=FFIDiff> + 'a
// {
//     curr.params.iter().filter_map(|(ref key, ref value)| {
//
//     })
// }

fn add_added_to_diffs(data: &mut DifferData, curr: &AllNodes, id: usize, path: &diff::Path, index: usize) {
    let id_parent = if path.len() < 2 {
        0
    } else {
        let (path_parent, _) = path.split_at(path.len() - 2);
        *data.state.node_ids
            .get(&path_parent)
            .expect("unable to find the id for parent path")
    };

    if let &AllNodes::Text(nodes::Text { ref params }) = curr {
        data.diffs.push(FFIDiff {
            id: id,
            tag: FFIDiffTag::TextAdded,
            u: FFIDiffUnion {
                text_added: FFIDiffTextAdded {
                    id_parent: id_parent,
                    index: index,
                    text: FFIString::new(params),
                },
            },
        });
        return;
    }

    macro_rules! gen_add_match {
        ($(($id:expr, $name_up:ident, $name_low:ident),)*) => {
            match curr {
                $(
                    &AllNodes::$name_up(nodes::$name_up { ref params, .. }) =>
                        (FFINodeType::$name_up, params),
                )*
                &AllNodes::Text(_) | &AllNodes::Widget(_) => unreachable!(),
            }
        };
    }
    let (ty, params) = html_nodes!(gen_add_match);

    data.diffs.push(FFIDiff {
        id: id,
        tag: FFIDiffTag::Added,
        u: FFIDiffUnion {
            added: FFIDiffAdded {
                id_parent: id_parent,
                ty: ty,
                index: index,
            },
        },
    });
    data.diffs.extend(params_to_ffi(id, params, false));
}

impl<'b> diff::Differ<AllNodes> for Differ<'b> {
    fn diff_added(&self, path: &diff::Path, index: usize, curr: &AllNodes) {
        let mut data = self.data.borrow_mut();
        curr.visit_enter(path, index, &mut |path, index, node| {
            let id = data.create_id_for_path(path);
            add_added_to_diffs(&mut *data, node, id, path, index);
        });
    }

    fn diff_removed(&self, path: &diff::Path, _index: usize, last: &AllNodes) {
        let mut data = self.data.borrow_mut();
        last.visit_exit(path, 0, &mut |path, _, _| {
            let id = data.state.node_ids.remove(path).unwrap();
            data.diffs.push(FFIDiff {
                id: id,
                tag: FFIDiffTag::Removed,
                u: unsafe { ::std::mem::uninitialized() },
            });
        });
    }

    fn diff_replaced(&self, path: &diff::Path, index: usize, curr: &AllNodes, last: &AllNodes) {
        let mut data = self.data.borrow_mut();
        last.visit_exit(path, 0, &mut |path, _, _| {
            let id = data.state.node_ids.remove(path);
            let id = if let Some(id) = id {
                id
            } else {
                panic!("wtf path: {}", path);
            };
            data.diffs.push(FFIDiff {
                id: id,
                tag: FFIDiffTag::Removed,
                u: unsafe { ::std::mem::uninitialized() },
            });
        });

        curr.visit_enter(path, index, &mut |path, index, node| {
            let id = data.create_id_for_path(path);
            add_added_to_diffs(&mut *data, node, id, path, index);
        });
    }

    fn diff_params_changed(&self, path: &diff::Path, curr: &AllNodes, last: &AllNodes) {
        macro_rules! gen_params_changed {
            ($(($id:expr, $name_up:ident, $name_low:ident),)*) => {

                let mut data = self.data.borrow_mut();
                match (curr, last) {
                    $(
                        (
                            &AllNodes::$name_up(nodes::$name_up { params: nodes::Params{params: ref curr}, .. }),
                            &AllNodes::$name_up(nodes::$name_up { params: nodes::Params{params: ref last}, .. }),
                        )
                    )|* => {
                        let id = data.get_id_for_path(path);
                        for (key, val_curr) in curr.iter() {
                            if let Some(val_last) = last.get(key) {
                                if val_curr != val_last {
                                    data.diffs.extend(param_to_ffi(id, key, val_curr, true));
                                }
                            } else {
                                data.diffs.extend(param_to_ffi(id, key, val_curr, false));
                            }
                        }

                        for key in last.keys() {
                            if !curr.contains_key(key) {
                                data.diffs.push(FFIDiff {
                                    id: id,
                                    tag: FFIDiffTag::ParamRemoved,
                                    u: FFIDiffUnion {
                                        param_set_to_true: FFIString::new(key),
                                    },
                                });
                            }
                        }
                    }
                    (
                        &AllNodes::Text(nodes::Text { params: ref curr }),
                        &AllNodes::Text(_),
                    ) => {
                        let id = data.get_id_for_path(path);
                        data.diffs.push(FFIDiff {
                            id: id,
                            tag: FFIDiffTag::TextSet,
                            u: FFIDiffUnion {
                                text_set: FFIString::new(curr),
                            },
                        });
                    }
                    _ => unreachable!(),
                }
            };
        };

        html_nodes!(gen_params_changed);
    }

    fn diff_reordered<I: Iterator<Item=(usize, usize)>>(&self, path: &diff::Path, indices: I) {
        let mut ffi_reordered: Vec<_> = indices
            .map(|(c_i, l_i)| {
                FFIReordered {
                    curr_index: c_i,
                    last_index: l_i,
                }
            })
            .collect();
        if ffi_reordered.is_empty() {
            return;
        }
        let mut data = self.data.borrow_mut();
        let (path_parent, _) = path.split_at(path.len() - 1);
        let id = data.get_id_for_path(&path_parent);
        ffi_reordered.sort_by_key(|e| e.curr_index);
        println!("{:?}", ffi_reordered);
        data.diffs.push(FFIDiff {
            id: id,
            tag: FFIDiffTag::Reordered,
            u: FFIDiffUnion {
                reordered: FFIDiffReordered {
                    ptr: ffi_reordered.as_ptr(),
                    len: ffi_reordered.len(),
                }
            },
        });
        ::std::mem::forget(ffi_reordered);
    }
}

pub struct Context {
    last: AllNodes,
    diff_ctx: diff::Context<AllNodes>,
    node_states: NodeStates,
}

impl Context {
    pub fn new(node_id: &str, mut curr: AllNodes) -> Context {
        let diff_ctx = diff::Context::new();
        let path = diff::Path::new();
        AllNodes::expand_widgets(&mut curr, None, &path);
        let mut node_states = NodeStates::new();
        Differ::new(&mut node_states).diff_added(&path, 0, &curr);
        Context {
            diff_ctx: diff_ctx,
            last: curr,
            node_states: node_states,
        }
    }

    pub fn update(&mut self, mut curr: AllNodes) {
        let path = diff::Path::new();
        AllNodes::expand_widgets(&mut curr, None, &path);
        nodes::AllNodes::diff(&curr, &self.last, &path, 0, &self.diff_ctx, &Differ::new(&mut self.node_states));
        self.last = curr;
    }
}
