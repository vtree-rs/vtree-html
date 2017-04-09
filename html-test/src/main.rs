extern crate vtree_html;
extern crate vtree;

use std::mem::forget;
use vtree_html::context::Context;
use vtree_html::nodes::{div, text, Params};

fn main() {
    let mut ctx = Context::new("ctx_node", text("bar"));
    ctx.update(
        div(
            Params::new(),
            (0..5).map(|i| (i, text(i.to_string())))
        )
    );

    ctx.update(
        div(
            Params::new(),
            [4, 0, 6, 1, 2].iter().map(|i| (i, text(i.to_string())))
        )
    );

    forget(ctx);
}
