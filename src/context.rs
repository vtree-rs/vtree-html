use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use nodes::{self, AllNodes};
use vtree::diff;

#[link(name="vtree_html")]
extern "C" {
    fn vtree_html_create_context(node_id: *const u8, node_id_len: usize) -> usize;
    fn vtree_html_remove_context(ctx_id: usize);
    fn vtree_html_diff(ctx_id: usize, diff: *const FFIDiff, len: usize);
}

#[repr(u32)]
#[derive(Debug)]
enum FFINodeType {
    Text = 0,
    Div = 1,
    Span = 2,
}

#[repr(u32)]
enum FFIDiffTag {
    Added = 0,
    Removed = 1,
    Reordered = 2,
    ParamSet = 3,
    ParamSetToTrue = 4,
    ParamRemoved = 5,
}

#[repr(C)]
struct FFIDiffAdded {
    id_parent: usize,
    ty: FFINodeType,
    index: usize,
}

// no need for "FFIDiffRemoved"

#[repr(C)]
struct FFIDiffReordered {
    last_index: usize,
    curr_index: usize,
}

#[repr(C)]
struct FFIDiffParamSet {
    key: *const u8,
    key_len: usize,
    value: *const u8,
    value_len: usize,
}

#[repr(C)]
struct FFIDiffParamSetToTrue {
    key: *const u8,
    key_len: usize,
}

#[repr(C)]
struct FFIDiffParamRemoved {
    key: *const u8,
    key_len: usize,
}

#[repr(C)]
union FFIDiffUnion {
    added: FFIDiffAdded,
    // no need for "removed"
    reordered: FFIDiffReordered,
    param_set: FFIDiffParamSet,
    param_set_to_true: FFIDiffParamSetToTrue,
    param_removed: FFIDiffParamRemoved,
}

#[repr(C)]
struct FFIDiff {
    id: usize,
    tag: FFIDiffTag,
    u: FFIDiffUnion,
}

struct DifferData<'a> {
    state: &'a mut NodeStates,
    diffs: Vec<FFIDiff>,
}

impl<'a> DifferData<'a> {
    fn new(state: &'a mut NodeStates) -> DifferData<'a> {
        DifferData {
            state: state,
            diffs: Vec::new(),
        }
    }

    fn get_next_id(&mut self) -> usize {
        // TODO: fix id overflow
        let id = self.state.next_node_id;
        self.state.next_node_id += 1;
        id
    }

    fn get_id_for_path(&self, path: &diff::Path) -> usize {
        match self.state.node_ids.get(path) {
            Some(id) => *id,
            None => panic!("couldn't find id for path `{}`", path),
        }
    }

    fn create_id_for_path(&mut self, path: &diff::Path) -> usize {
        let id = self.get_next_id();
        println!("create id for path: {} id: {}", path, id);
        self.state.node_ids.insert(path.clone(), id);
        id
    }
}

pub struct Differ<'a> {
    data: RefCell<DifferData<'a>>,
}

impl <'a> Differ<'a> {
    fn new(state: &'a mut NodeStates) -> Differ<'a> {
        Differ {
            data: RefCell::new(DifferData::new(state)),
        }
    }

    fn into_diffs(self) -> Vec<FFIDiff> {
        self.data.into_inner().diffs
    }
}

impl <'a> fmt::Debug for Differ<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Debug is unimplemented for Differ")
    }
}

fn param_to_ffi(id: usize, key: &str, value: &nodes::ParamValue, remove_false: bool) -> Option<FFIDiff> {
    let (tag, u) = match value {
        &nodes::ParamValue::String(ref v) => {
            (
                FFIDiffTag::ParamSet,
                FFIDiffUnion {
                    param_set: FFIDiffParamSet {
                        key: key.as_ptr(),
                        key_len: key.len(),
                        value: v.as_ptr(),
                        value_len: v.len(),
                    }
                },
            )
        }
        &nodes::ParamValue::Str(v) => {
            (
                FFIDiffTag::ParamSet,
                FFIDiffUnion {
                    param_set: FFIDiffParamSet {
                        key: key.as_ptr(),
                        key_len: key.len(),
                        value: v.as_ptr(),
                        value_len: v.len(),
                    }
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
                        param_set_to_true: FFIDiffParamSetToTrue {
                            key: key.as_ptr(),
                            key_len: key.len(),
                        }
                    },
                )
            } else {
                (
                    FFIDiffTag::ParamSetToTrue,
                    FFIDiffUnion {
                        param_set_to_true: FFIDiffParamSetToTrue {
                            key: key.as_ptr(),
                            key_len: key.len(),
                        }
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
            tag: FFIDiffTag::Added,
            u: FFIDiffUnion {
                added: FFIDiffAdded {
                    id_parent: id_parent,
                    ty: FFINodeType::Text,
                    index: index,
                }
            },
        });

        let key = "contents";
        data.diffs.push(FFIDiff {
            id: id,
            tag: FFIDiffTag::ParamSet,
            u: FFIDiffUnion {
                param_set: FFIDiffParamSet {
                    key: key.as_ptr(),
                    key_len: key.len(),
                    value: params.as_ptr(),
                    value_len: params.len(),
                }
            },
        });
        return;
    }

    let (ty, params) = match curr {
        &AllNodes::Div(nodes::Div { ref params, .. }) => (FFINodeType::Div, params),
        &AllNodes::Span(nodes::Span { ref params, .. }) => (FFINodeType::Span, params),
        &AllNodes::Text(_) | &AllNodes::Widget(_) => unreachable!(),
    };

    data.diffs.push(FFIDiff {
        id: id,
        tag: FFIDiffTag::Added,
        u: FFIDiffUnion {
            added: FFIDiffAdded {
                id_parent: id_parent,
                ty: ty,
                index: index,
            }
        },
    });
    data.diffs.extend(params_to_ffi(id, params, false));
}

impl<'a, 'b> diff::Differ<'a, AllNodes> for Differ<'b> {
    fn diff(&'a self, path: &diff::Path, diff: diff::Diff<'a, AllNodes>) {
        use self::diff::Diff;

        println!("{:?}", diff);

        let mut data = self.data.borrow_mut();

        match diff {
            Diff::Added {index, curr} => {
                curr.visit_enter(path, index, &mut |path, index, node| {
                    let id = data.create_id_for_path(path);
                    add_added_to_diffs(&mut *data, node, id, path, index);
                });
            }
            Diff::Removed {ref last, ..} => {
                last.visit_exit(path, 0, &mut |path, _, _| {
                    let id = data.state.node_ids.remove(path).unwrap();
                    println!("rm path: {} id: {:?}", path, id);
                    data.diffs.push(FFIDiff {
                        id: id,
                        tag: FFIDiffTag::Removed,
                        u: unsafe { ::std::mem::uninitialized() },
                    });
                });
            }
            Diff::Replaced {ref curr, ref last, index} => {
                last.visit_exit(path, 0, &mut |path, _, _| {
                    let id = data.state.node_ids.remove(path);
                    println!("rm path: {} id: {:?}", path, id);
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
            Diff::ParamsChanged { curr, last } => {
                match (curr, last) {
                    (
                        &AllNodes::Div(nodes::Div { params: nodes::Params{params: ref curr}, .. }),
                        &AllNodes::Div(nodes::Div { params: nodes::Params{params: ref last}, .. }),
                    ) |
                    (
                        &AllNodes::Span(nodes::Span { params: nodes::Params{params: ref curr}, .. }),
                        &AllNodes::Span(nodes::Span { params: nodes::Params{params: ref last}, .. }),
                    ) => {
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
                                        param_set_to_true: FFIDiffParamSetToTrue {
                                            key: key.as_ptr(),
                                            key_len: key.len(),
                                        }
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
                        let key = "contents";
                        data.diffs.push(FFIDiff {
                            id: id,
                            tag: FFIDiffTag::ParamSet,
                            u: FFIDiffUnion {
                                param_set: FFIDiffParamSet {
                                    key: key.as_ptr(),
                                    key_len: key.len(),
                                    value: curr.as_ptr(),
                                    value_len: curr.len(),
                                }
                            },
                        });
                    }
                    _ => unreachable!(),
                }
            }
            Diff::Reordered { indices } => {
                let (path_parent, _) = path.split_at(path.len() - 1);
                let id = data.get_id_for_path(&path_parent);
                let it = indices.into_iter().map(|(last_index, curr_index)| {
                    FFIDiff {
                        id: id,
                        tag: FFIDiffTag::Reordered,
                        u: FFIDiffUnion {
                            reordered: FFIDiffReordered {
                                last_index: last_index,
                                curr_index: curr_index,
                            }
                        },
                    }
                });
                data.diffs.extend(it);
            }
        }
    }
}

struct NodeStates {
    node_ids: HashMap<diff::Path, usize>,
    next_node_id: usize,
}

impl NodeStates {
    fn new() -> NodeStates {
        NodeStates {
            node_ids: HashMap::new(),
            next_node_id: 1,
        }
    }
}

pub struct Context {
    ctx_id: usize,
    last: AllNodes,
    diff_ctx: diff::Context<AllNodes>,
    node_states: NodeStates,
}

impl Context {
    pub fn new(node_id: &str, curr: AllNodes) -> Context {
        let ctx_id = unsafe {
            vtree_html_create_context(node_id.as_ptr(), node_id.len())
        };
        let diff_ctx = diff::Context::new();
        let path = diff::Path::new();
        let curr = curr.expand_widgets(None, &path);
        let mut node_states = NodeStates::new();
        let diffs = {
            let differ = Differ::new(&mut node_states);
            diff::Differ::diff(&differ, &path, diff::Diff::Added {
                index: 0,
                curr: &curr,
            });
            differ.into_diffs()
        };
        unsafe {
            vtree_html_diff(ctx_id, diffs.as_ptr(), diffs.len());
        }
        Context {
            ctx_id: ctx_id,
            diff_ctx: diff_ctx,
            last: curr,
            node_states: node_states,
        }
    }

    pub fn update(&mut self, curr: AllNodes) {
        let path = diff::Path::new();
        let curr = curr.expand_widgets(Some(&self.last), &path);
        let differ = Differ::new(&mut self.node_states);
        nodes::AllNodes::diff(&curr, &self.last, &path, 0, &self.diff_ctx, &differ);
        let diffs = differ.into_diffs();
        unsafe {
            vtree_html_diff(self.ctx_id, diffs.as_ptr(), diffs.len());
        }
        self.last = curr;
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            vtree_html_remove_context(self.ctx_id);
        }
    }
}
