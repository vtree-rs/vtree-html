extern crate vtree_html;
extern crate vtree;

use std::mem::forget;
use vtree_html::context::Context;
use vtree_html::nodes::{div, text, Params};

fn main() {
    let mut ctx = Context::new("ctx_node", text("bar"));
    ctx.update(
        div(Params::new(), &[
            (0, text("123")),
            (1, text("456")),
        ])
    );

    ctx.update(
        div(Params::new(), &[
            (0, text("789")),
            (1, text("456")),
        ])
    );

    forget(ctx);
}
